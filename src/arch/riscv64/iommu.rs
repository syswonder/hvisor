use log::{error, info};
use spin::{Once, RwLock};
use crate::memory::Frame;

const IOMMU_MODE: usize = 0x2;
pub const BLK_PCI_ID: usize = 0x4;
pub const PCIE_MMIO_BEG: usize = 0x4000_0000;
pub const PCIE_MMIO_SIZE: usize = 0x0000_0000;
pub const PCI_MAP_BEG: usize = 0x4_0000_0000;
pub const PCI_MAP_SIZE: usize = 0x4_0000_0000;

// # Memory-mapped Register Interface
//  Capabilities register fields
const RV_IOMMU_CAPS_VERSION_MASK: usize = 0xff;
const RV_IOMMU_CAPS_IGS_MASK: usize = 0x3 << 28;
const RV_IOMMU_SUPPORTED_VERSION: usize = 0x10;
const RV_IOMMU_CAPS_SV39X4_BIT: usize = 0x1 << 17;
const RV_IOMMU_CAPS_MSI_FLAT_BIT: usize = 0x1 << 17;
const RV_IOMMU_CAPS_AMO_HWAD_BIT: usize = 0x1 << 24;

// Features control register fields
const RV_IOMMU_FCTL_DEFAULT: u32 = 0x1 << 1;

// Interrupt pending register
// const RV_IOMMU_IPSR_FIP_BIT: u32 = 1 << 1;
const RV_IOMMU_IPSR_CLEAR: u32 = 0x0F;

// Interrupt Vectors
// const RV_IOMMU_ICVEC_DEFAULT: usize = 0x1 << 4;

// Confiure DC
const RV_IOMMU_DC_VALID_BIT: usize = 1 << 0;
const RV_IOMMU_IOHGATP_SV39X4: usize = 8 << 60;
const RV_IOMMU_DC_IOHGATP_PPN_MASK: usize = 0xFFF_FFFF_FFFF;
const RV_IOMMU_DC_IOHGATP_GSCID_MASK: usize = 0xFFFF << 44;
const RV_IOMMU_DDTP_PPN_MASK: usize = 0xFFF_FFFF_FFFF << 10;

pub static IOMMU: Once<RwLock<Iommu>> = Once::new();    // 允许只初始化一次

pub fn iommu<'a>() -> &'a RwLock<Iommu> {
    IOMMU.get().expect("Uninitialized hypervisor iommu!")
}

// alloc the Fram for DDT
pub fn iommu_init() {
    let iommu = Iommu::new(0x10010000);
    IOMMU.call_once(|| RwLock::new(iommu));
    rv_iommu_init();
}

// master CPU do!
pub fn rv_iommu_init(){
    let iommu = iommu();
    let _ = iommu.write().rv_iommu_init();
}

// every DMA device do!
pub fn iommu_add_device(vm_id: usize, device_id: usize, root_pt: usize){
    info!("RV_IOMMU_ADD_DEVICE: root_pt {:#x}, vm_id {}", root_pt, vm_id);
    let iommu = iommu();
    iommu.write().rv_iommu_add_device(device_id, vm_id, root_pt);
}

#[repr(C)]
#[repr(align(0x1000))]
pub struct IommuHw {
    caps: u64,
    fctl: u32,
    __custom1: [u8; 4],
    ddtp: u64,                  // 设备目录表指针
    cqb: u64,                   // 命令队列基地址
    cqh: u32,                   // 命令队列队首
    cqt: u32,                   // 命令队列队尾
    fqb: u64,                   // 故障队列
    fqh: u32,
    fqt: u32,
    pqb: u64,                   // 页面请求队列, ATS
    pqh: u32,
    pqt: u32,
    cqcsr: u32,                 // 命令队列 CSR
    fqcsr: u32,
    pqcsr: u32,
    ipsr: u32,                  // 中断待处理寄存器
    iocntovf: u32,              //      性能相关, HPM
    iocntinh: u32,
    iohpmcycles: u64,
    iohpmctr: [u64; 31],
    iohpmevt: [u64; 31],
    tr_req_iova: u64,           //      debug 相关, DBG
    tr_req_ctl: u64,
    tr_response: u64,
    __rsv1: [u8; 64],
    __custom2: [u8; 72],
    icvec: u64,                 // 矢量寄存器的中断原因
    msi_cfg_tbl: [MsiCfgTbl; 16],
    __rsv2: [u8;3072],
}

// #define BIT64_MASK(OFF, LEN) ((((UINT64_C(1) << ((LEN)-1)) << 1) - 1) << (OFF))

impl IommuHw {
    pub fn rv_iommu_check_features(&self){
        let caps = self.caps as usize;
        let version = caps & RV_IOMMU_CAPS_VERSION_MASK;
        // 获取版本, 1.0 规范对应 0x10
        if version != RV_IOMMU_SUPPORTED_VERSION{
            error!("RISC-V IOMMU unsupported version: {}", version);
        }
        // 支持 SV39x4 = 41 位虚拟地址, 用于第二阶段地址转换
        if caps & RV_IOMMU_CAPS_SV39X4_BIT == 0 {
            error!("RISC-V IOMMU HW does not support Sv39x4");
        }
        // 使用 MSI FLAT
        if caps & RV_IOMMU_CAPS_MSI_FLAT_BIT == 0 {
            error!("RISC-V IOMMU HW does not support MSI Address Translation (basic-translate mode)");
        }
        // 支持 IOMMU 中断生成, 0 - MSI, 1 - WSI, 2 - BOTH, 这里要支持有线信号中断生成
        if caps & RV_IOMMU_CAPS_IGS_MASK == 0 {
            error!("RISC-V IOMMU HW does not support WSI generation");
        }
        if caps & RV_IOMMU_CAPS_AMO_HWAD_BIT == 0 {
            error!("RISC-V IOMMU HW AMO HWAD unsupport");
        }
    }
    
    pub fn rv_iommu_init(&mut self, ddt_addr: usize){
        // Read and check caps
        self.rv_iommu_check_features();
        // Set fctl.WSI We will be first using WSI as IOMMU interrupt mechanism
        self.fctl = RV_IOMMU_FCTL_DEFAULT;
        // Clear all IP flags (ipsr)
        self.ipsr = RV_IOMMU_IPSR_CLEAR;
        // Configure ddtp with DDT base address and IOMMU mode
        self.ddtp = IOMMU_MODE as u64 | ((ddt_addr >> 2) & RV_IOMMU_DDTP_PPN_MASK) as u64;
        // self.ddtp = PLATFORM.iommu_mode as u64 | ddt_addr as u64;
        // error!{"RV_IOMMU_INIT: DDTP mode {:#x}, DDT_ADDR {:#x}", PLATFORM.iommu_mode, ddt_addr};
    }
}

#[repr(C)]
struct MsiCfgTbl{
    addr: u64,
    data: u32,
    vctl: u32,
}

#[repr(C)]
struct DdtEntry{
    tc: u64,
    iohgatp: u64,                  // iohgapt
    ta: u64,
    fsc: u64,                       // fsc
    msiptp: u64,                    // msi 页表
    msi_addr_mask: u64,
    msi_addr_pattern: u64,
    __rsv: u64,
}

#[repr(C)]
pub struct Lvl1DdtHw{
    dc: [DdtEntry; 64],
}

pub struct Iommu{
    pub base: usize,
    pub ddt: Frame,                 // 当前仅支持单级设备目录表, 即上述的Lvl1DdtHw
}

impl Iommu {
    pub fn new(base: usize) -> Self{
        Self { 
            base: base,
            ddt: Frame::new_zero().unwrap(),
        }
    }

    pub fn iommu(&self) -> &mut IommuHw{
        unsafe { &mut *(self.base as *mut _) }
    }

    pub fn dc(&self) -> &mut Lvl1DdtHw{
        unsafe { &mut *(self.ddt.start_paddr() as *mut _)}
    }

    pub fn rv_iommu_init(&mut self){
        // 检查 iommu 的版本等, 配置 ddtp
        self.iommu().rv_iommu_init(self.ddt.start_paddr());
    }

    pub fn rv_iommu_add_device(&self, device_id: usize, vm_id: usize, root_pt: usize){
        if device_id > 0 && device_id < 64{
            // configure DC
            let tc: u64 = 0 | RV_IOMMU_DC_VALID_BIT as u64 | 1 << 4;
            self.dc().dc[device_id].tc = tc;
            let mut iohgatp: u64 = 0;
            iohgatp |= (root_pt as u64 >> 12) & RV_IOMMU_DC_IOHGATP_PPN_MASK as u64;
            iohgatp |= (vm_id as u64) & RV_IOMMU_DC_IOHGATP_GSCID_MASK as u64;
            iohgatp |= RV_IOMMU_IOHGATP_SV39X4 as u64;
            self.dc().dc[device_id].iohgatp = iohgatp;
            self.dc().dc[device_id].fsc = 0;
            info!("{:#x}", &mut self.dc().dc[device_id] as *mut _ as usize);
            info!("RV IOMMU: Write DDT, add decive context, iohgatp {:#x}", iohgatp);
        }
        else{
            info!("RV IOMMU: Invalid device ID: {}", device_id);
        }
    }
}