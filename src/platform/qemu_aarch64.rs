pub const TENANT1_DTB_ADDR: usize = 0x90000000;
pub const TENANT2_DTB_ADDR: usize = 0x91000000;

pub static TENANTS: [usize; 2] = [TENANT1_DTB_ADDR, TENANT2_DTB_ADDR];

// #[repr(C)]
// #[repr(align(4096))]
// pub struct DTBBlob1([u8; include_bytes!("../../tenants/aarch64/devicetree/linux1.dtb").len()]);

// #[link_section = ".dtb"]
// /// the tenant dtb file
// pub static TENANT1_DTB: DTBBlob1 = DTBBlob1(*include_bytes!("../../tenants/aarch64/devicetree/linux1.dtb"));

// #[link_section = ".zone0"]
// /// the tenant kernel file
// pub static TENANT1: [u8; include_bytes!("../../tenants/aarch64/kernel/Image").len()] =
//     *include_bytes!("../../tenants/aarch64/kernel/Image");

// #[repr(C)]
// #[repr(align(4096))]
// pub struct DTBBlob2([u8; include_bytes!("../../tenants/aarch64/devicetree/linux2.dtb").len()]);

// #[link_section = ".dtb"]
// /// the tenant dtb file
// pub static TENANT2_DTB: DTBBlob2 = DTBBlob2(*include_bytes!("../../tenants/aarch64/devicetree/linux2.dtb"));

// #[link_section = ".zone1"]
// /// the tenant kernel file
// pub static TENANT2: [u8; include_bytes!("../../tenants/aarch64/kernel/Image").len()] =
//     *include_bytes!("../../tenants/aarch64/kernel/Image");

// pub static TENANTS: [(&'static [u8], &'static [u8]); 1] = [(&TENANT2, &TENANT2_DTB.0)];
// // pub static TENANTS: [(&'static [u8], &'static [u8]); 2] =
// //     [(&TENANT1, &TENANT1_DTB.0), (&TENANT2, &TENANT2_DTB.0)];
