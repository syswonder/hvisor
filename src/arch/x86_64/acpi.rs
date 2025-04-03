use crate::{
    config::HvConfigMemoryRegion,
    error::HvResult,
    percpu::this_zone,
    platform::qemu_x86_64::{ROOT_ZONE_ACPI_REGION, ROOT_ZONE_RSDP_REGION},
};
use acpi::{
    fadt::Fadt,
    madt::{LocalApicEntry, Madt, MadtEntry},
    mcfg::Mcfg,
    rsdp::Rsdp,
    sdt::{SdtHeader, Signature},
    AcpiHandler, AcpiTables, AmlTable, PciConfigRegions,
};
use alloc::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    vec::Vec,
};
use core::{pin::Pin, ptr::NonNull};
use spin::Mutex;

const RSDP_V1_SIZE: usize = 20;
const RSDP_V2_SIZE: usize = 36;

const RSDP_RSDT_OFFSET: usize = 16;
const RSDP_RSDT_PTR_SIZE: usize = 4;
const RSDT_PTR_SIZE: usize = 4;

const FADT_DSDT_OFFSET_32: usize = 0x28;
const FADT_DSDT_OFFSET_64: usize = 0x8c;

const FADT_FACS_OFFSET_32: usize = 0x24;
const FADT_FACS_OFFSET_64: usize = 0x84;

const SDT_HEADER_SIZE: usize = 36;

macro_rules! acpi_table {
    ($a: ident, $b: ident) => {
        #[repr(transparent)]
        struct $a {
            header: SdtHeader,
        }

        unsafe impl acpi::AcpiTable for $a {
            const SIGNATURE: Signature = Signature::$b;
            fn header(&self) -> &SdtHeader {
                &self.header
            }
        }
    };
}

#[derive(Clone, Debug)]
struct HvAcpiHandler {}

impl AcpiHandler for HvAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping::new(
            physical_address,
            NonNull::new(physical_address as *mut T).unwrap(),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {}
}

lazy_static::lazy_static! {
    static ref ROOT_ACPI: Mutex<RootAcpi> = {
        Mutex::new(RootAcpi::default())
    };
}

#[derive(Clone, Debug, Default)]
pub struct AcpiTable {
    bytes: Vec<u8>,
    gpa: usize,
    hpa: usize,
    is_addr_set: bool,
    is_dirty: bool,
}

impl AcpiTable {
    pub fn set_u8(&mut self, value: u8, offset: usize) {
        self.bytes[offset] = value;
        self.is_dirty = true;
    }

    pub fn set_u32(&mut self, value: u32, offset: usize) {
        let bytes = value.to_ne_bytes();
        self.bytes[offset..offset + 4].copy_from_slice(&bytes);
        self.is_dirty = true;
    }

    pub fn set_u64(&mut self, value: u64, offset: usize) {
        let bytes = value.to_ne_bytes();
        self.bytes[offset..offset + 8].copy_from_slice(&bytes);
        self.is_dirty = true;
    }

    // not for rsdp
    pub fn set_len(&mut self, len: usize) {
        self.bytes.resize(len, 0);
        self.set_u32(len as u32, 4);
        self.is_dirty = true;
    }

    pub fn get_len(&self) -> usize {
        self.bytes.len()
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn get_u8(&self, offset: usize) -> u8 {
        self.bytes[offset]
    }

    pub fn get_u16(&self, offset: usize) -> u16 {
        let bytes: [u8; 2] = self.bytes[offset..offset + 2].try_into().unwrap();
        u16::from_ne_bytes(bytes)
    }

    pub fn get_u32(&self, offset: usize) -> u32 {
        let bytes: [u8; 4] = self.bytes[offset..offset + 4].try_into().unwrap();
        u32::from_ne_bytes(bytes)
    }

    pub fn get_u64(&self, offset: usize) -> u64 {
        let bytes: [u8; 8] = self.bytes[offset..offset + 8].try_into().unwrap();
        u64::from_ne_bytes(bytes)
    }

    pub fn fill(&mut self, ptr: *const u8, len: usize) {
        self.bytes.clear();
        if self.bytes.capacity() < len {
            self.bytes.reserve(len);
        }

        unsafe {
            core::ptr::copy_nonoverlapping(ptr, self.bytes.as_mut_ptr(), len);
            self.bytes.set_len(len);
        }
    }

    pub fn copy_to_mem(&self) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.bytes.as_ptr(),
                self.hpa as *mut u8,
                self.bytes.len(),
            )
        };
    }

    pub fn remove(&mut self, start: usize, len: usize) {
        let tot_len = self.bytes.len();
        let end = start + len;
        assert!(end <= tot_len);

        if len == 0 {
            return;
        }

        unsafe {
            let ptr = self.bytes.as_mut_ptr();
            core::ptr::copy(ptr.add(end), ptr.add(start), tot_len - end);
        }
        self.set_len(tot_len - len);
    }

    pub fn set_addr(&mut self, hpa: usize, gpa: usize) {
        self.hpa = hpa;
        self.gpa = gpa;
        self.is_addr_set = true;
    }

    /// for rsdp, offset = 8; for the others, offset = 9.
    pub fn update_checksum(&mut self, offset: usize) {
        self.bytes[offset] = 0;
        let sum = self
            .bytes
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte));
        self.bytes[offset] = 0u8.wrapping_sub(sum);
    }
}

#[derive(Copy, Clone, Debug)]
struct AcpiPointer {
    pub from_sig: Signature,
    pub from_offset: usize,
    pub to_sig: Signature,
    pub pointer_size: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RootAcpi {
    rsdp: AcpiTable,
    tables: BTreeMap<Signature, AcpiTable>,
    pointers: Vec<AcpiPointer>,
}

impl RootAcpi {
    fn add_pointer(
        &mut self,
        from_sig: Signature,
        from_offset: usize,
        to_sig: Signature,
        pointer_size: usize,
    ) {
        self.pointers.push(AcpiPointer {
            from_sig,
            from_offset,
            to_sig,
            pointer_size,
        });
    }

    fn add_new_table(&mut self, sig: Signature, ptr: *const u8, len: usize) {
        let mut table = AcpiTable::default();
        table.fill(ptr, len);
        self.tables.insert(sig, table);
    }

    fn get_mut_table(&mut self, sig: Signature) -> Option<&mut AcpiTable> {
        self.tables.get_mut(&sig)
    }

    fn get_table(&self, sig: &Signature) -> Option<AcpiTable> {
        if self.tables.contains_key(sig) {
            Some(self.tables.get(sig).unwrap().clone())
        } else {
            None
        }
    }

    pub fn copy_to_zone_region(
        &self,
        rsdp_zone_region: &HvConfigMemoryRegion,
        acpi_zone_region: &HvConfigMemoryRegion,
    ) {
        let mut rsdp = self.rsdp.clone();
        let mut tables = self.tables.clone();

        // set rsdp addr
        rsdp.set_addr(
            rsdp_zone_region.physical_start as _,
            rsdp_zone_region.virtual_start as _,
        );

        let cpu_set = this_zone().read().cpu_set;
        let mut madt_cur: usize = SDT_HEADER_SIZE + 8;
        let mut madt = tables.get_mut(&Signature::MADT).unwrap();

        // fix madt cpu info
        for entry in
            unsafe { Pin::new_unchecked(&*(madt.get_bytes().clone().as_ptr() as *const Madt)) }
                .entries()
        {
            let mut entry_len = madt.get_u8(madt_cur + 1) as usize;
            match entry {
                MadtEntry::LocalApic(entry) => {
                    if !cpu_set.contains_cpu(entry.processor_id as _) {
                        madt.remove(madt_cur, entry_len);
                        entry_len = 0;
                    }
                }
                MadtEntry::LocalX2Apic(entry) => {
                    if !cpu_set.contains_cpu(entry.processor_uid as _) {}
                }
                _ => {}
            }
            madt_cur += entry_len;
        }

        // set pointers
        let hpa_start = acpi_zone_region.physical_start as usize;
        let gpa_start = acpi_zone_region.virtual_start as usize;
        let mut cur: usize = 0;

        let mut tables_involved = BTreeSet::<Signature>::new();

        for pointer in self.pointers.iter() {
            let to = tables.get_mut(&pointer.to_sig).unwrap();
            tables_involved.insert(pointer.to_sig);

            if !to.is_addr_set {
                info!(
                    "sig: {:x?}, hpa: {:x?}, gpa: {:x?}, size: {:x?}",
                    pointer.to_sig,
                    hpa_start + cur,
                    gpa_start + cur,
                    to.get_len()
                );
                to.set_addr(hpa_start + cur, gpa_start + cur);
                cur += to.get_len();
            }
            let to_gpa = to.gpa;

            let from = match pointer.from_sig == pointer.to_sig {
                true => &mut rsdp,
                false => tables.get_mut(&pointer.from_sig).unwrap(),
            };
            match pointer.pointer_size {
                4 => {
                    from.set_u32(to_gpa as _, pointer.from_offset);
                }
                8 => {
                    from.set_u64(to_gpa as _, pointer.from_offset);
                }
                _ => {
                    warn!("Unused pointer size!");
                }
            }
        }

        // update checksums
        rsdp.update_checksum(8);
        for (sig, table) in tables.iter_mut() {
            if table.is_dirty {
                table.update_checksum(9);
            }
        }

        // copy to memory
        rsdp.copy_to_mem();
        for (sig, table) in tables.iter() {
            // don't copy tables that are not inside ACPI tree
            if tables_involved.contains(sig) {
                table.copy_to_mem();
            }
        }
    }

    // let zone 0 bsp cpu does the work
    pub fn init(&mut self) {
        let rsdp_mapping = unsafe { Rsdp::search_for_on_bios(HvAcpiHandler {}).unwrap() };
        // FIXME: temporarily suppose we use ACPI 1.0
        assert!(rsdp_mapping.revision() == 0);

        self.rsdp.fill(
            rsdp_mapping.virtual_start().as_ptr() as *const u8,
            RSDP_V1_SIZE,
        );
        self.add_pointer(
            Signature::RSDT,
            RSDP_RSDT_OFFSET,
            Signature::RSDT,
            RSDP_RSDT_PTR_SIZE,
        );

        // get rsdt

        self.add_new_table(
            Signature::RSDT,
            rsdp_mapping.rsdt_address() as usize as *const u8,
            SDT_HEADER_SIZE,
        );
        let mut rsdt_offset = self.get_mut_table(Signature::RSDT).unwrap().get_len();

        let tables =
            unsafe { AcpiTables::from_validated_rsdp(HvAcpiHandler {}, rsdp_mapping) }.unwrap();

        if let Ok(madt) = tables.find_table::<Madt>() {
            self.add_new_table(
                Signature::MADT,
                madt.physical_start() as *const u8,
                madt.region_length(),
            );

            info!("-------------------------------- MADT --------------------------------");
            for entry in madt.get().entries() {
                info!("{:x?}", entry);
            }

            self.add_pointer(Signature::RSDT, rsdt_offset, Signature::MADT, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;
        }

        if let Ok(mcfg) = tables.find_table::<Mcfg>() {
            self.add_new_table(
                Signature::MCFG,
                mcfg.physical_start() as *const u8,
                mcfg.region_length(),
            );

            info!("-------------------------------- MCFG --------------------------------");
            for entry in mcfg.entries() {
                info!("{:x?}", entry);
            }

            self.add_pointer(Signature::RSDT, rsdt_offset, Signature::MCFG, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;
        }

        if let Ok(fadt) = tables.find_table::<Fadt>() {
            self.add_new_table(
                Signature::FADT,
                fadt.physical_start() as *const u8,
                fadt.region_length(),
            );

            self.add_pointer(Signature::RSDT, rsdt_offset, Signature::FADT, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;

            // dsdt

            if let Ok(dsdt) = tables.dsdt() {
                self.add_new_table(
                    Signature::DSDT,
                    (dsdt.address - SDT_HEADER_SIZE) as *const u8,
                    (dsdt.length as usize + SDT_HEADER_SIZE),
                );

                self.add_pointer(Signature::FADT, FADT_DSDT_OFFSET_32, Signature::DSDT, 4);
                self.add_pointer(Signature::FADT, FADT_DSDT_OFFSET_64, Signature::DSDT, 8);
            }

            // facs

            if let Ok(facs_addr) = fadt.facs_address() {
                self.add_new_table(Signature::FACS, facs_addr as *const u8, unsafe {
                    *((facs_addr + 4) as *const u32) as usize
                });

                self.add_pointer(Signature::FADT, FADT_FACS_OFFSET_32, Signature::FACS, 4);
                self.add_pointer(Signature::FADT, FADT_FACS_OFFSET_64, Signature::FACS, 8);
            }
        }

        acpi_table!(Dmar, DMAR);
        if let Ok(dmar) = tables.find_table::<Dmar>() {
            self.add_new_table(
                Signature::DMAR,
                dmar.physical_start() as *const u8,
                dmar.region_length(),
            );

            info!("dmar: {:x?}", unsafe {
                *((dmar.physical_start() + 56) as *const [u8; 8])
            });

            // self.add_pointer(Signature::RSDT, rsdt_offset, Signature::DMAR, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;
        }

        if let Some(rsdt) = self.get_mut_table(Signature::RSDT) {
            rsdt.set_len(rsdt_offset);
        }
    }
}

// let zone 0 bsp cpu does the work
pub fn root_init() {
    ROOT_ACPI.lock().init();
}

pub fn copy_to_root_zone_region() {
    ROOT_ACPI
        .lock()
        .copy_to_zone_region(&ROOT_ZONE_RSDP_REGION, &ROOT_ZONE_ACPI_REGION);
}

pub fn root_get_table(sig: &Signature) -> Option<AcpiTable> {
    ROOT_ACPI.lock().get_table(sig)
}
