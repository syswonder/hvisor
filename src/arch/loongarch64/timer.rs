
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
//      Ming Shen  <boneinscri@163.com>
//

use core::sync::atomic::{AtomicU64, Ordering};

use loongArch64::{register::{tcfg, ticlr}, time};

use crate::{arch::{cpu::this_cpu_id, register::{read_csr_cntc, read_gcsr_estat, read_gcsr_tcfg, read_gcsr_tval, write_csr_cntc, write_gcsr_estat, write_gcsr_tcfg, write_gcsr_ticlr, write_gcsr_tval}, trap::{ecfg_timer_disable, ktime_get}, zone::ZoneContext}, cpu_data::get_cpu_data};

const CSR_TCFG_EN: usize = 1 << 0;
const CSR_TCFG_PERIOD_SHIFT: usize = 1;
const CSR_TCFG_PERIOD: usize = (1 << CSR_TCFG_PERIOD_SHIFT);
const CPU_TIMER: usize = (1 << 11);
const CSR_TINTCLR_TI: usize = 1 << 0;
const CSR_TCFG_VAL_SHIFT: usize = 2;
const CSR_TCFG_VAL: usize = 0x3fffffffffffusize << CSR_TCFG_VAL_SHIFT;
const INT_TI: usize = 11;

pub fn restore_timer(mut ctx: &mut ZoneContext, pcpu_id: usize) {
    // TODO: pcpu_id -> vcpu
    // TODO: if it supports vcpu, we should read gcsr from trap context
    let gcsr_tcfg = ctx.gcsr_tcfg;
    write_gcsr_tcfg(0);

    // TODO: restore gcsr.estat and gcsr.tcfg for vcpu
    write_gcsr_estat(ctx.gcsr_estat);
    write_gcsr_tcfg(ctx.gcsr_tcfg);

    if (gcsr_tcfg & CSR_TCFG_EN == 0) {
        /* Guest timer is disabled, just restore timer registers */
        // TODO: restore gcsr.tval
        write_gcsr_tval(ctx.gcsr_tval);
        return;
    }
    
    let gcsr_tval = ctx.gcsr_tval;
    let gcsr_estat = ctx.gcsr_estat;

    if ((gcsr_tcfg & CSR_TCFG_PERIOD == 0) && (gcsr_tval > gcsr_tcfg)) {
        write_gcsr_tval(0);
        if (gcsr_estat & CPU_TIMER == 0) {
            write_gcsr_ticlr(CSR_TINTCLR_TI);
        }
        return;
    }

    let mut delta = 0;
    let now = ktime_get();
    
    let pcpu_data = get_cpu_data(pcpu_id);
    let expire = pcpu_data.arch_cpu.expire as usize;

    if now < expire {
        delta = expire - now;
    } else if (gcsr_tcfg & CSR_TCFG_PERIOD != 0) {
        let period = gcsr_tcfg & CSR_TCFG_VAL;
        delta = now - expire;
        delta = period - (delta % period);
        
        // inject timer interrupt
        pcpu_data.arch_cpu.add_irq(INT_TI);
    }
    write_gcsr_tval(delta);
}

pub fn do_save_timer(mut ctx: &mut ZoneContext, pcpu_id: usize) {
    let mut delta = 0;
    // TODO: read gcsr.tcfg from trap context (from vcpu)
    let gcsr_tcfg = ctx.gcsr_tcfg;
    let gcsr_tval = ctx.gcsr_tval;   
    
    if (gcsr_tval < gcsr_tcfg) {
        delta = gcsr_tval;
    } else {
        delta = 0;
    }
    let expire = ktime_get() + delta;
    
    let pcpu_data = get_cpu_data(pcpu_id);
    pcpu_data.arch_cpu.expire = expire as isize;
}

pub fn save_timer(mut ctx: &mut ZoneContext, pcpu_id: usize) {
    // TODO: if it supports, pcpu_id -> vcpu
    // TODO: if it supports vcpu, we should read gcsr from trap context
    
    // TODO: save gcsr.tcfg and gcsr.tval for vcpu
    ctx.gcsr_tcfg = read_gcsr_tcfg();
    ctx.gcsr_tval = read_gcsr_tval();

    // TODO: read gcsr.tcfg from trap context
    let gcsr_tcfg = ctx.gcsr_tcfg;
    if (gcsr_tcfg & CSR_TCFG_EN != 0) {
        do_save_timer(ctx, pcpu_id);
    }

    // TODO: save gcsr.estat for vcpu
    ctx.gcsr_estat = read_gcsr_estat();
}


static INIT_OFFSET: AtomicU64 = AtomicU64::new(0);
static GLOBAL_TIMER: AtomicU64 = AtomicU64::new(0);

pub fn sync_counter()
{
    let init_offset_val = INIT_OFFSET.load(Ordering::Relaxed);
    write_csr_cntc(init_offset_val as usize);
}

pub fn timer_init() {
    const HZ: usize = 100;
    // uefi firmware leaves timer interrupt pending, we need to clear it manually
    ticlr::clear_timer_interrupt();
    // get timer frequency
    let timer_freq = time::get_timer_freq();

    let pcpu_id = this_cpu_id();
    if pcpu_id == 0 {
        let init_offset_val = -(ktime_get() as isize - read_csr_cntc() as isize);
        INIT_OFFSET.store(init_offset_val as u64, Ordering::Relaxed);    
    }
    sync_counter();

    ecfg_timer_disable();
    // 100_000_000
    // 1s = 1000 ms = 1000_000 us
    // set timer
    let init_val = timer_freq / HZ;
    tcfg::set_periodic(true);    
    tcfg::set_init_val(init_val);
    tcfg::set_en(true);// enable timer, not timer interrupt
}
