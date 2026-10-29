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
// Physical GICv3 interrupt entry: acknowledge, dispatch and deactivate.
use super::lr::{handle_maintenace_interrupt, inject_irq};
use crate::arch::aarch64::sysreg::{read_sysreg, write_sysreg};
use crate::event::{check_events, dispatch_host_events};
use crate::hypercall::SGI_IPI_ID;
use core::sync::atomic::AtomicU64;

static TIMER_INTERRUPT_COUNTER: AtomicU64 = AtomicU64::new(0);
// how often to print timer interrupt counter
const TIMER_INTERRUPT_PRINT_INTERVAL: u64 = 50;

pub fn gicv3_handle_irq_el1() {
    while let Some(irq_id) = pending_irq() {
        if irq_id < 16 {
            deactivate_irq(irq_id);
            if irq_id == SGI_IPI_ID as usize {
                // SGI 7 is the reserved physical host doorbell. Guest SGIs,
                // including guest SGI 7, are posted through software state.
                dispatch_host_events(|| while check_events() {});
            } else {
                warn!("unexpected physical SGI {}", irq_id);
            }
        } else {
            #[cfg(all(dwc_pcie, dwc_msi))]
            let mut is_dwc_msi_irq = false;

            if irq_id == 27 {
                // virtual timer interrupt
                TIMER_INTERRUPT_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
                if TIMER_INTERRUPT_COUNTER.load(core::sync::atomic::Ordering::SeqCst)
                    % TIMER_INTERRUPT_PRINT_INTERVAL
                    == 0
                {
                    trace!(
                        "Virtual timer interrupt, counter = {}",
                        TIMER_INTERRUPT_COUNTER.load(core::sync::atomic::Ordering::SeqCst)
                    );
                }
            } else if irq_id == 25 {
                // maintenace interrupt
                handle_maintenace_interrupt();
            } else if irq_id > 31 {
                //inject phy irq
                trace!("*** get spi_irq id = {}", irq_id);

                #[cfg(all(dwc_pcie, dwc_msi))]
                {
                    if let Some(domain_id) =
                        crate::pci::dwc_msi::get_domain_id_by_irq(irq_id as u32)
                    {
                        is_dwc_msi_irq = true;
                        crate::pci::dwc_msi::dwc_msi_transfer_and_inject(domain_id, irq_id);
                    }
                }
            } else {
                warn!("not konw irq id = {}", irq_id);
            }

            #[cfg(all(dwc_pcie, dwc_msi))]
            if irq_id != 25 && !is_dwc_msi_irq {
                inject_irq(irq_id, true);
            }

            #[cfg(not(all(dwc_pcie, dwc_msi)))]
            if irq_id != 25 {
                inject_irq(irq_id, true);
            }

            deactivate_irq(irq_id);
        }
    }
    trace!("handle done")
}

fn pending_irq() -> Option<usize> {
    let iar = read_sysreg!(icc_iar1_el1) as usize;
    if iar == 0x3ff {
        None
    } else {
        Some(iar as _)
    }
}

fn deactivate_irq(irq_id: usize) {
    write_sysreg!(icc_eoir1_el1, irq_id as u64);
    if irq_id < 16 || irq_id == 25 {
        write_sysreg!(icc_dir_el1, irq_id as u64);
    }
}
