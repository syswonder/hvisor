use alloc::collections::BTreeMap;
use core::sync::atomic::fence;
use core::sync::atomic::Ordering;
use spin::Mutex;

use crate::{
    cell::this_cell_id, control::suspend_self, error::HvResult, memory::MMIOAccess,
    percpu::this_cpu_id,
};

use super::gicv3::inject_irq;

/// Save the irqs the virtio-device wants to inject. The format is <cpu_id, List<irq_id>>, and the first elem of List<irq_id> is the valid len of it.
pub static VIRTIO_IRQS: Mutex<BTreeMap<u64, [u64; MAX_DEVS + 1]>> = Mutex::new(BTreeMap::new());
/// Save the results the virtio-device returns to the vm for cfg requests. The format is <cpu_id, returned value>
// pub static VIRTIO_CFG_RESULTS: Mutex<BTreeMap<u64, u64>> = Mutex::new(BTreeMap::new());
// Controller of the shared memory the root linux's virtio device and hvisor shares.
pub static HVISOR_DEVICE: Mutex<HvisorDevice> = Mutex::new(HvisorDevice::default());

const QUEUE_NOTIFY: usize = 0x50;
pub const MAX_REQ: u32 = 32;
pub const MAX_DEVS: usize = 4; // Attention: The max virtio-dev number for vm is 4.
pub const MAX_CPUS: usize = 20;
/// non root cell's virtio request handler
pub fn mmio_virtio_handler(mmio: &mut MMIOAccess, base: u64) -> HvResult {
    debug!("mmio virtio handler");
    let need_interrupt = if mmio.address == QUEUE_NOTIFY { 1 } else { 0 };
    if need_interrupt == 1 {
        debug!("notify !!!, cpu id is {}", this_cpu_id());
    }
    mmio.address += base as usize;
    let mut dev = HVISOR_DEVICE.lock();
    while dev.is_req_list_full() {
        // When root linux's cpu is in el2's finish req handler and is getting the dev lock,
        // if we don't release dev lock, it will cause a dead lock.
        drop(dev);
        dev = HVISOR_DEVICE.lock();
    }
    let hreq = HvisorDeviceReq::new(
        this_cpu_id(),
        mmio.address as _,
        mmio.size,
        mmio.value,
        this_cell_id(),
        mmio.is_write,
        need_interrupt,
    );
    let (cfg_flags, cfg_values) = unsafe {
        (
            core::slice::from_raw_parts(dev.get_cfg_flags(), MAX_CPUS),
            core::slice::from_raw_parts(dev.get_cfg_values(), MAX_CPUS),
        )
    };
    let cpu_id = this_cpu_id() as usize;
    let old_cfg_flag = cfg_flags[cpu_id];
    dev.push_req(hreq);
    drop(dev);
    // if it is cfg request, current cpu should be blocked until gets the result
    if need_interrupt == 0 {
        // when virtio backend finish the req, it will add 1 to cfg_flag.
        // info!("wait return");
        while cfg_flags[cpu_id] == old_cfg_flag {}
        // info!("have return");
        if !mmio.is_write {
            // ensure cfg value is right.
            fence(Ordering::Acquire);
            mmio.value = cfg_values[cpu_id];
            debug!("non root receives value: {:#x?}", mmio.value);
        }
    }
    debug!("non root returns");
    Ok(())
}

/// When virtio req type is notify, root cell will send sgi to non root, \
/// and non root will call this function.
pub fn handle_virtio_result() {
    debug!("notify resolved");
    let mut map = VIRTIO_IRQS.lock();
    let irq_list = map.get_mut(&this_cpu_id()).unwrap();
    let len = irq_list[0] as usize;
    for irq_id in irq_list[1..=len].iter() {
        inject_irq(*irq_id as _, false);
    }
    irq_list[0] = 0;
}

pub struct HvisorDevice {
    base_address: usize, // el1 and el2 shared region addr, el2 virtual address
    is_enable: bool,
}

impl HvisorDevice {
    // return a mut region
    pub fn region(&self) -> &mut HvisorDeviceRegion {
        if !self.is_enable {
            panic!("hvisor device region is not enabled!");
        }
        unsafe { &mut *(self.base_address as *mut HvisorDeviceRegion) }
    }
    // return a non mut region
    fn immut_region(&self) -> &HvisorDeviceRegion {
        if !self.is_enable {
            panic!("hvisor device region is not enabled!");
        }
        unsafe { &*(self.base_address as *const HvisorDeviceRegion) }
    }

    pub const fn default() -> Self {
        HvisorDevice {
            base_address: 0,
            is_enable: false,
        }
    }

    pub fn set_base_addr(&mut self, base_addr: usize) {
        self.base_address = base_addr;
        self.is_enable = true;
    }

    pub fn is_req_list_full(&self) -> bool {
        let region = self.immut_region();
        if ((region.req_rear + 1) & (MAX_REQ - 1)) == region.req_front {
            debug!("hvisor req queue full");
            true
        } else {
            false
        }
    }

    pub fn is_res_list_empty(&self) -> bool {
        let region = self.immut_region();
        if region.res_rear == region.res_front {
            true
        } else {
            false
        }
    }

    pub fn push_req(&mut self, req: HvisorDeviceReq) {
        let region = self.region();
        region.req_list[(region.req_rear % MAX_REQ) as usize] = req;
        // Write barrier so that virtio device sees changes to req_list before change to req_idx
        fence(Ordering::SeqCst);
        region.req_rear = (region.req_rear + 1) % MAX_REQ;
        // Write barrier so that device can see change after this method returns
        fence(Ordering::SeqCst);
    }

    pub fn get_cfg_flags(&self) -> *const u8 {
        let region = self.immut_region();
        region.cfg_flags.as_ptr()
    }

    pub fn get_cfg_values(&self) -> *const u64 {
        let region = self.immut_region();
        region.cfg_values.as_ptr()
    }
}

/// El1 and EL2 shared region for virtio requests and results.
#[repr(C)]
pub struct HvisorDeviceRegion {
    /// The first elem of req list, only virtio device updates
    pub req_front: u32,
    /// The last elem's next place of req list, only hvisor updates
    pub req_rear: u32,
    /// The first elem of res list, only hvisor updates
    pub res_front: u32,
    /// The last elem's next place of res list, only virtio device updates
    res_rear: u32,
    pub req_list: [HvisorDeviceReq; MAX_REQ as usize],
    pub res_list: [HvisorDeviceRes; MAX_REQ as usize], // irqs
    cfg_flags: [u8; MAX_CPUS],
    cfg_values: [u64; MAX_CPUS],
}

/// Hvisor device requests
#[repr(C)]
pub struct HvisorDeviceReq {
    pub src_cpu: u64,
    address: u64,
    size: u64,
    pub value: u64,
    src_cell: u32,
    is_write: u8,
    pub need_interrupt: u8,
}

#[repr(C)]
pub struct HvisorDeviceRes {
    pub target_cell: u32,
    pub irq_id: u32,
}

impl HvisorDeviceReq {
    pub fn new(
        src_cpu: u64,
        address: u64,
        size: u64,
        value: u64,
        src_cell: u32,
        is_write: bool,
        need_interrupt: u8,
    ) -> Self {
        let is_write = if is_write { 1 } else { 0 };
        Self {
            src_cpu,
            address,
            size,
            value,
            src_cell,
            is_write,
            need_interrupt,
        }
    }
}
