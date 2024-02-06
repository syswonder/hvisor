const VIRT_QUEUE_SIZE: usize = 512;

#[repr(C, align(16))]
#[derive(Copy, Clone)]
struct VirtqDesc {
    /*Address (guest-physical)*/
    pub addr: usize,
    /* Length */
    len: u32,
    /* The flags as indicated above */
    flags: u16,
    /* We chain unused descriptors via this, too */
    next: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; VIRT_QUEUE_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VirtqUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; VIRT_QUEUE_SIZE],
}

pub struct VirtQueue {
    vq_idx: usize,
    num: usize,
    desc_table_addr: usize,
    avail_addr: usize,
    used_addr: usize,

    desc_table: Option<&'static mut VirtqDesc>,
    avail_ring: Option<&'static mut VirtqAvail>,
    used_ring: Option<&'static mut VirtqUsed>,
    
    notify_handler: Option<fn()>,
    last_avail_idx: usize,
    last_used_idx: usize,
    used_flags: u16,

    ready: usize,   
}