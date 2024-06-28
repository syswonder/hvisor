use alloc::vec::Vec;
use spin::mutex::Mutex;
use aarch64_cpu::registers::{Readable, Writeable};
use tock_registers::{register_structs, registers::{ReadOnly, ReadWrite}};
use crate::memory::{Frame, VirtAddr};


const SMMU_BASE_ADDR:VirtAddr = 0x09050000;

const CR0_SMMUEN:usize = 1;
// const CR0_CMDQEN:usize = 1 << 3;
// const CR0_EVTQEN:usize = 1 << 2;

const CR1_TABLE_SH_OFF:usize = 10;
const CR1_TABLE_OC_OFF:usize = 8;
const CR1_TABLE_IC_OFF:usize = 6;
const CR1_QUEUE_SH_OFF:usize = 4;
const CR1_QUEUE_OC_OFF:usize = 2;
const CR1_QUEUE_IC_OFF:usize = 0;
const ARM_SMMU_SH_ISH:usize = 3;

// const CR1_CACHE_NC:usize = 0;
const CR1_CACHE_WB:usize = 1;
// const CR1_CACHE_WT:usize = 2;

const IDR0_S2P_BIT:usize = 1;
const IDR0_S1P_BIT:usize = 1<<1;
const IDR0_ST_LEVEL_OFF:usize = 27;
const IDR0_ST_LEVEL_LEN:usize = 2;
const IDR0_VMID16_BIT:usize = 1<<18;

const IDR1_SIDSIZE_OFF:usize = 0;
const IDR1_SIDSIZE_LEN:usize = 6;
const IDR1_CMDQS_OFF:usize = 21;
const IDR1_CMDQS_LEN:usize = 5;
const IDR1_EVTQS_OFF:usize = 16;
const IDR1_EVTQS_LEN:usize = 5;
const CMDQ_MAX_SZ_SHIFT:usize = 8;
const EVTQ_MAX_SZ_SHIFT:usize = 7;

// const IDR1_TABLES_PRESET:usize = 30;
// const IDR1_QUEUES_PRESET:usize = 29;
// const IDR1_REL:usize = 28;

const STRTAB_BASE_OFF:usize = 6;
const STRTAB_BASE_LEN:usize = 46;
const STRTAB_BASE_RA:usize = 1 << 62;

const STRTAB_STE_DWORDS_BITS:usize = 3;
const STRTAB_STE_DWORDS:usize = 1 << STRTAB_STE_DWORDS_BITS;
const STRTAB_STE_SIZE:usize = STRTAB_STE_DWORDS << 3;
const STRTAB_STE_0_V:usize = 1;
const STRTAB_STE_0_CFG_OFF:usize = 1;
// const STRTAB_STE_0_CFG_ABORT:usize = 0;
const STRTAB_STE_0_CFG_BYPASS:usize = 4;
// const STRTAB_STE_0_CFG_S1_TRANS:usize = 5;
const STRTAB_STE_0_CFG_S2_TRANS:usize = 6;
// const STRTAB_STE_0_S1CTXPTR_OFF:usize = 6;
// const STRTAB_STE_0_S1CTXPTR_LEN:usize = 46;
// const STRTAB_STE_0_S1CDMAX_OFF:usize = 59;
// const STRTAB_STE_0_S1CDMAX_LEN:usize = 5;
// const STRTAB_STE_1_S1DSS_OFF:usize = 0;
// const STRTAB_STE_1_S1CIR_OFF:usize = 2;
// const STRTAB_STE_1_S1COR_OFF:usize = 4;
// const STRTAB_STE_1_S1CSH_OFF:usize = 6;
// const STRTAB_STE_1_S1STALLD:usize = 1 << 27;
// const STRTAB_CTXDESC_DWORDS:usize = 8;
// const STRTAB_CTXDESC_S1CTXPTR_SHIFT:usize = 6;
const STRTAB_STE_1_SHCFG_OFF:usize = 44;
const STRTAB_STE_1_SHCFG_INCOMING:usize = 1;
const STRTAB_STE_2_VTCR_OFF:usize = 32;
const STRTAB_STE_2_VTCR_LEN:usize = 19;
const STRTAB_STE_2_S2VMID_OFF:usize = 0;
const STRTAB_STE_2_S2PTW:usize = 1 << 54;
const STRTAB_STE_2_S2AA64:usize = 1 << 51;
const STRTAB_STE_2_S2R:usize = 1 << 58;

const STRTAB_STE_3_S2TTB_OFF:usize = 4;
const STRTAB_STE_3_S2TTB_LEN:usize = 48;

const STRTAB_BASE_CFG_FMT_OFF:usize = 16;
// const STRTAB_BASE_CFG_FMT_LEN:usize = 2;
const STRTAB_BASE_CFG_FMT_LINEAR:usize = 0 << 16;
// const STRTAB_BASE_CFG_FMT_2LVL:usize = 1 << 16;
// const STRTAB_BASE_CFG_SPLIT_OFF:usize = 6;
// const STRTAB_BASE_CFG_SPLIT_LEN:usize = 5;
const STRTAB_BASE_CFG_LOG2SIZE_OFF:usize = 0;
// const STRTAB_BASE_CFG_LOG2SIZE_LEN:usize = 6;

const Q_BASE_RWA:usize = 1 << 62;
const Q_BASE_ADDR_OFF:usize = 5;
const Q_BASE_ADDR_LEN:usize = 47;
const Q_BASE_LOG2SIZE_OFF:usize = 0;


// page0 + page1
register_structs!{
    #[allow(non_snake_case)]
    pub RegisterPage{
        (0x0000 => IDR0:ReadOnly<u32>),
        (0x0004 => IDR1:ReadOnly<u32>),
        (0x0008 => IDR2:ReadOnly<u32>),
        (0x000c => IDR3:ReadOnly<u32>),
        (0x0010 => IDR4:ReadOnly<u32>),
        (0x0014 => IDR5:ReadOnly<u32>),
        (0x0018 => IIDR:ReadOnly<u32>),
        (0x001c => AIDR:ReadOnly<u32>),
        (0x0020 => CR0:ReadWrite<u32>),
        (0x0024 => CR0ACK:ReadOnly<u32>),
        (0x0028 => CR1:ReadWrite<u32>),
        (0x002c => CR2:ReadWrite<u32>),
        (0x0030 => _reserved0), 
        (0x0050 => IRQ_CTRL:ReadWrite<u32>),
        (0x0054 => IRQ_CTRLACK:ReadOnly<u32>),
        (0x0058 => _reserved1),
        (0x0060 => GERROR:ReadOnly<u32>),
        (0x0064 => GERRORN:ReadWrite<u32>),
        (0x0068 => GERROR_IRQ_CFG0:ReadWrite<u64>),
        (0x0070 => _reserved2),
        (0x0080 => STRTAB_BASE:ReadWrite<u64>),
        (0x0088 => STRTAB_BASE_CFG:ReadWrite<u32>),
        (0x008c => _reserved3),
        (0x0090 => CMDQ_BASE:ReadWrite<u64>),
        (0x0098 => CMDQ_PROD:ReadWrite<u32>),
        (0x009c => CMDQ_CONS:ReadWrite<u32>),
        (0x00a0 => EVENTQ_BASE:ReadWrite<u64>),
        (0x00a8 => _reserved4),
        (0x00b0 => EVENTQ_IRQ_CFG0:ReadWrite<u64>),
        (0x00b8 => EVENTQ_IRQ_CFG1:ReadWrite<u32>),
        (0x00bc => EVENTQ_IRQ_CFG2:ReadWrite<u32>),
        (0x00c0 => _reserved5),
        (0x100a8 => EVENTQ_PROD:ReadWrite<u32>),
        (0x100ac => EVENTQ_CONS:ReadWrite<u32>),
        (0x100b0 => _reserved6),
        (0x20000 => @END),
    }
}

unsafe impl Sync for RegisterPage{}

pub fn extract_bits(value:usize,start:usize,length:usize) -> usize{
    let mask = (1 << length) -1;
    (value >> start) & mask
}

pub fn min(a:usize,b:usize) -> usize{
    if a<b{
        a
    }else{
        b
    }
}


pub struct Smmuv3{
    rp:&'static RegisterPage,

    strtab_2lvl:bool,
    sid_max_bits:usize,

    frames:Vec<Frame>,

    // strtab
    strtab_base:usize,

    // cmdq
    cmdq_base:usize,
    cmdq_max_n_shift:usize,

    // evtq
    evtq_base:usize,
    evtq_max_n_shift:usize,
}


impl Smmuv3{
    fn new() -> Self{
        let rp = unsafe {
            &*(SMMU_BASE_ADDR as *const RegisterPage)
        };

        let mut r = Self{
            rp:rp,
            strtab_2lvl:false,
            sid_max_bits:0,
            frames:vec![],
            strtab_base:0,
            cmdq_base:0,
            cmdq_max_n_shift:0,
            evtq_base:0,
            evtq_max_n_shift:0,
        };

        r.check_env();

        r.init_structures();

        r.device_reset();

        r
    }

    fn check_env(&mut self){
        let idr0 = self.rp.IDR0.get() as usize;

        info!("Smmuv3 IDR0:{:b}",idr0);

        // supported types of stream tables.
        let stb_support = extract_bits(idr0, IDR0_ST_LEVEL_OFF, IDR0_ST_LEVEL_LEN);
        match stb_support{
            0 => info!("Smmuv3 Linear Stream Table Supported."),
            1 => {info!("Smmuv3 2-level Stream Table Supoorted.");
                self.strtab_2lvl = true;
            }
            _ => info!("Smmuv3 don't support any stream table."),
        }

        // supported address translation stages.
        let s1p_support = idr0 & IDR0_S1P_BIT;
        match s1p_support{
            0 => info!("Smmuv3 Stage-1 translation not supported."),
            _ => info!("Smmuv3 Stage-1 translation supported."),
        }

        let s2p_support = idr0 & IDR0_S2P_BIT;
        match s2p_support{
            0 => info!("Smmuv3 Stage-2 translation not supported."),
            _ => info!("Smmuv3 Stage-2 translation supported."),
        }

        // 16-bit VMID supported.
        match idr0 & IDR0_VMID16_BIT{
            0 => info!("Smmuv3 16-bit VMID not supported."),
            _ => info!("Smmuv3 16-bit VMID supported.")
        }

        let idr1 = self.rp.IDR1.get() as usize;

        // if idr1 & (IDR1_TABLES_PRESET | IDR1_QUEUES_PRESET | IDR1_REL) !=0{
        //     error!("[Smmuv3] unknown error!");
        // }

        // cmdq and evtq
        self.cmdq_max_n_shift = min(CMDQ_MAX_SZ_SHIFT,extract_bits(idr1, IDR1_CMDQS_OFF, IDR1_CMDQS_LEN));
        self.evtq_max_n_shift = min(EVTQ_MAX_SZ_SHIFT,extract_bits(idr1, IDR1_EVTQS_OFF, IDR1_EVTQS_LEN));

        // max bits of stream_id
        let sid_max_bits = extract_bits(idr1, IDR1_SIDSIZE_OFF, IDR1_SIDSIZE_LEN);
        info!("Smmuv3 SID_MAX_BITS:{:?}",sid_max_bits);

        self.sid_max_bits = sid_max_bits;

        // sid_max_bis>=7,must allow the use of secondary tables.
        if sid_max_bits>=7 && extract_bits(idr0, IDR0_ST_LEVEL_OFF, IDR0_ST_LEVEL_LEN)==0{
            error!("Smmuv3 the system must support for 2-level table");
        }

        // must use linear table
        if sid_max_bits <= 8{
            info!("Smmuv3 must use linear stream table!");
        }
    }

    fn init_structures(&mut self){
        // self.init_queues();

        self.init_strtab();
    }

    #[allow(unused)]
    fn init_queues(&mut self){
        self.init_cmdq();

        self.init_evtq();
    }

    fn init_cmdq(&mut self){
        if let Ok(frame) = Frame::new(){
            self.cmdq_base=frame.start_paddr();
            self.frames.push(frame);
        }

        let mut base = extract_bits(self.cmdq_base, Q_BASE_ADDR_OFF, Q_BASE_ADDR_LEN);
        base = base << Q_BASE_ADDR_OFF;
        base |= Q_BASE_RWA;
        base |= self.cmdq_max_n_shift << Q_BASE_LOG2SIZE_OFF;

        self.rp.CMDQ_BASE.set(base as _);
        self.rp.CMDQ_PROD.set(0);
        self.rp.CMDQ_CONS.set(0);
    }

    fn init_evtq(&mut self){
        if let Ok(frame) = Frame::new(){
            self.evtq_base=frame.start_paddr();
            self.frames.push(frame);
        }

        let mut base = extract_bits(self.evtq_base, Q_BASE_ADDR_OFF, Q_BASE_ADDR_LEN);
        base = base << Q_BASE_ADDR_OFF;
        base |= Q_BASE_RWA;
        base |= self.evtq_max_n_shift << Q_BASE_LOG2SIZE_OFF;

        self.rp.EVENTQ_BASE.set(base as _);
        self.rp.EVENTQ_PROD.set(0);
        self.rp.EVENTQ_CONS.set(0);
    }

    fn init_strtab(&mut self){
        // linear stream table is our priority
        self.init_linear_strtab();
    }

    // strtab
    fn init_linear_strtab(&mut self){
        info!("Smmuv3 init linear stream table");

        // The lower (5+self.sid_max_bits) bits must be 0.
        // let tab_size = (1 << self.sid_max_bits) * STRTAB_STE_SIZE;
        // let frame_count = tab_size / PAGE_SIZE;
        if let Ok(frame) = Frame::new_contiguous(100, 0){
            self.strtab_base = frame.start_paddr();
            self.frames.push(frame);
        }

        self.strtab_base = 0x40800000;

        info!("strtab_base:0x{:x}",self.strtab_base);
        

        let mut base = extract_bits(self.strtab_base, STRTAB_BASE_OFF, STRTAB_BASE_LEN);
        base = base << STRTAB_BASE_OFF;
        base |= STRTAB_BASE_RA;
        self.rp.STRTAB_BASE.set(base as _);

        // strtab_base_cfg
        let mut cfg:usize = 0;
        // format : linear table
        cfg |= STRTAB_BASE_CFG_FMT_LINEAR << STRTAB_BASE_CFG_FMT_OFF;

        // table size as log2(entries)
        // entry_num = 2^(sid_bits)
        // log2(size) = sid_bits
        cfg |= self.sid_max_bits << STRTAB_BASE_CFG_LOG2SIZE_OFF;

        // linear table -> ignore SPLIT field
        self.rp.STRTAB_BASE_CFG.set(cfg as _);

        // init strtab entries
        self.init_bypass_stes();
    }

    fn init_bypass_stes(&mut self){
        let entry_num:usize = 1 << self.sid_max_bits;

        for sid in 0..entry_num{
            self.init_bypass_ste(sid);
        }
    }

    // init bypass ste
    fn init_bypass_ste(&mut self,sid:usize){
        let base = self.strtab_base + sid * STRTAB_STE_SIZE;
        let tab = unsafe{&mut *(base as *mut [u64;STRTAB_STE_DWORDS])};

        let mut val:usize = 0;

        val |= STRTAB_STE_0_V;
        val |= STRTAB_STE_0_CFG_BYPASS << STRTAB_STE_0_CFG_OFF;

        tab[0] = val as _;
        tab[1] = (STRTAB_STE_1_SHCFG_INCOMING << STRTAB_STE_1_SHCFG_OFF) as _;
    }

    fn device_reset(&mut self){
        /* CR1 (table and queue memory attributes) */
        let mut reg = ARM_SMMU_SH_ISH << CR1_TABLE_SH_OFF;
        reg |= CR1_CACHE_WB << CR1_TABLE_OC_OFF;
        reg |= CR1_CACHE_WB << CR1_TABLE_IC_OFF;
        reg |= ARM_SMMU_SH_ISH << CR1_QUEUE_SH_OFF;
        reg |= CR1_CACHE_WB << CR1_QUEUE_OC_OFF;
        reg |= CR1_CACHE_WB << CR1_QUEUE_IC_OFF;
        self.rp.CR1.set(reg as _);
        
        let cr0 = CR0_SMMUEN;
        // cr0 |= CR0_CMDQEN;
        // cr0 |= CR0_EVTQEN;
        self.rp.CR0.set(cr0 as _);
    }

    // s1 bypass and s2 tranlate
    fn write_ste(&mut self,sid:usize,vmid:usize,root_pt:usize){
        let base = self.strtab_base + sid * STRTAB_STE_SIZE;
        info!("write base:0x{:x}",base);
        let tab = unsafe{&mut *(base as *mut [u64;STRTAB_STE_DWORDS])};

        let mut val0:usize = 0;
        val0 |= STRTAB_STE_0_V;
        val0 |= STRTAB_STE_0_CFG_S2_TRANS << STRTAB_STE_0_CFG_OFF;

        let mut val2:usize = 0;
        val2 |= vmid << STRTAB_STE_2_S2VMID_OFF;
        val2 |= STRTAB_STE_2_S2PTW;
        val2 |= STRTAB_STE_2_S2AA64;
        val2 |= STRTAB_STE_2_S2R;

        let vtcr = 20 + (2<<6) + (1<<8) + (1<<10) + (3<<12) + (0<<14) + (4<<16);
        let v = extract_bits(vtcr as _, 0, STRTAB_STE_2_VTCR_LEN);
        val2 |= v << STRTAB_STE_2_VTCR_OFF;

        let vttbr = extract_bits(root_pt, STRTAB_STE_3_S2TTB_OFF, STRTAB_STE_3_S2TTB_LEN);

        tab[0] |= val0 as u64;
        tab[2] |= val2 as u64;
        tab[3] |= (vttbr << STRTAB_STE_3_S2TTB_OFF) as u64;
    }

    
}

static SMMUV3: spin::Once<Mutex<Smmuv3>> = spin::Once::new();

/// smmuv3 init
pub fn iommu_init(){
    info!("Smmuv3 init...");
    SMMUV3.call_once(|| Mutex::new(Smmuv3::new()));
}

/// for hvisor page_table
pub fn smmuv3_base() -> usize{
    SMMU_BASE_ADDR.into()
}

/// smmuv3_size
/// RegisterPage0 + RegisterPage1
pub fn smmuv3_size() -> usize{
    0x20000
}

/// write ste
/// how to get sid?
/// qemu-args -> pci-device -> property:addr -> bdf -> sid = (b<<5) | (d<<3) | f
/// e.g: virtio-blk-pci -> addr=2.0 -> sid=0x10
/// how to varify the sid?
/// qemu_args += -trace smmuv3_*
/// then you can see the output like: smmuv3_translate_success smmuv3-iommu-memory-region-16-2 sid=0x10 iova=0x8e041242 translated=0x8e041242 perm=0x3
pub fn iommu_add_device(vmid:usize, sid:usize, root_pt:usize){
    let mut smmu = SMMUV3.get().unwrap().lock();
    smmu.write_ste(sid as _, vmid as _, root_pt);
    info!("wirte ste: vmid=0x{:x},sid=0x{:x},root_pt:0x{:x}",vmid,sid,root_pt);
}
