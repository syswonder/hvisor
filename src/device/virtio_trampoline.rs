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

use crate::{
    arch::cpu::{get_target_cpu, this_cpu_id},
    consts::MAX_WAIT_TIMES,
    device::irqchip::inject_irq,
    error::HvResult,
    event::{send_event, IPI_EVENT_WAKEUP_VIRTIO_DEVICE},
    hypercall::SGI_IPI_ID,
    memory::MMIOAccess,
    zone::this_zone_id,
};
use alloc::collections::BTreeMap;
use core::{
    fmt::{Debug, Formatter, Result},
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
pub static VIRTIO_BRIDGE: VirtioBridgeController = VirtioBridgeController::new();

const QUEUE_NOTIFY: usize = 0x50;
pub const MAX_REQ: u32 = 32;
pub const MAX_DEVS: usize = 8; // Attention: The max virtio-dev number for vm is 8 (loongarch64 needs 3 consoles and 3 disks for zgclab project).
pub const MAX_CPUS: usize = 32;
pub const MAX_BACKOFF: usize = 1024;

use crate::platform::IRQ_WAKEUP_VIRTIO_DEVICE;
// #[cfg(all(not(target_arch = "riscv64"), not(target_arch = "x86_64")))]
// pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
// #[cfg(target_arch = "riscv64")]
// pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x20;
// #[cfg(target_arch = "x86_64")]
// pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x6;

/// non root zone's virtio request handler
pub fn mmio_virtio_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    // debug!("mmio virtio handler");
    let cpu_id = this_cpu_id() as usize;
    let need_interrupt = if mmio.address == QUEUE_NOTIFY { 1 } else { 0 };
    if need_interrupt == 1 {
        trace!("notify !!!, cpu id is {}", cpu_id);
    }
    mmio.address += base;
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

    // Due to cfg_flag and cfg_value are per-cpu, so there is no need to lock them.
    let old_cfg_flag = VIRTIO_BRIDGE.cfg_flag(cpu_id);
    let mut count: usize = 0;
    let mut ipi_sent = false;
    // if it is cfg request, current cpu should be blocked until gets the result
    if need_interrupt == 0 {
        loop {
            // If backend is sleep, hvisor needs to send ipi to wake it up.
            #[cfg(not(target_arch = "loongarch64"))]
            if !ipi_sent && VIRTIO_BRIDGE.need_wakeup() {
                debug!("need wakeup (recheck), sending ipi to wake up virtio device");
                send_event(
                    get_target_cpu(IRQ_WAKEUP_VIRTIO_DEVICE, 0),
                    SGI_IPI_ID as _,
                    IPI_EVENT_WAKEUP_VIRTIO_DEVICE,
                );
                ipi_sent = true;
            }
            // when virtio backend finish the req, it will add 1 to cfg_flags[cpu_id].
            if VIRTIO_BRIDGE.is_cfg_updated(cpu_id, old_cfg_flag) {
                break;
            }
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
    pub fn req_agent(&self) -> ReqAgent {
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

    pub fn need_wakeup(&self) -> bool {
        let base = self.base_address.load(Ordering::Relaxed);
        let need_wakeup = unsafe { (&*(base as *const VirtioBridge)).need_wakeup.get() };
        fence(Ordering::Acquire);
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
