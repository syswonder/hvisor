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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>
//
use crate::consts::IPI_EVENT_SEND_IPI;
#[cfg(feature = "plic")]
use crate::consts::IPI_EVENT_UPDATE_HART_LINE;
use crate::cpu_data::{get_cpu_data, this_cpu_data, CpuSet, VcpuState};
use crate::platform::BOARD_HARTID_MAP;

// arch_send_event
pub fn arch_send_event(cpu_id: u64, _sgi_num: u64) {
    let hart_id = BOARD_HARTID_MAP[cpu_id as usize];
    debug!("arch_send_event: cpu_id: {}", hart_id);
    #[cfg(feature = "aclint")]
    crate::device::irqchip::aclint::aclint_send_ipi(hart_id as usize);
    #[cfg(not(feature = "aclint"))]
    {
        let sbi_ret: sbi_rt::SbiRet =
            sbi_rt::send_ipi(sbi_rt::HartMask::from_mask_base(1 << hart_id, 0));
        if sbi_ret.is_err() {
            error!("arch_send_event: send_ipi failed: {:?}", sbi_ret);
        }
    }
}

/// Handle send_ipi event.
pub fn arch_ipi_handler() {
    unsafe {
        riscv_h::register::hvip::set_vssip();
    }
}

/// Handle hart suspend event.
pub fn arch_hart_suspend() {
    info!("cpu {} suspending...", this_cpu_data().id);
    this_cpu_data().vcpu_state.store(VcpuState::Blocked);
    loop {
        // TODO: use wfi to optimize the loop
        if !this_cpu_data().vcpu_state.is_blocked() {
            break;
        }
        core::hint::spin_loop();
    }
    // Remote sets `Ready` to leave Blocked; this hart then marks itself `Running` again.
    this_cpu_data().vcpu_state.store(VcpuState::Running);
    info!("cpu {} resumed from suspend.", this_cpu_data().id);
}

pub fn arch_check_events(event: Option<usize>) {
    match event {
        #[cfg(feature = "plic")]
        Some(IPI_EVENT_UPDATE_HART_LINE) => {
            use crate::device::irqchip::plic::update_hart_line;
            update_hart_line();
        }
        Some(IPI_EVENT_SEND_IPI) => {
            use crate::arch::riscv64::ipi::arch_ipi_handler;
            arch_ipi_handler();
        }
        _ => {
            panic!("arch_check_events: unhandled event: {:?}", event);
        }
    }
}

pub fn arch_prepare_send_event(_cpu_id: usize, _ipi_int_id: usize, _event_id: usize) {
    debug!("risc-v arch_prepare_send_event: do nothing now.")
}

/// Wait for other cpus in the cpu set to suspend.
#[allow(unused)]
pub fn wait_for_other_cpus_suspend(cpu_set: CpuSet) {
    let this_cpu_id = crate::arch::cpu::this_cpu_id();
    for target_cpu_id in cpu_set.iter() {
        if target_cpu_id == this_cpu_id {
            continue;
        }
        // Wait for the cpu to suspend.
        loop {
            if get_cpu_data(target_cpu_id).vcpu_state.is_blocked() {
                break;
            }
            core::hint::spin_loop();
        }
    }
}

/// Signal other cpus in the cpu set to resume.
#[allow(unused)]
pub fn signal_other_cpus_resume(cpu_set: CpuSet) {
    let this_cpu_id = crate::arch::cpu::this_cpu_id();
    for target_cpu_id in cpu_set.iter() {
        if target_cpu_id == this_cpu_id {
            continue;
        }
        get_cpu_data(target_cpu_id)
            .vcpu_state
            .store(VcpuState::Ready);
    }
}
