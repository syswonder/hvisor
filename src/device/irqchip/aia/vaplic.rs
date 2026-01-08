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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use super::*;
use crate::cpu_data::this_cpu_data;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitvec::prelude::*;
use spin::Mutex;

// Note: one hart execute one VM!
pub const GUEST_INDEX: usize = 1; // MSI-mode: forward irq to VS

/// Virtual Advanced Platform-Level Interrupt Controller (vAPLIC)
#[allow(unused)]
pub struct VirtualAPLIC {
    /// Base address of the vAPLIC in guest physical memory
    base_addr: usize,
    /// Maximum number of interrupts
    max_interrupts: usize,
    /// Inner state of the vAPLIC (thread-safe)
    inner: Arc<Mutex<VirtualAPLICInner>>,
}

/// Inner state of the vAPLIC, only supports MSI-mode
struct VirtualAPLICInner {
    /// Hardware interrupt bitmap
    hw: BitVec,
    /// Active interrupt bitmap
    active: BitVec,
    /// Pending interrupt bitmap
    pending: BitVec,
    /// Interrupt enable bitmap
    enable: BitVec,
    /// Domain configuration
    domaincfg: u32,
    /// Source configuration for each interrupt
    srccfg: Vec<u32>,
    /// Interrupt targets
    target: Vec<u32>,
}

impl VirtualAPLIC {
    /// Create a new vAPLIC instance
    pub fn new(base_addr: usize, max_interrupts: usize) -> Self {
        VirtualAPLIC {
            base_addr,
            max_interrupts,
            inner: Arc::new(Mutex::new(VirtualAPLICInner {
                hw: bitvec![0; max_interrupts + 1],
                active: bitvec![0; max_interrupts + 1],
                pending: bitvec![0; max_interrupts + 1],
                enable: bitvec![0; max_interrupts + 1],
                domaincfg: 0,
                srccfg: vec![0; max_interrupts + 1],
                target: vec![0; max_interrupts + 1],
            })),
        }
    }

    /// Set one interrupt as hardware interrupt.
    pub fn vaplic_set_hw(&self, intr_id: usize, hw: bool) {
        let mut inner = self.inner.lock();
        inner.vaplic_set_hw(intr_id, hw);
    }

    /// Get one interrupt as hardware interrupt.
    pub fn vaplic_get_hw(&self, intr_id: usize) -> bool {
        let inner = self.inner.lock();
        inner.vaplic_get_hw(intr_id)
    }

    /// Get one interrupt as hardware interrupt.
    pub fn vaplic_get_target(&self, intr_id: usize) -> u32 {
        let inner = self.inner.lock();
        inner.vaplic_get_target(intr_id)
    }

    /// vAPLIC emul access.
    pub fn vaplic_emul_access(
        &self,
        offset: usize,
        size: usize,
        value: usize,
        is_write: bool,
    ) -> u32 {
        if size != 4 || offset & 0x3 != 0 {
            panic!("vaplic_emul_access: only allowed word accesses");
            return 0;
        }

        match offset {
            // Domain configuration
            0x0000 => {
                let mut inner = self.inner.lock();
                if is_write {
                    // Write domain config
                    let msi_mode = (value as u32 & 0b100 != 0);
                    // let phys_msi_mode = host_aplic().get_msimode();
                    // if (msi_mode != phys_msi_mode) {
                    //     error!(
                    //         "vAPLIC msi_mode {} is different from host APLIC msi_mode {}",
                    //         msi_mode, phys_msi_mode
                    //     );
                    //     return 0;
                    // }
                    let bigendian = (value as u32 & 0b1 != 0);
                    // let phys_bigendian = host_aplic().get_bigendian();
                    // if (bigendian != phys_bigendian) {
                    //     error!(
                    //         "vAPLIC bigendian {} is different from host APLIC bigendian {}",
                    //         bigendian, phys_bigendian
                    //     );
                    //     return 0;
                    // }
                    info!(
                        "Set vAPLIC domaincfg to {:#x}, msi_mode {}, bigendian {}",
                        value, msi_mode, bigendian
                    );
                    let new_value = value & 0b1_0000_0101; // 3bits: IE, MSI-mode, Bigendian
                    inner.vaplic_set_domaincfg(new_value as u32);
                } else {
                    // Read domain config
                    return inner.vaplic_get_domaincfg() as _;
                }
            }
            // Source configuration
            0x0004..=0x0FFC => {
                let irq_id = (offset - 0x0004) / 4 + 1; // Begin from irq_id 1.
                let mut inner = self.inner.lock();
                if is_write {
                    if ((value >> 10) & 0x1 == 0x1) {
                        error!("vAPLIC sourcecfg delegate isn't supported!");
                        return 0;
                    } else {
                        let mode = match value {
                            0 => SourceMode::Inactive,
                            1 => SourceMode::Detached,
                            4 => SourceMode::RisingEdge,
                            5 => SourceMode::FallingEdge,
                            6 => SourceMode::LevelHigh,
                            7 => SourceMode::LevelLow,
                            _ => {
                                error!("Unknown sourcecfg mode");
                                return 0;
                            }
                        };
                        if inner.hw[irq_id] {
                            host_aplic().set_sourcecfg(irq_id as u32, mode);
                            inner.vaplic_set_sourcecfg(irq_id, value as u32);
                        } else {
                            // if mode != SourceMode::Inactive {
                            //     error!(
                            //         "Want to set sourcecfg active for IRQ {} with no hw flag.",
                            //         irq_id
                            //     );
                            //     return 0;
                            // }
                            inner.vaplic_set_sourcecfg(irq_id, value as u32);
                        }
                    }
                } else {
                    info!("Read vAPLIC sourcecfg for IRQ {}", irq_id);
                    return inner.vaplic_get_sourcecfg((offset - 0x0004) / 4) as _;
                }
            }
            // msiaddrcfg
            0x1BC8..=0x1BCC => {
                if is_write {
                    let host_msiaddr = host_aplic().get_msiaddr();
                    if host_msiaddr != value {
                        error!(
                            "vAPLIC msiaddrcfg {:x} is different from host APLIC msiaddrcfg {:x}",
                            value, host_msiaddr
                        );
                        return 0;
                    }
                } else {
                    error!("Want to read APLIC msiaddrcfg");
                }
            }
            // Setip
            0x1C00..=0x1C7C => {
                error!("Want to read/write APLIC setip, not supported yet.");
            }
            // Setipnum
            0x1CDC => {
                error!("Want to read/write APLIC setipnum, not supported yet.");
            }
            // In_clrip
            0x1D00..=0x1D7C => {
                error!("Want to read/write APLIC in clrip, not supported yet.");
            }
            // Clripnum
            0x1DDC => {
                error!("Want to read/write APLIC clripnum, not supported yet.");
            }
            // Setie
            0x1E00..=0x1E7C => {
                error!("Want to read/write APLIC setie, not supported yet.");
            }
            // Setienum
            0x1EDC => {
                let irq_id = value as u32;
                let mut inner = self.inner.lock();
                if is_write {
                    if inner.hw[irq_id as usize] {
                        host_aplic().set_setienum(irq_id);
                        debug!("vAPLIC setienum for IRQ {} --> host APLIC", irq_id);
                        inner.vaplic_set_enable(irq_id as usize, true);
                    } else {
                        inner.vaplic_set_enable(irq_id as usize, true);
                    }
                } else {
                    warn!("Want to read APLIC setienum.");
                }
            }
            // Clrie
            0x1F00..=0x1F7C => {
                let irq_idx = (offset - 0x1F00) / 4;
                let mut inner = self.inner.lock();
                if is_write {
                    for irq_id in irq_idx * 32..(irq_idx + 1) * 32 {
                        if irq_id > self.max_interrupts {
                            break;
                        }
                        if (value & (1 << (irq_id - irq_idx * 32))) != 0 {
                            if inner.hw[irq_id] {
                                host_aplic().set_clrienum(irq_id as u32);
                                debug!("vAPLIC clrienum for IRQ {} --> host APLIC", irq_id);
                                inner.vaplic_set_enable(irq_id, false);
                            } else {
                                inner.vaplic_set_enable(irq_id, false);
                            }
                        }
                    }
                } else {
                    error!("Want to read APLIC clrie, not supported yet.");
                }
            }
            // Clrienum
            0x1FDC => {
                let irq_id = value as u32;
                let mut inner = self.inner.lock();
                if is_write {
                    if inner.hw[irq_id as usize] {
                        host_aplic().set_clrienum(irq_id);
                        debug!("vAPLIC clrienum for IRQ {} --> host APLIC", irq_id);
                        inner.vaplic_set_enable(irq_id as usize, false);
                    } else {
                        inner.vaplic_set_enable(irq_id as usize, false);
                    }
                } else {
                    warn!("Want to read APLIC clrienum.");
                }
            }
            // Setipnum_le
            0x2000 => {
                let irq_id = value as u32;
                let mut inner = self.inner.lock();
                if is_write {
                    if inner.hw[irq_id as usize] {
                        host_aplic().set_setipnum_le(irq_id);
                        inner.vaplic_set_pending(irq_id as usize, true);
                    } else {
                        inner.vaplic_set_pending(irq_id as usize, true);
                    }
                } else {
                    warn!("Want to read APLIC setipnum_le.");
                }
            }
            // Setipnum_be
            0x2004 => {
                error!("Want to read/write APLIC setipnum_be, not supported yet.");
            }
            // Genmsi
            0x3000 => {
                error!("Want to read/write APLIC genmsi, not supported yet.");
            }
            // Target configuration
            0x3004..=0x3FFC => {
                let irq_id = (offset - 0x3004) / 4 + 1;
                let mut inner = self.inner.lock();
                if is_write {
                    let hart_id = (value >> 18) & 0x3fff;
                    let phys_hart_id = hart_id
                        + this_cpu_data()
                            .zone
                            .as_ref()
                            .unwrap()
                            .read()
                            .cpu_set
                            .first_cpu()
                            .unwrap();
                    let guest_id = (value >> 12) & 0x3f;
                    let eiid = value & 0x7ff;
                    if inner.hw[irq_id] {
                        if !host_aplic().get_msimode() {
                            error!("hvisor's vAPLIC only supports MSI-mode");
                            return 0;
                        }
                        host_aplic().set_target_msi(
                            irq_id as u32,
                            phys_hart_id as u32,
                            GUEST_INDEX as u32,
                            eiid as u32,
                        );
                        info!(
                            "vAPLIC set target for IRQ {} to guest {}, hart {}, eiid {} -> host APLIC Guest {}",
                            irq_id, guest_id, hart_id, eiid, GUEST_INDEX
                        );
                        inner.vaplic_set_target(irq_id, value as u32);
                    } else {
                        debug!("Want to set target with no hw flag, irq: {}", irq_id);
                        inner.vaplic_set_target(irq_id, value as u32);
                    }
                } else {
                    info!("Read vAPLIC target for IRQ {}", irq_id);
                    return inner.vaplic_get_target(irq_id) as _;
                }
            }
            _ => panic!(
                "Invalid vAPLIC access at offset: {:#x}, size: {:#x}",
                offset, size
            ),
        }
        return 0;
    }
}

#[allow(unused)]
impl VirtualAPLICInner {
    /// vAPLIC get hardware interrupt.
    fn vaplic_get_hw(&self, intr_id: usize) -> bool {
        self.hw[intr_id]
    }

    /// vAPLIC set hardware interrupt.
    fn vaplic_set_hw(&mut self, intr_id: usize, hw: bool) {
        self.hw.set(intr_id, hw);
    }

    /// vAPLIC get active interrupt.
    fn vaplic_get_active(&self, intr_id: usize) -> bool {
        self.active[intr_id]
    }

    /// vAPLIC set active interrupt.
    fn vaplic_set_active(&mut self, intr_id: usize, active: bool) {
        self.active.set(intr_id, active);
    }

    /// vAPLIC get interrupt pending bit.
    fn vaplic_get_pending(&self, intr_id: usize) -> bool {
        self.pending[intr_id]
    }

    /// vAPLIC set interrupt pending bit.
    fn vaplic_set_pending(&mut self, intr_id: usize, pend: bool) {
        self.pending.set(intr_id, pend);
    }

    /// vAPLIC get enable bit.
    fn vaplic_get_enable(&self, intr_id: usize) -> bool {
        self.enable[intr_id]
    }

    /// vAPLIC set enable bit.
    fn vaplic_set_enable(&mut self, intr_id: usize, enable: bool) {
        self.enable.set(intr_id, enable);
    }

    /// vAPLIC get domain configuration.
    fn vaplic_get_domaincfg(&self) -> u32 {
        self.domaincfg
    }

    /// vAPLIC set domain configuration.
    fn vaplic_set_domaincfg(&mut self, domaincfg: u32) {
        self.domaincfg = domaincfg;
    }

    /// vAPLIC get source configuration.
    fn vaplic_get_sourcecfg(&self, intr_id: usize) -> u32 {
        self.srccfg[intr_id]
    }

    /// vAPLIC set source configuration.
    fn vaplic_set_sourcecfg(&mut self, intr_id: usize, srccfg: u32) {
        self.srccfg[intr_id] = srccfg;
    }

    /// vAPLIC get target.
    fn vaplic_get_target(&self, intr_id: usize) -> u32 {
        self.target[intr_id]
    }

    /// vAPLIC set target.
    fn vaplic_set_target(&mut self, intr_id: usize, target: u32) {
        self.target[intr_id] = target;
    }
}
