//! Hvisor user program
// #![deny(warnings, missing_docs)]
use std::{fs::File, num::NonZeroUsize, os::fd::FromRawFd};

use log::{error, info};
use nix::{
    fcntl::{open, OFlag},
    ioctl_none,
    sys::{
        mman::{mmap, MapFlags, ProtFlags},
        stat::Mode,
    }, libc::PT_NULL,
};

use crate::device::virtio::{HvisorDeviceRegion, HVISOR_REGION, HVISOR_DEVICE_REGION};

pub mod device;

ioctl_none!(
    /// ioctl for init virtio
    ioctl_init_virtio,
    1,
    0
);

fn main() {
    info!("hvisor user init");
    let fd = open("/dev/hvisor", OFlag::O_RDWR, Mode::empty()).unwrap();
    unsafe {
        let hvisor_file = File::from_raw_fd(fd);
        let res = ioctl_init_virtio(fd).unwrap();
        if res < 0 {
            error!("ioctl init virtio error!");
        }
        let res = mmap(
            None,
            NonZeroUsize::new(4096).unwrap(),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            Some(hvisor_file),
            0,
        ).unwrap();
        let res = &mut *(res as *mut HvisorDeviceRegion);
        HVISOR_DEVICE_REGION = Some(res);
    }
    
}
