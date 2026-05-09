
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

use core::usize::MAX;

const EIOINTC_IRQS: usize = 256;
const EIOINTC_ROUTE_MAX_VCPUS: usize = 256;
const LOONGSON_IP_NUM: usize = 8;

const EIOINTC_IRQS_U8_NUMS: usize = EIOINTC_IRQS / 8;
const EIOINTC_IRQS_U16_NUMS: usize = EIOINTC_IRQS_U8_NUMS / 2;
const EIOINTC_IRQS_U32_NUMS: usize = EIOINTC_IRQS_U8_NUMS / 4;
const EIOINTC_IRQS_U64_NUMS: usize = EIOINTC_IRQS_U8_NUMS / 8;
/* map to ipnum per 32 irqs */
const EIOINTC_IRQS_NODETYPE_COUNT: usize = 16;

const EIOINTC_NODETYPE_START: usize = 0xa0;
const EIOINTC_NODETYPE_END: usize = 0xbf;
const EIOINTC_IPMAP_START: usize = 0xc0;
const EIOINTC_IPMAP_END: usize = 0xc7;
const EIOINTC_ENABLE_START: usize = 0x200;
const EIOINTC_ENABLE_END: usize = 0x21f;
const EIOINTC_BOUNCE_START: usize = 0x280;
const EIOINTC_BOUNCE_END: usize = 0x29f;
const EIOINTC_ISR_START: usize = 0x300;
const EIOINTC_ISR_END: usize = 0x31f;
const EIOINTC_COREISR_START: usize = 0x400;
const EIOINTC_COREISR_END: usize = 0x41f;
const EIOINTC_COREMAP_START: usize = 0x800;
const EIOINTC_COREMAP_END: usize = 0x8ff;

pub const EIOINTC_BASE: usize = 0x1400;
// #define EIOINTC_REG_ROUTE    0x1c00

// 0x14a0 - 0x1400 = 0xa0  : nodetype
// 0x14c0 - 0x1400 = 0xc0  : ipmap
// 0x1680 - 0x1400 = 0x280 : bounce
// 0x1c00 - 0x1400 = 0x800 : coremap
// 0x1800 - 0x1400 = 0x400 : coreisr
// 0x1600 - 0x1400 = 0x200 : enable

pub const EIOINTC_SIZE: usize = 0x900;
pub const EIOINTC_VIRT_BASE: usize = 0x40000000;
pub const EIOINTC_VIRT_SIZE: usize = 0x1000;

pub const EIOINTC_ENABLE: usize = 1;
pub const EIOINTC_ENABLE_INT_ENCODE: usize = 2;
pub const EIOINTC_ENABLE_CPU_ENCODE: usize = 3;

macro_rules! bit {
    ($nr:expr) => {
        1usize << $nr
    };
}

const BITS_PER_ELEMENT: usize = 64;
const ARRAY_SIZE: usize = (EIOINTC_IRQS + BITS_PER_ELEMENT - 1) / BITS_PER_ELEMENT;
// the size of u64 array
type BitmapArray = [[[u64; ARRAY_SIZE]; LOONGSON_IP_NUM]; EIOINTC_ROUTE_MAX_VCPUS];

pub fn set_bit(bitmap: &mut BitmapArray, cpu: usize, ipum: usize, bit: usize) {
    if cpu < EIOINTC_ROUTE_MAX_VCPUS && ipum < LOONGSON_IP_NUM && bit < EIOINTC_IRQS {
        let word_index = bit / BITS_PER_ELEMENT;
        let bit_index = bit % BITS_PER_ELEMENT;
        bitmap[cpu][ipum][word_index] |= (1u64 << bit_index);
    } else {
        panic!("BitmapArray, set_bit, Index out of bounds");
    }
}

pub fn clear_bit(bitmap: &mut BitmapArray, cpu: usize, ipum: usize, bit: usize) {
    if cpu < EIOINTC_ROUTE_MAX_VCPUS && ipum < LOONGSON_IP_NUM && bit < EIOINTC_IRQS {
        let word_index = bit / BITS_PER_ELEMENT;
        let bit_index = bit % BITS_PER_ELEMENT;
        bitmap[cpu][ipum][word_index] &= !(1u64 << bit_index);
    } else {
        panic!("BitmapArray, clear_bit, Index out of bounds");
    }
}

pub fn find_first_bit(bitmap: &BitmapArray, cpu: usize, ipum: usize) -> Option<usize> {
    if cpu >= EIOINTC_ROUTE_MAX_VCPUS || ipum >= LOONGSON_IP_NUM {
        panic!("BitmapArray, find_first_bit, Index out of bounds");
    }

    for (word_index, &word) in bitmap[cpu][ipum].iter().enumerate() {
        if word != 0 {
            let bit_index = word.trailing_zeros() as usize;
            return Some(word_index * BITS_PER_ELEMENT + bit_index);
        }
    }
    None
}

#[derive(Debug)]
pub struct LoongArch64Eiointc {
    pub num_cpu: usize,
    pub features: usize,
    pub status: usize,
    pub nodetype: [u8; EIOINTC_IRQS_NODETYPE_COUNT * 2], // u8 * 32
    pub bounce: [u8; EIOINTC_IRQS_U8_NUMS], // 32
    pub isr: [u8; EIOINTC_IRQS_U8_NUMS], // 32
    pub coreisr: [[u8; EIOINTC_IRQS_U8_NUMS]; EIOINTC_ROUTE_MAX_VCPUS], // 32 * 256
    pub enable: [u8; EIOINTC_IRQS_U8_NUMS], // 32
    pub ipmap: [u8; EIOINTC_IRQS_U8_NUMS / 4], // 8
    pub coremap: [u8; EIOINTC_IRQS], // 256
    pub sw_coremap: [u8; EIOINTC_IRQS], // 256
    pub sw_coreisr: BitmapArray,// 256 * 8 * (4 * 64 bits)
}

impl LoongArch64Eiointc {
    pub fn new() -> Self {
        Self {
            num_cpu: 0,
            features: 0,
            status: 0,
            nodetype: [0; EIOINTC_IRQS_NODETYPE_COUNT * 2],
            bounce: [0; EIOINTC_IRQS_U8_NUMS],
            isr: [0; EIOINTC_IRQS_U8_NUMS],
            coreisr: [[0; EIOINTC_IRQS_U8_NUMS]; EIOINTC_ROUTE_MAX_VCPUS],
            enable: [0; EIOINTC_IRQS_U8_NUMS],
            ipmap: [0; EIOINTC_IRQS_U8_NUMS / 4],
            coremap: [0; EIOINTC_IRQS],
            sw_coremap: [0; EIOINTC_IRQS],
            sw_coreisr: [[[0; ARRAY_SIZE]; LOONGSON_IP_NUM]; EIOINTC_ROUTE_MAX_VCPUS],
        }
    }
}

pub fn do_real_write_iocsr(addr: usize, val: usize, len: usize) {
    match len {
        1 => {
            // iocsrwr.b
            unsafe {
                asm!("iocsrwr.b {}, {}", in(reg) val, in(reg) addr);
            }
        }
        2 => {
            // iocsrwr.h
            unsafe {
                asm!("iocsrwr.h {}, {}", in(reg) val, in(reg) addr);
            }
        }
        4 => {
            // iocsrwr.w
            unsafe {
                asm!("iocsrwr.w {}, {}", in(reg) val, in(reg) addr);
            }
        }
        8 => {
            // iocsrwr.d
            unsafe {
                asm!("iocsrwr.d {}, {}", in(reg) val, in(reg) addr);
            }
        }
        _ => {
            // should not reach here
            panic!("write invalid iocsr type, this is impossible");
        }
    }
}

pub fn do_real_read_iocsr(addr: usize, len: usize) -> usize {    
    match len {
        1 => {
            // iocsrrd.b
            let mut val= 0;
            unsafe {
                asm!("iocsrrd.b {}, {}", out(reg) val, in(reg) addr);
            }
            val
        }
        2 => {
            // iocsrrd.h
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.h {}, {}", out(reg) val, in(reg) addr);
            }
            val
        }
        4 => {
            // iocsrrd.w
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.w {}, {}", out(reg) val, in(reg) addr);
            }
            val
        }
        8 => {
            // iocsrrd.d
            let mut val = 0;
            unsafe {
                asm!("iocsrrd.d {}, {}", out(reg) val, in(reg) addr);
            }
            val
        }, 
        _ => {
            // should not reach here
            panic!("read invalid iocsr type, this is impossible");
        }
    }
}

pub fn read_masked_data(pbuf: *const u8, len: usize) -> usize {
    unsafe {
        match len {
            1 => {
                let byte_ptr = pbuf as *mut u8;
                *byte_ptr as usize
            }
            2 => {
                let short_ptr = pbuf as *mut u16;
                *short_ptr as usize
            }
            4 => {
                let int_ptr = pbuf as *mut u32;
                *int_ptr as usize
            }
            8 => {
                let long_ptr = pbuf as *mut u64;
                *long_ptr as usize
            }
            _ => {
                panic!("write_memory: invalid length {:#x}", len);
            }
        }    
    }
}

pub fn write_masked_data(pbuf: *mut u8, val: usize, len: usize) {
    unsafe {
        match len {
            1 => {
                let byte_ptr = pbuf as *mut u8;
                *byte_ptr = val as u8;
            }
            2 => {
                let short_ptr = pbuf as *mut u16;
                *short_ptr = val as u16;
            }
            4 => {
                let int_ptr = pbuf as *mut u32;
                *int_ptr = val as u32;
            }
            8 => {
                let long_ptr = pbuf as *mut u64;
                *long_ptr = val as u64;
            }
            _ => {
                warn!("write_memory: invalid length {:#x}", len);
            }
        }    
    }
}      
fn get_masked_data(data: usize, len: usize) -> usize {
    match len {
      1 => data & 0xff,
      2 => data & 0xffff,
      4 => data & 0xffffffff,
      8 => data,
      _ => {
          panic!("read_mailbox: unknown data len: {}", len);
      }
    }
}

// ffs和__ffs？
// 查找最低有效位（LSB）的1的位置
// ffs从1开始，__ffs从0开始
// trailing_zeros?
// 最低有效位（LSB，即最右边的位）开始连续 0 的个数
fn ffs(x: usize) -> usize {
    if x == 0 {
        0
    } else {
        x.trailing_zeros() as usize + 1
    }
}

fn __ffs(x: usize) -> usize {
    if x == 0 {
        0
    } else {
        x.trailing_zeros() as usize
    }
}

#[inline]
// ref from Linux kernel
fn count_trailing_zeros(x: usize, len: usize) -> i32 {
    const COUNT_TRAILING_ZEROS_0: i32 = -1;

    if len == 4 {
        ffs(x) as i32
    } else {
        if x != 0 {
            __ffs(x) as i32
        } else {
            COUNT_TRAILING_ZEROS_0
        }
    }
}


pub fn loongarch_eiointc_readl(pcpu_id: usize, addr: usize, len: usize) -> usize {
    let mut ret = 0;
    let offset = addr - EIOINTC_BASE;
    let pcpu_data = get_cpu_data(pcpu_id); 
    let eiointc = pcpu_data.arch_cpu.eiointc.lock();
    

    match offset {
        EIOINTC_NODETYPE_START..=EIOINTC_NODETYPE_END => {
            // nodetype
            let idx_u8 = (offset - EIOINTC_NODETYPE_START) ;
            let pbuf_read = &eiointc.nodetype[idx_u8];
            ret = read_masked_data(pbuf_read, len);
        }
        EIOINTC_IPMAP_START..=EIOINTC_IPMAP_END => {
            let idx_u8 = (offset - EIOINTC_IPMAP_START) ;
            let pbuf_read = &eiointc.ipmap[idx_u8];
            ret = read_masked_data(pbuf_read, len);
        }
        EIOINTC_ENABLE_START..=EIOINTC_ENABLE_END => {
            let idx_u8 = (offset - EIOINTC_ENABLE_START) ;
            let pbuf_read = &eiointc.enable[idx_u8];
            ret = read_masked_data(pbuf_read, len);
        }
        EIOINTC_BOUNCE_START..=EIOINTC_BOUNCE_END => {
            let idx_u8 = (offset - EIOINTC_BOUNCE_START) ;
            let pbuf_read = &eiointc.bounce[idx_u8];
            ret = read_masked_data(pbuf_read, len);
        }
        EIOINTC_COREISR_START..=EIOINTC_COREISR_END => {
            let idx_u8 = (offset - EIOINTC_COREISR_START) ;
 
            if pcpu_id == 4 {
                let offset = EIOINTC_BASE + EIOINTC_COREISR_START + 0 * EIOINTC_IRQS_U8_NUMS + idx_u8;
                ret = do_real_read_iocsr(offset, len);
            } else {
                let offset = EIOINTC_BASE + EIOINTC_COREISR_START + 0 * EIOINTC_IRQS_U8_NUMS + idx_u8;
                ret = do_real_read_iocsr(offset, len);
            }
        }
        EIOINTC_COREMAP_START..=EIOINTC_COREMAP_END => {
            let idx_u8 = (offset - EIOINTC_COREMAP_START) ;
            let pbuf_read = &eiointc.coremap[idx_u8];       
            ret = read_masked_data(pbuf_read, len);            
        }
        _ => {
            panic!(
                "loongarch_eiointc_readl, Invalid EIOINTC offset: {:#x}",
                offset
            );
        }
    };
    ret
}

use crate::cpu_data::{get_cpu_data, this_zone};
use crate::consts::MAX_CPU_NUM;
fn get_real_pcpu_id(target_cpu_id: usize) -> usize {
    assert!(target_cpu_id < MAX_CPU_NUM);
    let zone = this_zone();
    let cpu_set = zone.read().cpu_set();
    let target_pcpu_real_id = cpu_set.iter().nth(target_cpu_id).unwrap();
    target_pcpu_real_id
}

pub fn get_vcpuid_from_pcpuid(target_pcpu_id: usize) -> usize {
    assert!(target_pcpu_id < MAX_CPU_NUM);
    let zone = this_zone();
    let cpu_set = zone.read().cpu_set();
    let target_vcpuid = cpu_set.pcpuid_to_vcpuid(target_pcpu_id).unwrap();
    target_vcpuid
}

use crate::PHY_TO_DMW_UNCACHED;
const IOCSR_BASE:usize = 0x1fe00000;
const INT_HWI0: usize = 2;
fn eiointc_update_irq(eiointc: &mut spin::MutexGuard<'_, LoongArch64Eiointc>, irq: usize, level: usize) {

    let ipmap_u8_idx = irq / 32;
    let ipmap_pbuf_read = &eiointc.ipmap[ipmap_u8_idx];
    let mut ipnum = read_masked_data(ipmap_pbuf_read, 1); // u8
    
    if ((eiointc.status & bit!(EIOINTC_ENABLE_INT_ENCODE)) == 0) {
    
        let ipnum_new = count_trailing_zeros(ipnum, 1);
        if ipnum_new >= 4 {
            panic!("eiointc_update_irq, no ipnum found, irq: {}, level: {}", irq, level);
        }
        ipnum = (if ipnum_new >= 0 && ipnum_new < 4 {
            ipnum_new
        } else {
            0
        }) as usize;
    }

    let cpu = eiointc.sw_coremap[irq] as usize;
    let irq_index = irq / 32; // u32
    // 8 * 32 = 256

    let irq_mask = bit!(irq & 0x1f); // 1_1111 : 32 bits / 4 bytes
    let irq_index_u8 = irq_index * 4;

    let mut found;
    if (level != 0) {
        // set
        let enable_pbuf_read = &eiointc.enable[irq_index_u8];
        let enable_u32 = read_masked_data(enable_pbuf_read, 4);
        if ((enable_u32 & irq_mask) == 0) {
            return;
        }

        let coreisr_pbuf_read = &eiointc.coreisr[cpu][irq_index_u8];
        let coreisr_u32_new= read_masked_data(coreisr_pbuf_read, 4) | irq_mask;
    
        let mut check_data_before = 0;
        let mut check_data_after = 0;
        {
            let pbuf_read = &eiointc.coreisr[cpu][irq_index_u8];
            check_data_before = read_masked_data(pbuf_read, 4);
        }
        let coreisr_pbuf_write = &mut eiointc.coreisr[cpu][irq_index_u8];
        write_masked_data(coreisr_pbuf_write, coreisr_u32_new, 4);// TODO: do real write this
        {
            let pbuf_read = &eiointc.coreisr[cpu][irq_index_u8];
            check_data_after = read_masked_data(pbuf_read, 4);
        }

        found = find_first_bit(&eiointc.sw_coreisr, cpu, ipnum);
        set_bit(&mut eiointc.sw_coreisr, cpu, ipnum, irq);

    } else {
        // clear
        let coreisr_pbuf_read = &eiointc.coreisr[cpu][irq_index_u8];
        let coreisr_u32_new = read_masked_data(coreisr_pbuf_read, 4) & (!irq_mask);

        let mut check_data_before = 0;
        let mut check_data_after = 0;
        {
            let pbuf_read = &eiointc.coreisr[cpu][irq_index_u8];
            check_data_before = read_masked_data(pbuf_read, 4);
        }
        let coreisr_pbuf_write = &mut eiointc.coreisr[cpu][irq_index_u8];
        write_masked_data(coreisr_pbuf_write, coreisr_u32_new, 4);// TODO: do real write this
        {
            let pbuf_read = &eiointc.coreisr[cpu][irq_index_u8];
            check_data_after = read_masked_data(pbuf_read, 4);
        }

        clear_bit(&mut eiointc.sw_coreisr, cpu, ipnum, irq);
        found = find_first_bit(&eiointc.sw_coreisr, cpu, ipnum);

    }
    if (found.is_none()) {
    } else if (found.unwrap() < EIOINTC_IRQS) {
        return; /* other irq is handling, needn't update parent irq */
    }

    return;

    let int_hw_num = INT_HWI0 + ipnum;

    let target_pcpu_data = get_cpu_data(cpu); // check this carefully
    if (level != 0) {

    } else {

    }
}

pub fn eiointc_set_irq(pcpu_id: usize, irq: usize, level: usize) {        
    info!("eiointc_set_irq, pcpu_id: {}, irq: {}, level: {}", pcpu_id, irq, level);
    let pcpu_data = get_cpu_data(pcpu_id);
    let mut eiointc = pcpu_data.arch_cpu.eiointc.lock();
    
    let isr_bitmap_word = irq / 8;
    let isr_bitmap_offset = irq % 8;
    if level != 0 {
        eiointc.isr[isr_bitmap_word] |= bit!(isr_bitmap_offset) as u8;
        warn!("eiointc_set_irq, set, irq: {}", irq);
    } else {
        eiointc.isr[isr_bitmap_word] &= !bit!(isr_bitmap_offset) as u8;
        warn!("eiointc_set_irq, clear, irq: {}", irq);
    }
    eiointc_update_irq(&mut eiointc, irq, level);
}

fn eiointc_enable_irq(eiointc: &mut spin::MutexGuard<'_, LoongArch64Eiointc>, index: usize, mask: usize, level: usize) {
    let mut val = mask & 0xff;

    let mut irq = ffs(val);
    
    while (irq != 0) {
        eiointc_update_irq(eiointc, irq - 1 + index * 8, level);

        val &= !bit!(irq - 1);
        irq = ffs(val);
    }
}

use core::{arch::asm, char, panic, ptr::{read_volatile, write_volatile}, sync::atomic::AtomicU64};


fn eiointc_update_sw_coremap(eiointc: &mut spin::MutexGuard<'_, LoongArch64Eiointc>, irq: usize, pvalue: usize, len: usize, notify: bool) {

    let mut val = pvalue;
    for i in 0..len {
        let mut cpu = (val & 0xff);
        val >>= 8;

        if ((eiointc.status & bit!(EIOINTC_ENABLE_INT_ENCODE)) == 0) {
            cpu = __ffs(cpu);

            if cpu >= 4 {
                error!("eiointc_update_sw_coremap, attention 2, check it, cpu : {}", cpu);
            }
            cpu = if cpu >= 4 { 0 } else { cpu };

            assert!(cpu >= 0);
        }

        if (eiointc.sw_coremap[irq + i] == cpu as u8) {
            // warn!("eiointc_update_sw_coremap, continue, cpu: {}", cpu);
            continue;
        }

        let pcpu_id: usize = get_real_pcpu_id(cpu);
        if pcpu_id != cpu {
            warn!("[Attention], eiointc_update_sw_coremap, pcpu_id: {}, cpu: {}", 
                pcpu_id, cpu
            );
        }

        if (notify)
        {
            eiointc_update_irq(eiointc, irq + i, 0);// clear original irq
            
            eiointc.sw_coremap[irq + i] = pcpu_id as u8;
            eiointc.coremap[irq + i] = (1 << pcpu_id) as u8;// update it!

            eiointc_update_irq(eiointc, irq + i, 1);// set new irq
        } 
        else {
            panic!("notify is false, not tested");
            eiointc.sw_coremap[irq + i] = cpu as u8;
        }
    }
}

pub fn find_first_bit_single(val: usize, bits: usize) -> usize {
    assert!(bits == 8 || bits == 16 || bits == 32 || bits == 64);
    let masked_value = get_masked_data(val, bits / 8);
    let pos = ffs(masked_value);
    
    if pos == 0 {
        bits
    } else {
        (pos - 1) as usize
    }
}

pub fn loongarch_eiointc_writel(pcpu_id: usize, addr: usize, val: usize, len: usize) -> usize {

    let mut ret = val;
    let offset = addr - EIOINTC_BASE;

    let data = get_masked_data(val, len);
    
    let pcpu_data = get_cpu_data(pcpu_id);
    let mut eiointc = pcpu_data.arch_cpu.eiointc.lock();

    match offset {
        EIOINTC_NODETYPE_START..=EIOINTC_NODETYPE_END => {
            // nodetype
            let idx_u8 = (offset - EIOINTC_NODETYPE_START);

            let pbuf_write = &mut eiointc.nodetype[idx_u8];
        }
        EIOINTC_IPMAP_START..=EIOINTC_IPMAP_END => {
            // ipmap
            let idx_u8 = (offset - EIOINTC_IPMAP_START) ;
        
            let pbuf_write = &mut eiointc.ipmap[idx_u8];
            write_masked_data(pbuf_write, val, len);
        }
        EIOINTC_ENABLE_START..=EIOINTC_ENABLE_END => {
            // enable (important)
            let idx_u8 = (offset - EIOINTC_ENABLE_START) ;

            let enable_pbuf_read = &eiointc.enable[idx_u8];
            let old_enable = read_masked_data(enable_pbuf_read, len);
            
            let enable_pbuf_write = &mut eiointc.enable[idx_u8];
            write_masked_data(enable_pbuf_write, val, len) ;
            
            let enable_pbuf_read = &eiointc.enable[idx_u8];
            let enable_val= read_masked_data(enable_pbuf_read, len);

            do_real_write_iocsr(addr, val, len);
            
            return ret;
            /*
             * 1: enable irq.
             * update irq when isr is set.
             */
            let data = enable_val & !old_enable;
            info!("write enable, set, enable_val & !old_enable : {:#x}/{}", data, data.trailing_zeros());
            if old_enable != 0 {
                for i in 0..len {
                    let mask = (data >> (i * 8)) & 0xff;
                    eiointc_enable_irq(&mut eiointc, idx_u8 + i, mask, 1);
                }    
            }
            /*
             * 0: disable irq.
             * update irq when isr is set.
             */
            let data = !enable_val & old_enable;
            info!("write enable, clear, !enable_val & old_enable : {:#x}/{}", data, data.trailing_zeros());
            if old_enable != 0 {
                for i in 0..len {
                    let mask = (data >> (i * 8)) & 0xff;
                    eiointc_enable_irq(&mut eiointc, idx_u8 + i, mask, 0);
                }    
            }
        }
        EIOINTC_BOUNCE_START..=EIOINTC_BOUNCE_END => {
            // bounce
            let idx_u8 = (offset - EIOINTC_BOUNCE_START) ;
            
            let bounce_pbuf_write = &mut eiointc.bounce[idx_u8];
            write_masked_data(bounce_pbuf_write, val, len);
        }
        
        EIOINTC_COREISR_START..=EIOINTC_COREISR_END => {
            let idx_u8 = (offset - EIOINTC_COREISR_START);

            /* use attrs to get current cpu index */
            let cpu = pcpu_id;

            let mut coreisr = data;

            let coreisr_pbuf_read = &eiointc.coreisr[cpu][idx_u8];
            let old_coreisr = read_masked_data(coreisr_pbuf_read, len);
            
            /* write 1 to clear interrupt */
            let coreisr_new = coreisr & !old_coreisr;

            let coreisr_pbuf_write = &mut eiointc.coreisr[cpu][idx_u8];
            write_masked_data(coreisr_pbuf_write, coreisr_new, len); // TODO: do real write this

            coreisr &= old_coreisr;
            
            let bits = len * 8;// 4 * 8 = 32?
            let index = idx_u8 / len;
            
            let mut irq = find_first_bit_single(coreisr, bits);
            while (irq < bits) {    
                eiointc_update_irq(&mut eiointc, irq + index * bits, 0);

                // update coreisr and irq
                coreisr &= !bit!(irq);// clear irq from coreisr
                irq = find_first_bit_single(coreisr, bits);
            }
        }
        EIOINTC_COREMAP_START..=EIOINTC_COREMAP_END => {
            // coremap (important)
            let irq = offset - EIOINTC_COREMAP_START;
            let idx_u8 = irq;

            let coremap_pbuf_write = &mut eiointc.coremap[idx_u8];
            write_masked_data(coremap_pbuf_write, val, len);

            eiointc_update_sw_coremap(&mut eiointc, irq, val, len, true);

            let pbuf_read = &eiointc.coremap[idx_u8];
            let coremap_read = read_masked_data(pbuf_read, len);
            ret = coremap_read;// update!
        }
        _ => {
            panic!("eiointc_writel, Invalid EIOINTC offset: {:#x}", offset);
        }
    };

    do_real_write_iocsr(addr, ret, len);
    ret
}