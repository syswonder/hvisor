use super::virtio::VirtioDeviceType;

pub struct VirtMmioRegs {
    device_id: u32,
    dev_feature_sel: u32,
    drv_feature_sel: u32,
    queue_sel: u32,
    queue_num_max: u32,
    interrupt_status: u32,
    status: u32,
    generation: u32,
}

impl VirtMmioRegs {
    fn new(dev_type: VirtioDeviceType) -> Self {
        Self {
            device_id: dev_type as u32,
            dev_feature_sel: 0,
            drv_feature_sel: 0,
            queue_sel: 0,
            queue_num_max: 0,
            interrupt_status: 0,
            status: 0,
            generation: 0,
        }
    }
}
