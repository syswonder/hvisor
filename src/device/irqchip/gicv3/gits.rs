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
//
use core::ptr;

use spin::{mutex::Mutex, Once};

use crate::{
    consts::MAX_ZONE_NUM, device::irqchip::gicv3::gicr::enable_one_lpi, memory::Frame,
    zone::this_zone_id,
};

use super::host_gits_base;

pub const GITS_CTRL: usize = 0x0000; // enable / disable
pub const GITS_IIDR: usize = 0x0004; // read-only
pub const GITS_TYPER: usize = 0x0008; // read-only
pub const GITS_MPAMIDR: usize = 0x0010; // read-only
pub const GITS_PARTIDR: usize = 0x0014; // supported MPAM sizes
pub const GITS_MPIDR: usize = 0x0018; // read-only, its affinity
pub const GITS_STATUSR: usize = 0x0040; // error reporting
pub const GITS_UMSIR: usize = 0x0048; // unmapped msi
pub const GITS_CBASER: usize = 0x0080; // the addr of command queue
pub const GITS_CWRITER: usize = 0x0088; // rw, write an command to the cmdq, write this reg to tell hw
pub const GITS_CREADR: usize = 0x0090; // read-only, hardware changes it
pub const GITS_BASER: usize = 0x0100; // itt, desc
pub const GITS_COLLECTION_BASER: usize = GITS_BASER + 0x8;
pub const GITS_TRANSLATER: usize = 0x10000 + 0x0040; // to signal an interrupt, written by devices

pub const PER_CMD_BYTES: usize = 0x20;
pub const PER_CMD_QWORD: usize = PER_CMD_BYTES >> 3;

fn ring_ptr_update(val: usize) -> usize {
    if val >= 0x10000 {
        val - 0x10000
    } else {
        val
    }
}

// created by root linux, and make a virtual one to non root
pub struct DeviceTable {
    baser: usize,
}

impl DeviceTable {
    fn new() -> Self {
        let dt_baser_reg = host_gits_base() + GITS_BASER;
        let dt_baser = unsafe { ptr::read_volatile(dt_baser_reg as *mut u64) };
        Self {
            baser: dt_baser as _,
        }
    }

    fn set_baser(&mut self, value: usize) {
        self.baser = value;
    }

    fn read_baser(&self) -> usize {
        self.baser
    }
}

pub struct CollectionTable {
    baser: usize,
}

impl CollectionTable {
    fn new() -> Self {
        let ct_baser_reg = host_gits_base() + GITS_COLLECTION_BASER;
        let ct_baser = unsafe { ptr::read_volatile(ct_baser_reg as *mut u64) };
        Self {
            baser: ct_baser as _,
        }
    }

    fn set_baser(&mut self, value: usize) {
        self.baser = value;
    }

    fn read_baser(&self) -> usize {
        self.baser
    }
}

pub struct Cmdq {
    phy_addr: usize,
    readr: usize,
    writer: usize,
    frame: Frame,

    phy_base_list: [usize; MAX_ZONE_NUM],
    cbaser_list: [usize; MAX_ZONE_NUM],
    creadr_list: [usize; MAX_ZONE_NUM],
    cwriter_list: [usize; MAX_ZONE_NUM],
}

impl Cmdq {
    fn new() -> Self {
        let f = Frame::new_contiguous(16, 0).unwrap();
        trace!("its cmdq base: 0x{:x}", f.start_paddr());
        let r = Self {
            phy_addr: f.start_paddr(),
            readr: 0,
            writer: 0,
            frame: f,
            phy_base_list: [0; MAX_ZONE_NUM],
            cbaser_list: [0; MAX_ZONE_NUM],
            creadr_list: [0; MAX_ZONE_NUM],
            cwriter_list: [0; MAX_ZONE_NUM],
        };
        r.init_real_cbaser();
        r
    }

    fn init_real_cbaser(&self) {
        let reg = host_gits_base() + GITS_CBASER;
        let writer = host_gits_base() + GITS_CWRITER;
        let val = 0xb80000000000040f | self.phy_addr;
        let ctrl = host_gits_base() + GITS_CTRL;
        unsafe {
            let origin_ctrl = ptr::read_volatile(ctrl as *mut u64);
            ptr::write_volatile(ctrl as *mut u64, origin_ctrl | 0xfffffffffffffffeu64); // turn off, vm will turn on this ctrl
            ptr::write_volatile(reg as *mut u64, val as u64);
            ptr::write_volatile(writer as *mut u64, 0 as u64); // init cwriter
        }
    }

    fn set_cbaser(&mut self, zone_id: usize, value: usize) {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.cbaser_list[zone_id] = value;
        self.phy_base_list[zone_id] = value & 0xffffffffff000;
    }

    fn read_baser(&self, zone_id: usize) -> usize {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.cbaser_list[zone_id]
    }

    fn set_cwriter(&mut self, zone_id: usize, value: usize) {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        if value == 0 {
            trace!("ignore first write");
        } else {
            self.insert_cmd(zone_id, value);
        }

        self.cwriter_list[zone_id] = value;
    }

    fn read_cwriter(&mut self, zone_id: usize) -> usize {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.cwriter_list[zone_id]
    }

    fn read_creadr(&mut self, zone_id: usize) -> usize {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.creadr_list[zone_id]
    }

    fn update_creadr(&mut self, zone_id: usize, writer: usize) {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.creadr_list[zone_id] = writer;
    }

    // it's ok to add qemu-args: -trace gicv3_gits_cmd_*, remember to remain `enable one lpi`
    fn analyze_cmd(&self, value: [u64; 4]) {
        let code = (value[0] & 0xff) as usize;
        match code {
            0x0b => {
                let id = value[0] & 0xffffffff00000000;
                let event = value[1] & 0xffffffff;
                let icid = value[2] & 0xffff;
                enable_one_lpi((event - 8192) as _);
                trace!(
                    "MAPI cmd, for device {:#x}, event = intid = {:#x} -> icid {:#x}",
                    id >> 32,
                    event,
                    icid
                );
            }
            0x08 => {
                let id = value[0] & 0xffffffff00000000;
                let itt_base = (value[2] & 0x000fffffffffffff) >> 8;
                trace!(
                    "MAPD cmd, set ITT: {:#x} to device {:#x}",
                    itt_base,
                    id >> 32
                );
            }
            0x0a => {
                let id = value[0] & 0xffffffff00000000;
                let event = value[1] & 0xffffffff;
                let intid = value[1] >> 32;
                let icid = value[2] & 0xffff;
                enable_one_lpi((intid - 8192) as _);
                trace!(
                    "MAPTI cmd, for device {:#x}, event {:#x} -> icid {:#x} + intid {:#x}",
                    id >> 32,
                    event,
                    icid,
                    intid
                );
            }
            0x09 => {
                let icid = value[2] & 0xffff;
                let rd_base = (value[2] >> 16) & 0x7ffffffff;
                trace!("MAPC cmd, icid {:#x} -> redist {:#x}", icid, rd_base);
            }
            0x05 => {
                trace!("SYNC cmd");
            }
            0x04 => {
                trace!("CLEAR cmd");
            }
            0x0f => {
                trace!("DISCARD cmd");
            }
            0x03 => {
                trace!("INT cmd");
            }
            0x0c => {
                trace!("INV cmd");
            }
            0x0d => {
                trace!("INVALL cmd");
            }
            _ => {
                trace!("other cmd, code: 0x{:x}", code);
            }
        }
    }

    fn insert_cmd(&mut self, zone_id: usize, writer: usize) {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id");

        let zone_addr = self.phy_base_list[zone_id];

        let origin_readr = self.creadr_list[zone_id];

        let cmd_size = writer - origin_readr;
        let cmd_num = cmd_size / PER_CMD_BYTES;

        trace!("cmd size: {:#x}, cmd num: {:#x}", cmd_size, cmd_num);

        let mut vm_cmdq_addr = zone_addr + origin_readr;
        let mut real_cmdq_addr = self.phy_addr + self.readr;

        for _cmd_id in 0..cmd_num {
            unsafe {
                let v = ptr::read_volatile(vm_cmdq_addr as *mut [u64; PER_CMD_QWORD]);
                self.analyze_cmd(v.clone());

                for i in 0..PER_CMD_QWORD {
                    ptr::write_volatile(real_cmdq_addr as *mut u64, v[i] as u64);
                    real_cmdq_addr += 8;
                }
            }
            vm_cmdq_addr += PER_CMD_BYTES;
            vm_cmdq_addr = ring_ptr_update(vm_cmdq_addr - zone_addr) + zone_addr;
            real_cmdq_addr = ring_ptr_update(real_cmdq_addr - self.phy_addr) + self.phy_addr;
        }

        self.writer += cmd_size;
        self.writer = ring_ptr_update(self.writer); // ring buffer ptr
        let cwriter = host_gits_base() + GITS_CWRITER;
        let readr = host_gits_base() + GITS_CREADR;
        unsafe {
            ptr::write_volatile(cwriter as *mut u64, self.writer as _);
            loop {
                self.readr = (ptr::read_volatile(readr as *mut u64)) as usize; // hw readr
                if self.readr == self.writer {
                    trace!(
                        "readr={:#x}, writer={:#x}, its cmd end",
                        self.readr,
                        self.writer
                    );
                    break;
                } else {
                }
            }
        }
        self.update_creadr(zone_id, writer);
    }
}

pub static DT: Once<Mutex<DeviceTable>> = Once::new();
pub static CMDQ: Once<Mutex<Cmdq>> = Once::new();
pub static CT: Once<Mutex<CollectionTable>> = Once::new();

pub fn gits_init() {
    DT.call_once(|| Mutex::new(DeviceTable::new()));
    CMDQ.call_once(|| Mutex::new(Cmdq::new()));
    CT.call_once(|| Mutex::new(CollectionTable::new()));
}

pub fn set_cbaser(value: usize) {
    let mut cmdq = CMDQ.get().unwrap().lock();
    cmdq.set_cbaser(this_zone_id(), value);
}

pub fn read_cbaser() -> usize {
    let cmdq = CMDQ.get().unwrap().lock();
    cmdq.read_baser(this_zone_id())
}

pub fn set_cwriter(value: usize) {
    let mut cmdq = CMDQ.get().unwrap().lock();
    cmdq.set_cwriter(this_zone_id(), value);
}

pub fn read_cwriter() -> usize {
    let mut cmdq = CMDQ.get().unwrap().lock();
    cmdq.read_cwriter(this_zone_id())
}

pub fn read_creadr() -> usize {
    let mut cmdq = CMDQ.get().unwrap().lock();
    cmdq.read_creadr(this_zone_id())
}

pub fn read_dt_baser() -> usize {
    let dt = DT.get().unwrap().lock();
    dt.read_baser()
}

pub fn set_dt_baser(value: usize) {
    let mut dt = DT.get().unwrap().lock();
    dt.set_baser(value);
}

pub fn read_ct_baser() -> usize {
    let ct = CT.get().unwrap().lock();
    ct.read_baser()
}

pub fn set_ct_baser(value: usize) {
    let mut ct = CT.get().unwrap().lock();
    ct.set_baser(value);
}
