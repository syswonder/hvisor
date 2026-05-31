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
//      Guowei Li <2401213322@stu.pku.edu.cn>
//      Jingyu Liu <liujingyu24s@ict.ac.cn>
//
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_mut)]
#![deny(unused)]

#[cfg(not(target_arch = "loongarch64"))]
use crate::{
    arch::cpu::get_target_cpu,
    event::{send_event, IPI_EVENT_WAKEUP_VIRTIO_DEVICE},
    hypercall::SGI_IPI_ID,
};
use crate::{
    arch::cpu::this_cpu_id, consts::MAX_WAIT_TIMES, device::irqchip::inject_irq, error::HvResult,
    memory::MMIOAccess, zone::this_zone_id,
};
use alloc::collections::BTreeMap;
use core::{
    fmt::{Debug, Formatter, Result},
    mem::size_of,
    sync::atomic::{fence, AtomicBool, AtomicUsize, Ordering},
};
use spin::{Mutex, MutexGuard};
use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::ReadWrite,
};

/// Save the irqs the virtio-device wants to inject. The format is <cpu_id, List<irq_id>>, and the first elem of List<irq_id> is the valid len of it.
pub static VIRTIO_IRQS: Mutex<BTreeMap<usize, [u64; MAX_DEVS + 1]>> = Mutex::new(BTreeMap::new());
// Controller of the shared memory the root linux's virtio device and hvisor shares.
pub static VIRTIO_PCI_BRIDGE: Mutex<VirtioPCIBridge> = Mutex::new(VirtioPCIBridge::dummy());
pub static VIRTIO_BRIDGE: VirtioBridgeController = VirtioBridgeController::new();

const QUEUE_NOTIFY: usize = 0x50;
pub const MAX_REQ: u32 = 32;
pub const MAX_DEVS: usize = 8; // Attention: The max virtio-dev number for vm is 8 (loongarch64 needs 3 consoles and 3 disks for zgclab project).
pub const MAX_CPUS: usize = 32;
pub const MAX_VQ: usize = 16;
const MAX_PCI_CONFIG_REQ: usize = 4;
const MAX_PCI_CONFIG_RES: usize = 4;
const MAX_PCI_DATA_REQ: usize = 32;
pub const VIRTIO_PCI_HYPERCALL_VERSION: u16 = 1;

pub const MAX_BACKOFF: usize = 1024;

#[cfg(not(target_arch = "loongarch64"))]
use crate::platform::IRQ_WAKEUP_VIRTIO_DEVICE;

/// non root zone's virtio request handler
pub fn mmio_virtio_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    // debug!("mmio virtio handler");
    let cpu_id = this_cpu_id() as usize;
    let need_interrupt = if mmio.address == QUEUE_NOTIFY { 1 } else { 0 };
    if need_interrupt == 1 {
        trace!("notify !!!, cpu id is {}", cpu_id);
    }
    mmio.address += base;
    // Ensure read old_cfg_flag before push_req
    let old_cfg_flag = VIRTIO_BRIDGE.cfg_flag(cpu_id);
    fence(Ordering::Acquire);

    // Try to push req to req_list (in VirtioBridge critical area)
    // To avoid concurrent access to req_list, hvisor should lock VIRTIO_BRIDGE's req_list related part (here use req_agent)
    let mut backoff = 1;
    let mut req_agent = VIRTIO_BRIDGE.req_agent();
    while req_agent.is_full() {
        // When root linux's cpu is in el2's finish req handler and is getting the dev lock,
        // if we don't release dev lock, it will cause a dead lock.
        drop(req_agent);
        // Exponential Backoff Algorithm, here especially useful for big.LITTLE architecture.
        for _ in 0..backoff {
            core::hint::spin_loop();
        }
        backoff <<= 1;
        backoff = backoff.min(MAX_BACKOFF);
        req_agent = VIRTIO_BRIDGE.req_agent();
    }
    let hreq = HvisorDeviceReq::new(
        cpu_id as _,
        mmio.address as _,
        mmio.size as _,
        mmio.value as _,
        this_zone_id() as _,
        mmio.is_write,
        need_interrupt,
    );
    // debug!("non root sends req: {:#x?}", hreq);
    req_agent.push_req(hreq);
    drop(req_agent);

    #[cfg(not(target_arch = "loongarch64"))]
    let mut is_ipi_sent = false;
    // If backend is sleep, hvisor needs to send ipi to wake it up.
    #[cfg(not(target_arch = "loongarch64"))]
    check_need_wakeup_and_send_ipi(&mut is_ipi_sent);

    let mut count: usize = 0;
    // if it is cfg request, current cpu should be blocked until gets the result
    if need_interrupt == 0 {
        // when virtio backend finish the req, it will add 1 to cfg_flags[cpu_id].
        while !VIRTIO_BRIDGE.is_cfg_updated(cpu_id, old_cfg_flag) {
            count += 1;
            if count == MAX_WAIT_TIMES {
                warn!(
                    "virtio backend is too slow, please check it! addr: {:x} is_write: {:x?}",
                    mmio.address, mmio.is_write
                );
            }
            if count == MAX_WAIT_TIMES * 10 {
                error!(
                    "virtio backend may have some problem, please check it! addr: {:x} is_write: {:x?}",
                    mmio.address, mmio.is_write
                );
                count = 0;
            }
            // check_need_wakeup_and_send_ipi(&mut is_ipi_sent);
        }
        if !mmio.is_write {
            // ensure cfg value is right.
            mmio.value = VIRTIO_BRIDGE.cfg_value(cpu_id) as _;
            // debug!("non root receives value: {:#x?}", mmio.value);
        }
    }
    // debug!("non root returns");
    Ok(())
}

#[cfg(not(target_arch = "loongarch64"))]
pub fn check_need_wakeup_and_send_ipi(is_send_ipi: &mut bool) {
    if !(*is_send_ipi) && VIRTIO_BRIDGE.need_wakeup() {
        debug!("need wakeup (recheck), sending ipi to wake up virtio device");
        send_event(
            get_target_cpu(IRQ_WAKEUP_VIRTIO_DEVICE, 0),
            SGI_IPI_ID as _,
            IPI_EVENT_WAKEUP_VIRTIO_DEVICE,
        );
        *is_send_ipi = true;
    }
}

/// When virtio req type is notify, root zone will send sgi to non root, \
/// and non root will call this function.
pub fn handle_virtio_irq() {
    let mut map = VIRTIO_IRQS.lock();
    let irq_list = map.get_mut(&this_cpu_id()).unwrap();
    let len = irq_list[0] as usize;
    for irq_id in irq_list[1..=len].iter() {
        inject_irq(*irq_id as _, false);
    }
    irq_list[0] = 0;
}

/// Virtio bridge controller.
pub struct VirtioBridgeController {
    base_address: AtomicUsize,
    is_enable: AtomicBool,
    req_lock: Mutex<()>,
    res_lock: Mutex<()>,
}

impl VirtioBridgeController {
    pub const fn new() -> Self {
        Self {
            base_address: AtomicUsize::new(0),
            is_enable: AtomicBool::new(false),
            req_lock: Mutex::new(()),
            res_lock: Mutex::new(()),
        }
    }

    pub fn set_base_addr(&self, base: usize) {
        self.base_address.store(base, Ordering::Release);
        self.is_enable.store(true, Ordering::Release);
    }

    /// Get req list agent.
    fn req_agent(&self) -> ReqAgent {
        if !self.is_enable.load(Ordering::Acquire) {
            panic!("VirtioBridge not enabled");
        }
        let guard = self.req_lock.lock();
        ReqAgent {
            base: self.base_address.load(Ordering::Relaxed),
            _guard: guard,
        }
    }

    /// Get res list agent.
    pub fn res_agent(&self) -> ResAgent {
        if !self.is_enable.load(Ordering::Acquire) {
            panic!("VirtioBridge not enabled");
        }
        let guard = self.res_lock.lock();
        ResAgent {
            base: self.base_address.load(Ordering::Relaxed),
            _guard: guard,
        }
    }

    /// Get cfg flags (0..MAX_CPUS)
    fn cfg_flags(&self) -> &[ReadWrite<u64>; MAX_CPUS] {
        let base = self.base_address.load(Ordering::Relaxed);
        unsafe { &(*(base as *const VirtioBridge)).cfg_flags }
    }

    /// Get cfg values (0..MAX_CPUS)
    fn cfg_values(&self) -> &[ReadWrite<u64>; MAX_CPUS] {
        let base = self.base_address.load(Ordering::Relaxed);
        unsafe { &(*(base as *const VirtioBridge)).cfg_values }
    }

    pub fn is_cfg_updated(&self, cpu_id: usize, old_val: u64) -> bool {
        let val = self.cfg_flags()[cpu_id].get();
        fence(Ordering::Acquire);
        val != old_val
    }

    pub fn cfg_flag(&self, cpu_id: usize) -> u64 {
        self.cfg_flags()[cpu_id].get()
    }

    pub fn cfg_value(&self, cpu_id: usize) -> u64 {
        self.cfg_values()[cpu_id].get()
    }

    #[allow(unused)]
    pub fn need_wakeup(&self) -> bool {
        let base = self.base_address.load(Ordering::Relaxed);
        fence(Ordering::SeqCst);
        let need_wakeup = unsafe { (&*(base as *const VirtioBridge)).need_wakeup.get() };
        need_wakeup == 1
    }
}

struct ReqAgent<'a> {
    base: usize,
    _guard: MutexGuard<'a, ()>,
}

impl<'a> ReqAgent<'a> {
    fn region(&self) -> &mut VirtioBridge {
        unsafe { &mut *(self.base as *mut VirtioBridge) }
    }

    fn immut_region(&self) -> &VirtioBridge {
        unsafe { &*(self.base as *const VirtioBridge) }
    }

    pub fn is_full(&self) -> bool {
        let region = self.immut_region();
        let req_front = region.req_front.get();
        let req_rear = (region.req_rear.get() + 1) & (MAX_REQ - 1);
        // fence: ensure all following req_list reads are visible after req_rear & req_front read
        fence(Ordering::Acquire);
        req_rear == req_front
    }

    pub fn push_req(&mut self, req: HvisorDeviceReq) {
        let region = self.region();
        let slot = &region.req_list[(region.req_rear.get() % MAX_REQ) as usize];
        slot.src_cpu.set(req.src_cpu);
        slot.address.set(req.address);
        slot.size.set(req.size);
        slot.value.set(req.value);
        slot.src_zone.set(req.src_zone);
        slot.is_write.set(req.is_write);
        slot.need_interrupt.set(req.need_interrupt);
        // Write barrier so that virtio device sees changes to req_list before change to req_idx
        fence(Ordering::Release);
        region.req_rear.set((region.req_rear.get() + 1) % MAX_REQ);
    }
}

pub struct ResAgent<'a> {
    base: usize,
    _guard: MutexGuard<'a, ()>,
}

impl<'a> ResAgent<'a> {
    fn region(&self) -> &mut VirtioBridge {
        unsafe { &mut *(self.base as *mut VirtioBridge) }
    }

    fn immut_region(&self) -> &VirtioBridge {
        unsafe { &*(self.base as *const VirtioBridge) }
    }

    pub fn is_empty(&self) -> bool {
        let region = self.immut_region();
        let res_rear = region.res_rear.get();
        let res_front = region.res_front.get();
        // fence: ensure all following res_list reads are visible after res_rear & res_front read
        fence(Ordering::Acquire);
        res_rear == res_front
    }

    pub fn peek_front(&self) -> (u32, u64, u32) {
        let region = self.immut_region();
        let res_front = region.res_front.get() as usize;
        let res = &region.res_list[res_front];
        let irq_id = res.irq_id.get() as u64;
        let target_zone = res.target_zone.get();
        (res_front as u32, irq_id, target_zone)
    }

    pub fn advance_front(&mut self) {
        let region = self.region();
        // fence: ensure all previous res_list updates are visible before advancing res_front
        fence(Ordering::Release);
        region
            .res_front
            .set((region.res_front.get() + 1) & (MAX_REQ - 1));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtqueueAreaInfo {
    pub desc_area: u64,
    pub avail_area: u64,
    pub used_area: u64,

    pub queue_size: u64,
}

impl VirtqueueAreaInfo {
    pub fn new(desc: u64, avail: u64, used: u64, queue_size: u64) -> Self {
        Self {
            desc_area: desc,
            avail_area: avail,
            used_area: used,
            queue_size,
        }
    }

    pub fn dummy() -> Self {
        Self {
            desc_area: 0,
            avail_area: 0,
            used_area: 0,
            queue_size: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIConfigInfo {
    features: u64,
    dev_id: u16,
    num_of_queues: u16,
    dtype: u16,
    padding: u16,
    vqs: [VirtqueueAreaInfo; MAX_VQ],
}

impl VirtioPCIConfigInfo {
    pub fn dummy() -> Self {
        Self {
            features: 0,
            dev_id: 0,
            num_of_queues: 0,
            dtype: 0,
            padding: 0,
            vqs: [VirtqueueAreaInfo::dummy(); 16],
        }
    }

    pub fn set_features(&mut self, val: u64) {
        self.features = val;
    }

    // pub fn set_dev_id(&mut self, val: u16) {
    //     self.dev_id = val;
    // }

    pub fn set_num_of_queues(&mut self, val: u16) {
        self.num_of_queues = val;
    }

    pub fn set_dtype(&mut self, val: u16) {
        self.dtype = val;
    }

    pub fn set_vqs(&mut self, idx: usize, val: VirtqueueAreaInfo) {
        if idx >= MAX_VQ {
            error!("It's illegal to have idx:0x{:x} exceeding MAX_VQ", idx);
            return;
        }
        self.vqs[idx] = val;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIConfigReq {
    request_id: u32,
    padding: u32,
    info: VirtioPCIConfigInfo,
}

impl VirtioPCIConfigReq {
    fn new(request_id: u32, info: VirtioPCIConfigInfo) -> Self {
        Self {
            request_id,
            padding: 0,
            info,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIConfigRes {
    request_id: u32,
    status: u32,
    dev_id: u16,
    padding: u16,
}

impl VirtioPCIConfigRes {
    #[allow(unused)]
    pub fn success(request_id: u32, dev_id: u16) -> Self {
        Self {
            request_id,
            status: 0,
            dev_id,
            padding: 0,
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtioPCIHypercallOp {
    None = 0,
    ConfigReqComplete = 1,
    DataReqComplete = 2,
    Reset = 3,
}

impl TryFrom<u16> for VirtioPCIHypercallOp {
    type Error = ();

    fn try_from(value: u16) -> core::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::ConfigReqComplete),
            2 => Ok(Self::DataReqComplete),
            3 => Ok(Self::Reset),
            _ => Err(()),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtioPCIHypercallInfo {
    pub version: u16,
    pub op: u16,
    pub target_cpu: u32,
    pub request_id: u32,
    pub status: u32,
}

impl VirtioPCIHypercallInfo {
    pub const fn new() -> Self {
        Self {
            version: VIRTIO_PCI_HYPERCALL_VERSION,
            op: VirtioPCIHypercallOp::None as u16,
            target_cpu: 0,
            request_id: 0,
            status: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VirtioPCIDataInfo {
    dev_id: u16,
    queue_id: u16,
    cpu_id: u16,
    msix_vector_idx: u16,
}

impl VirtioPCIDataInfo {
    pub fn new(dev_id: u16, queue_id: u16, msix_vector_idx: u16) -> Self {
        let cpu_id = this_cpu_id() as u16;
        Self {
            dev_id,
            queue_id,
            cpu_id,
            msix_vector_idx,
        }
    }

    pub fn get_identifier(&self) -> u64 {
        (self.dev_id as u64) | ((self.queue_id as u64) << 16) | ((self.cpu_id as u64) << 32)
    }

    // pub fn from_u64(data_req_id:u64)->Self{
    //     let dev_id = data_req_id&0x0000_0000_0000_ffff;
    //     let msix_vector_idx = data_req_id&0x0000_0000_ffff_0000;
    //     let cpu_id = data_req_id&0x0000_ffff_0000_0000;
    //     Self { dev_id: dev_id as u16, msix_vector_idx: msix_vector_idx as u16, cpu_id: cpu_id as u16, _padding: 0x0000 }
    // }

    // pub fn get_dev_id(&self)->u16{
    //     self.dev_id
    // }

    pub fn get_msix_vector_idx(&self) -> u16 {
        self.msix_vector_idx
    }

    // pub fn get_cpu_id(&self)->u16{
    //     self.cpu_id
    // }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VirtioPCIDataReq {
    request_id: u32,
    padding: u32,
    info: VirtioPCIDataInfo,
}

impl VirtioPCIDataReq {
    fn new(request_id: u32, info: VirtioPCIDataInfo) -> Self {
        Self {
            request_id,
            padding: 0,
            info,
        }
    }
}

#[repr(C)]
struct VirtioPCIBridgeRegion {
    hypercall_info: VirtioPCIHypercallInfo,
    config_req_front: ReadWrite<u32>,
    config_req_rear: ReadWrite<u32>,
    config_res_front: ReadWrite<u32>,
    config_res_rear: ReadWrite<u32>,
    data_req_front: ReadWrite<u32>,
    data_req_rear: ReadWrite<u32>,
    config_req_list: [VirtioPCIConfigReq; MAX_PCI_CONFIG_REQ],
    config_res_list: [VirtioPCIConfigRes; MAX_PCI_CONFIG_RES],
    data_req_list: [VirtioPCIDataReq; MAX_PCI_DATA_REQ],
}

pub struct VirtioPCIBridge {
    base: usize,
    next_request_id: u32,
}

impl VirtioPCIBridge {
    pub const fn dummy() -> Self {
        Self {
            base: 0,
            next_request_id: 1,
        }
    }

    pub fn init(&mut self, addr: usize) {
        self.base = addr;
        self.next_request_id = 1;
        assert!(size_of::<VirtioPCIBridgeRegion>() <= crate::consts::PAGE_SIZE);
        unsafe {
            core::ptr::write_bytes(self.base as *mut u8, 0, size_of::<VirtioPCIBridgeRegion>());
        }
        self.write_hypercall_info(VirtioPCIHypercallInfo::new());
    }

    fn immut_region(&self) -> &VirtioPCIBridgeRegion {
        assert!(self.base != 0, "VirtioPCIBridge not initialized");
        unsafe { &*(self.base as *const VirtioPCIBridgeRegion) }
    }

    fn region(&mut self) -> &mut VirtioPCIBridgeRegion {
        assert!(self.base != 0, "VirtioPCIBridge not initialized");
        unsafe { &mut *(self.base as *mut VirtioPCIBridgeRegion) }
    }

    pub fn write_hypercall_info(&mut self, info: VirtioPCIHypercallInfo) {
        let region = self.region();
        unsafe {
            core::ptr::write_volatile(
                &mut region.hypercall_info as *mut VirtioPCIHypercallInfo,
                info,
            );
        }
        fence(Ordering::Release);
    }

    pub fn hypercall_info(&self) -> VirtioPCIHypercallInfo {
        let region = self.immut_region();
        fence(Ordering::Acquire);
        unsafe { core::ptr::read_volatile(&region.hypercall_info as *const _) }
    }

    fn alloc_request_id(&mut self) -> u32 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id == 0 {
            self.next_request_id = 1;
        }
        request_id
    }

    fn is_config_req_full(&self) -> bool {
        let region = self.immut_region();
        let front = region.config_req_front.get() as usize;
        let rear = region.config_req_rear.get() as usize;
        fence(Ordering::Acquire);
        ((rear + 1) & (MAX_PCI_CONFIG_REQ - 1)) == front
    }

    fn is_data_req_full(&self) -> bool {
        let region = self.immut_region();
        let front = region.data_req_front.get() as usize;
        let rear = region.data_req_rear.get() as usize;
        fence(Ordering::Acquire);
        ((rear + 1) & (MAX_PCI_DATA_REQ - 1)) == front
    }

    fn is_config_res_empty(&self) -> bool {
        let region = self.immut_region();
        let front = region.config_res_front.get();
        let rear = region.config_res_rear.get();
        fence(Ordering::Acquire);
        front == rear
    }

    fn wait_for_queue_slot(&self, is_full: impl Fn(&Self) -> bool, queue_name: &str) -> HvResult {
        let mut count = 0usize;
        let mut backoff = 1usize;
        while is_full(self) {
            for _ in 0..backoff {
                core::hint::spin_loop();
            }
            count += backoff;
            if count >= MAX_WAIT_TIMES {
                return hv_result_err!(EBUSY, format!("virtio pci {queue_name} queue is full"));
            }
            backoff = (backoff << 1).min(MAX_BACKOFF);
        }
        Ok(())
    }

    pub fn push_config_req(&mut self, config: VirtioPCIConfigInfo) -> HvResult<u32> {
        self.wait_for_queue_slot(Self::is_config_req_full, "config request")?;
        let request_id = self.alloc_request_id();
        let region = self.region();
        let rear = region.config_req_rear.get() as usize;
        unsafe {
            core::ptr::write_volatile(
                &mut region.config_req_list[rear] as *mut VirtioPCIConfigReq,
                VirtioPCIConfigReq::new(request_id, config),
            );
        }
        fence(Ordering::Release);
        region
            .config_req_rear
            .set(((rear + 1) & (MAX_PCI_CONFIG_REQ - 1)) as u32);
        Ok(request_id)
    }

    pub fn push_data_req(&mut self, data_info: VirtioPCIDataInfo) -> HvResult<u32> {
        self.wait_for_queue_slot(Self::is_data_req_full, "data request")?;
        let request_id = self.alloc_request_id();
        let region = self.region();
        let rear = region.data_req_rear.get() as usize;
        unsafe {
            core::ptr::write_volatile(
                &mut region.data_req_list[rear] as *mut VirtioPCIDataReq,
                VirtioPCIDataReq::new(request_id, data_info),
            );
        }
        fence(Ordering::Release);
        region
            .data_req_rear
            .set(((rear + 1) & (MAX_PCI_DATA_REQ - 1)) as u32);
        Ok(request_id)
    }

    fn peek_config_res(&self) -> VirtioPCIConfigRes {
        let region = self.immut_region();
        let front = region.config_res_front.get() as usize;
        fence(Ordering::Acquire);
        unsafe { core::ptr::read_volatile(&region.config_res_list[front] as *const _) }
    }

    fn advance_config_res(&mut self) {
        let region = self.region();
        let front = region.config_res_front.get() as usize;
        fence(Ordering::Release);
        region
            .config_res_front
            .set(((front + 1) & (MAX_PCI_CONFIG_RES - 1)) as u32);
    }

    pub fn til_config_finish(&mut self, request_id: u32) -> HvResult<u16> {
        let mut count = 0usize;
        let mut backoff = 1usize;

        loop {
            if !self.is_config_res_empty() {
                let res = self.peek_config_res();
                self.advance_config_res();
                if res.request_id != request_id {
                    return hv_result_err!(
                        EIO,
                        format!(
                            "virtio pci config response mismatch: expect {}, got {}",
                            request_id, res.request_id
                        )
                    );
                }
                if res.status != 0 {
                    return hv_result_err!(
                        EIO,
                        format!(
                            "virtio pci config request {} failed with status {}",
                            request_id, res.status
                        )
                    );
                }
                return Ok(res.dev_id);
            }

            for _ in 0..backoff {
                core::hint::spin_loop();
            }
            count += backoff;
            if count >= MAX_WAIT_TIMES {
                return hv_result_err!(
                    EBUSY,
                    format!("virtio pci config request {} timed out", request_id)
                );
            }
            backoff = (backoff << 1).min(MAX_BACKOFF);
        }
    }
}

/// El1 and EL2 shared region for virtio requests and results.
#[repr(C)]
struct VirtioBridge {
    /// The first elem of req list, only virtio device updates
    req_front: ReadWrite<u32>,
    /// The last elem's next place of req list, only hvisor updates
    req_rear: ReadWrite<u32>,
    /// The first elem of res list, only hvisor updates
    res_front: ReadWrite<u32>,
    /// The last elem's next place of res list, only virtio device updates
    res_rear: ReadWrite<u32>,
    req_list: [HvisorDeviceReqVolatile; MAX_REQ as usize],
    res_list: [HvisorDeviceResVolatile; MAX_REQ as usize], // irqs
    cfg_flags: [ReadWrite<u64>; MAX_CPUS],
    cfg_values: [ReadWrite<u64>; MAX_CPUS],
    _mmio_addrs: [ReadWrite<u64>; MAX_DEVS], // remove later
    _mmio_avail: ReadWrite<u8>,              // remove later
    need_wakeup: ReadWrite<u8>,
}

impl Debug for VirtioBridge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("VirtioBridge")
            .field("req_front", &self.req_front.get())
            .field("req_rear", &self.req_rear.get())
            .field("res_front", &self.res_front.get())
            .field("res_rear", &self.res_rear.get())
            .finish()
    }
}

/// Hvisor device requests
#[repr(C)]
#[derive(Debug)]
struct HvisorDeviceReq {
    src_cpu: u64,
    address: u64,
    size: u64,
    value: u64,
    src_zone: u32,
    is_write: u8,
    need_interrupt: u8,
    _padding: u16,
}

impl HvisorDeviceReq {
    fn new(
        src_cpu: u64,
        address: u64,
        size: u64,
        value: u64,
        src_zone: u32,
        is_write: bool,
        need_interrupt: u8,
    ) -> Self {
        let is_write = if is_write { 1 } else { 0 };
        Self {
            src_cpu,
            address,
            size,
            value,
            src_zone,
            is_write,
            need_interrupt,
            _padding: 0,
        }
    }
}

#[repr(C)]
struct HvisorDeviceReqVolatile {
    src_cpu: ReadWrite<u64>,
    address: ReadWrite<u64>,
    size: ReadWrite<u64>,
    value: ReadWrite<u64>,
    src_zone: ReadWrite<u32>,
    is_write: ReadWrite<u8>,
    need_interrupt: ReadWrite<u8>,
    _padding: u16,
}

#[repr(C)]
struct HvisorDeviceResVolatile {
    target_zone: ReadWrite<u32>,
    irq_id: ReadWrite<u32>,
}
