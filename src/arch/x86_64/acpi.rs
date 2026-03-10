// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::boot,
    config::{HvConfigMemoryRegion, HvZoneConfig},
    cpu_data::{this_zone, CpuSet},
    error::HvResult,
    platform::ROOT_PCI_MAX_BUS,
};
use acpi::{
    fadt::Fadt,
    madt::{LocalApicEntry, Madt, MadtEntry},
    mcfg::{Mcfg, McfgEntry},
    rsdp::Rsdp,
    sdt::{SdtHeader, Signature},
    AcpiHandler, AcpiTables, AmlTable, PciConfigRegions,
};
use alloc::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    vec::Vec,
};
use core::{
    any::Any,
    mem::size_of,
    pin::Pin,
    ptr::{read_unaligned, write_unaligned, NonNull},
    slice,
};
use spin::{Mutex, Once};

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

const RSDP_CHECKSUM_OFFSET: usize = 8;
const ACPI_CHECKSUM_OFFSET: usize = 9;

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

static ROOT_ACPI: Once<RootAcpi> = Once::new();

#[derive(Clone, Debug)]
enum PatchValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

#[derive(Clone, Debug, Default)]
pub struct AcpiTable {
    sig: Option<Signature>,
    src: usize,
    patches: BTreeMap<usize, PatchValue>,
    len: usize,
    checksum: u8,
    gpa: usize,
    hpa: usize,
    is_addr_set: bool,
}

fn get_byte_sum_u32(value: u32) -> u8 {
    value
        .to_ne_bytes()
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b))
}

fn get_byte_sum_u64(value: u64) -> u8 {
    value
        .to_ne_bytes()
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b))
}

impl AcpiTable {
    pub fn set_u8(&mut self, value: u8, offset: usize) {
        self.patches.insert(offset, PatchValue::U8(value));
        let old = unsafe { *((self.src + offset) as *const u8) };
        self.checksum = self.checksum.wrapping_add(old).wrapping_sub(value);
    }

    pub fn set_u32(&mut self, value: u32, offset: usize) {
        self.patches.insert(offset, PatchValue::U32(value));
        let old = unsafe { read_unaligned((self.src + offset) as *const u32) };
        self.checksum = self
            .checksum
            .wrapping_add(get_byte_sum_u32(old))
            .wrapping_sub(get_byte_sum_u32(value));
    }

    pub fn set_u64(&mut self, value: u64, offset: usize) {
        self.patches.insert(offset, PatchValue::U64(value));
        let old = unsafe { read_unaligned((self.src + offset) as *const u64) };
        self.checksum = self
            .checksum
            .wrapping_add(get_byte_sum_u64(old))
            .wrapping_sub(get_byte_sum_u64(value));
    }

    /// new len must not be longer
    pub fn set_new_len(&mut self, len: usize) {
        let src_len = self.get_u32(4) as usize;
        println!("len: {:x}, selflen: {:x}", len, src_len);
        assert!(len <= src_len);

        // update checksum
        for offset in len..src_len {
            self.checksum = self
                .checksum
                .wrapping_add(unsafe { *((self.src + offset) as *const u8) });
        }

        self.set_u32(len as _, 4);
        self.len = len;
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_unpatched_src(&self) -> *const u8 {
        self.src as *const u8
    }

    pub fn get_u8(&self, offset: usize) -> u8 {
        if let Some(&PatchValue::U8(value)) = self.patches.get(&offset) {
            return value;
        }
        unsafe { *((self.src + offset) as *const u8) }
    }

    pub fn get_u16(&self, offset: usize) -> u16 {
        if let Some(&PatchValue::U16(value)) = self.patches.get(&offset) {
            return value;
        }
        unsafe { read_unaligned((self.src + offset) as *const u16) }
    }

    pub fn get_u32(&self, offset: usize) -> u32 {
        if let Some(&PatchValue::U32(value)) = self.patches.get(&offset) {
            return value;
        }
        unsafe { read_unaligned((self.src + offset) as *const u32) }
    }

    pub fn get_u64(&self, offset: usize) -> u64 {
        if let Some(&PatchValue::U64(value)) = self.patches.get(&offset) {
            return value;
        }
        unsafe { read_unaligned((self.src + offset) as *const u64) }
    }

    pub fn fill(
        &mut self,
        sig: Option<Signature>,
        ptr: *const u8,
        len: usize,
        checksum_offset: usize,
    ) {
        self.sig = sig;
        self.patches.clear();
        self.src = ptr as usize;
        self.len = len;
        self.checksum = unsafe { *(ptr.wrapping_add(checksum_offset)) };
    }

    pub unsafe fn copy_to_mem(&self) {
        core::ptr::copy(self.src as *const u8, self.hpa as *mut u8, self.len);

        macro_rules! write_patch {
            ($addr:expr, $val:expr, $ty:ty) => {
                write_unaligned($addr as *mut $ty, $val)
            };
        }

        for (offset, value) in self.patches.iter() {
            let addr = self.hpa + *offset;
            match *value {
                PatchValue::U8(v) => write_patch!(addr, v, u8),
                PatchValue::U16(v) => write_patch!(addr, v, u16),
                PatchValue::U32(v) => write_patch!(addr, v, u32),
                PatchValue::U64(v) => write_patch!(addr, v, u64),
                _ => {}
            }
        }
    }

    pub fn set_addr(&mut self, hpa: usize, gpa: usize) {
        self.hpa = hpa;
        self.gpa = gpa;
        self.is_addr_set = true;
    }

    /// for rsdp, offset = 8; for the others, offset = 9.
    pub fn update_checksum(&mut self, offset: usize) {
        unsafe { *((self.src + offset) as *mut u8) = self.checksum };
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
    /// we need to store rsdp to a safer place
    rsdp_copy: Vec<u8>,
    rsdp: AcpiTable,
    tables: BTreeMap<Signature, AcpiTable>,
    ssdts: BTreeMap<usize, AcpiTable>,
    pointers: Vec<AcpiPointer>,
    config_space_base: usize,
    config_space_size: usize,
    /// key: apic id, value: cpu id (continuous)
    apic_id_to_cpu_id: BTreeMap<usize, usize>,
    /// key: cpu id (continuous), value: apic id
    cpu_id_to_apic_id: BTreeMap<usize, usize>,
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
        table.fill(Some(sig), ptr, len, ACPI_CHECKSUM_OFFSET);
        self.tables.insert(sig, table);
    }

    fn add_ssdt(&mut self, ptr: *const u8, len: usize, rsdt_offset: usize) {
        let mut table = AcpiTable::default();
        table.fill(Some(Signature::SSDT), ptr, len, ACPI_CHECKSUM_OFFSET);
        self.ssdts.insert(rsdt_offset, table);
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
        banned_tables: &BTreeSet<Signature>,
        cpu_set: &CpuSet,
    ) {
        let mut rsdp = self.rsdp.clone();
        let mut tables = self.tables.clone();
        let mut ssdts = self.ssdts.clone();

        // set rsdp addr
        rsdp.set_addr(
            rsdp_zone_region.physical_start as _,
            rsdp_zone_region.virtual_start as _,
        );

        let mut madt_cur: usize = SDT_HEADER_SIZE + 8;
        let mut madt = tables.get_mut(&Signature::MADT).unwrap();

        // fix madt cpu info
        for entry in
            unsafe { Pin::new_unchecked(&*(madt.get_unpatched_src() as *const Madt)) }.entries()
        {
            let mut entry_len = madt.get_u8(madt_cur + 1) as usize;
            match entry {
                MadtEntry::LocalApic(entry) => {
                    let mut disable_lapic = true;
                    if contains_apic_id(entry.apic_id as _) {
                        let cpuid = get_cpu_id(entry.apic_id as _);
                        if cpu_set.contains_cpu(cpuid) {
                            disable_lapic = false;
                        }
                        // reset processor id
                        madt.set_u8(cpuid as _, madt_cur + 2);
                    }
                    if disable_lapic {
                        // set flag to disable lapic
                        madt.set_u32(0x0, madt_cur + 4);
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

            let to_gpa = match banned_tables.contains(&pointer.to_sig) {
                true => 0,
                false => to.gpa,
            };

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

        let ban_ssdt = banned_tables.contains(&Signature::SSDT);
        let from = tables.get_mut(&Signature::RSDT).unwrap();
        for (&offset, ssdt) in ssdts.iter_mut() {
            info!(
                "sig: {:x?}, hpa: {:x?}, gpa: {:x?}, size: {:x?}",
                Signature::SSDT,
                hpa_start + cur,
                gpa_start + cur,
                ssdt.get_len()
            );
            ssdt.set_addr(hpa_start + cur, gpa_start + cur);
            cur += ssdt.get_len();

            let to_gpa = match ban_ssdt {
                true => 0,
                false => ssdt.gpa,
            };
            from.set_u32(to_gpa as _, offset);
        }

        // update checksums
        rsdp.update_checksum(RSDP_CHECKSUM_OFFSET);
        for (sig, table) in tables.iter_mut() {
            table.update_checksum(ACPI_CHECKSUM_OFFSET);
        }

        // copy to memory
        unsafe { rsdp.copy_to_mem() };
        for (sig, table) in tables.iter() {
            // don't copy tables that are not inside ACPI tree
            if tables_involved.contains(sig) {
                unsafe { table.copy_to_mem() };
            }
        }
        if !ban_ssdt {
            for (&offset, ssdt) in ssdts.iter() {
                unsafe { ssdt.copy_to_mem() };
            }
        }
    }

    // let zone 0 bsp cpu does the work
    pub fn init() -> Self {
        let mut root_acpi = Self::default();
        let rsdp_addr = boot::get_multiboot_tags().rsdp_addr.unwrap();

        root_acpi.rsdp_copy = unsafe {
            slice::from_raw_parts(rsdp_addr as *const u8, core::mem::size_of::<Rsdp>()).to_vec()
        };
        let rsdp_copy_addr = root_acpi.rsdp_copy.as_ptr() as usize;

        let handler = HvAcpiHandler {};
        let rsdp_mapping = unsafe {
            handler.map_physical_region::<Rsdp>(rsdp_copy_addr, core::mem::size_of::<Rsdp>())
        };

        // let rsdp_mapping = unsafe { Rsdp::search_for_on_bios(HvAcpiHandler {}).unwrap() };
        // TODO: temporarily suppose we use ACPI 1.0
        assert!(rsdp_mapping.revision() == 0);

        root_acpi.rsdp.fill(
            None,
            rsdp_mapping.virtual_start().as_ptr() as *const u8,
            RSDP_V1_SIZE,
            RSDP_CHECKSUM_OFFSET,
        );
        root_acpi.add_pointer(
            Signature::RSDT,
            RSDP_RSDT_OFFSET,
            Signature::RSDT,
            RSDP_RSDT_PTR_SIZE,
        );

        // get rsdt
        let rsdt_addr = rsdp_mapping.rsdt_address() as usize;
        root_acpi.add_new_table(Signature::RSDT, rsdt_addr as *const u8, SDT_HEADER_SIZE);
        let mut rsdt_offset = root_acpi.get_mut_table(Signature::RSDT).unwrap().get_len();

        let tables =
            unsafe { AcpiTables::from_validated_rsdp(HvAcpiHandler {}, rsdp_mapping) }.unwrap();

        // print rsdt entries
        let mut rsdt_entry = rsdt_addr + 36;
        let size = (unsafe { *((rsdt_addr + 4) as *const u32) } as usize - 36) / 4;
        for i in 0..size {
            let addr = unsafe { *(rsdt_entry as *const u32) } as usize;
            let sig_ptr = addr as *const u8;
            let sig =
                unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(sig_ptr, 4)) };

            println!("sig: {:#x?} ptr: {:x} len: {:x}", sig, addr, unsafe {
                *((addr + 4) as *const u32)
            });
            rsdt_entry += 4;
        }

        // mcfg
        if let Ok(mcfg) = tables.find_table::<Mcfg>() {
            root_acpi.add_new_table(
                Signature::MCFG,
                mcfg.physical_start() as *const u8,
                mcfg.region_length(),
            );

            println!("---------- MCFG ----------");
            let mut offset = size_of::<Mcfg>() + 0xb;

            if let Some(entry) = mcfg
                .entries()
                .iter()
                .find(|&entry| entry.pci_segment_group == 0)
            {
                // we only support segment group 0
                println!("{:x?}", entry);

                let max_bus = ROOT_PCI_MAX_BUS as u8;
                // update bus_number_end
                root_acpi
                    .get_mut_table(Signature::MCFG)
                    .unwrap()
                    .set_u8(max_bus, offset);
                offset += size_of::<McfgEntry>();

                root_acpi.config_space_base = entry.base_address as _;
                root_acpi.config_space_size =
                    (((max_bus as u64 - entry.bus_number_start as u64) + 1) << 20) as usize;
            }

            root_acpi.add_pointer(Signature::RSDT, rsdt_offset, Signature::MCFG, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;
        }

        // fadt
        if let Ok(fadt) = tables.find_table::<Fadt>() {
            root_acpi.add_new_table(
                Signature::FADT,
                fadt.physical_start() as *const u8,
                fadt.region_length(),
            );

            println!("---------- FADT ----------");

            root_acpi.add_pointer(Signature::RSDT, rsdt_offset, Signature::FADT, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;

            // acpi
            let sci_int = fadt.sci_interrupt;
            let smi_port = fadt.smi_cmd_port;
            let acpi_enable = fadt.acpi_enable;
            let acpi_disable = fadt.acpi_disable;
            let pm1a_con = fadt.pm1a_control_block();
            let pm1a_evt = fadt.pm1a_event_block();

            /*println!(
                "sci_interrupt: {:x}, smi_cmd_port: {:x}, acpi_enable: {:x}, acpi_disable: {:x}, pm1a_con: {:#x?}, pm1a_evt: {:#x?}",
                sci_int, smi_port, acpi_enable, acpi_disable, pm1a_con, pm1a_evt,
            );*/
            // println!("{:#x?}", fadt.get());
            // loop {}

            // dsdt
            if let Ok(dsdt) = tables.dsdt() {
                root_acpi.add_new_table(
                    Signature::DSDT,
                    (dsdt.address - SDT_HEADER_SIZE) as *const u8,
                    (dsdt.length as usize + SDT_HEADER_SIZE),
                );
                println!(
                    "sig: \"DSDT\" ptr: {:x}, len: {:x}",
                    dsdt.address, dsdt.length
                );

                root_acpi.add_pointer(Signature::FADT, FADT_DSDT_OFFSET_32, Signature::DSDT, 4);
                root_acpi.add_pointer(Signature::FADT, FADT_DSDT_OFFSET_64, Signature::DSDT, 8);
            }

            // facs
            if let Ok(facs_addr) = fadt.facs_address() {
                let len = unsafe { *((facs_addr + 4) as *const u32) as usize };
                root_acpi.add_new_table(Signature::FACS, facs_addr as *const u8, len);
                println!("sig: \"FACS\" ptr: {:x}, len: {:x}", facs_addr, len);

                root_acpi.add_pointer(Signature::FADT, FADT_FACS_OFFSET_32, Signature::FACS, 4);
                root_acpi.add_pointer(Signature::FADT, FADT_FACS_OFFSET_64, Signature::FACS, 8);
            }
        }

        // madt
        if let Ok(madt) = tables.find_table::<Madt>() {
            root_acpi.add_new_table(
                Signature::MADT,
                madt.physical_start() as *const u8,
                madt.region_length(),
            );

            println!("---------- MADT ----------");
            for entry in madt.get().entries() {
                match entry {
                    MadtEntry::LocalApic(entry) => {
                        if entry.flags != 0 {
                            println!("{:x?}", entry);
                            let cpu_id = root_acpi.apic_id_to_cpu_id.len();
                            root_acpi
                                .apic_id_to_cpu_id
                                .insert(entry.apic_id as _, cpu_id);
                            root_acpi
                                .cpu_id_to_apic_id
                                .insert(cpu_id, entry.apic_id as _);
                        }
                    }
                    _ => {}
                }
            }

            root_acpi.add_pointer(Signature::RSDT, rsdt_offset, Signature::MADT, RSDT_PTR_SIZE);
            rsdt_offset += RSDT_PTR_SIZE;
        }

        // dmar
        acpi_table!(Dmar, DMAR);
        if let Ok(dmar) = tables.find_table::<Dmar>() {
            root_acpi.add_new_table(
                Signature::DMAR,
                dmar.physical_start() as *const u8,
                dmar.region_length(),
            );

            /*println!("DMAR: {:x?}", unsafe {
                *((dmar.physical_start() + 56) as *const [u8; 8])
            });*/

            // self.add_pointer(Signature::RSDT, rsdt_offset, Signature::DMAR, RSDT_PTR_SIZE);
            // rsdt_offset += RSDT_PTR_SIZE;
        }

        // ssdt
        for ssdt in tables.ssdts() {
            root_acpi.add_ssdt(
                (ssdt.address - SDT_HEADER_SIZE) as *const u8,
                (ssdt.length as usize + SDT_HEADER_SIZE),
                rsdt_offset,
            );
            rsdt_offset += RSDT_PTR_SIZE;
        }

        if let Some(rsdt) = root_acpi.get_mut_table(Signature::RSDT) {
            rsdt.set_new_len(rsdt_offset);
        }
        root_acpi
    }
}

// let zone 0 bsp cpu does the work
pub fn root_init() {
    ROOT_ACPI.call_once(|| RootAcpi::init());
}

pub fn copy_to_guest_memory_region(config: &HvZoneConfig, cpu_set: &CpuSet) {
    let mut banned: BTreeSet<Signature> = BTreeSet::new();
    if config.zone_id != 0 {
        banned.insert(Signature::FADT);
        banned.insert(Signature::SSDT);
    }
    ROOT_ACPI.get().unwrap().copy_to_zone_region(
        &config.memory_regions()[config.arch_config.rsdp_memory_region_id],
        &config.memory_regions()[config.arch_config.acpi_memory_region_id],
        &banned,
        cpu_set,
    );
}

pub fn root_get_table(sig: &Signature) -> Option<AcpiTable> {
    ROOT_ACPI.get().unwrap().get_table(sig)
}

pub fn root_get_config_space_info() -> Option<(usize, usize)> {
    let acpi = ROOT_ACPI.get().unwrap();
    Some((acpi.config_space_base, acpi.config_space_size))
}

fn contains_apic_id(apic_id: usize) -> bool {
    ROOT_ACPI
        .get()
        .unwrap()
        .apic_id_to_cpu_id
        .contains_key(&apic_id)
}

pub fn get_cpu_id(apic_id: usize) -> usize {
    *ROOT_ACPI
        .get()
        .unwrap()
        .apic_id_to_cpu_id
        .get(&apic_id)
        .unwrap()
}

pub fn get_apic_id(cpu_id: usize) -> usize {
    *ROOT_ACPI
        .get()
        .unwrap()
        .cpu_id_to_apic_id
        .get(&cpu_id)
        .unwrap()
}
