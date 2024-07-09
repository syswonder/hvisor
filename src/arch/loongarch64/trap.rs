use super::zone::ZoneContext;
use core::arch;
use core::arch::asm;
use core::panic;
use loongArch64::register::ecfg::LineBasedInterrupt;
use loongArch64::register::*;
use loongArch64::time;

pub fn install_trap_vector() {
    // force disable INT here
    crmd::set_ie(false);
    // clear UEFI firmware's previous timer configs
    tcfg::set_en(false);
    ticlr::clear_timer_interrupt();
    println!("loongarch64: force disabled interrupts");

    timer_init();
    // crmd::set_ie(true);

    // set CSR.EENTRY to _hyp_trap_vector and int vector offset to 0
    ecfg::set_vs(0);
    eentry::set_eentry(_hyp_trap_vector as usize);
    println!(
        "loongarch64: _hyp_trap_vector at 0x{:x}",
        _hyp_trap_vector as usize
    );
}

pub fn get_ms_counter(ms: usize) -> usize {
    ms * (time::get_timer_freq() / 1000)
}

pub fn get_us_counter(us: usize) -> usize {
    us * (time::get_timer_freq() / 1000_000)
}

pub fn timer_init() {
    // uefi firmware leaves timer interrupt pending, we need to clear it manually
    ticlr::clear_timer_interrupt();
    // get timer frequency
    let timer_freq = time::get_timer_freq();
    // 100_000_000
    // 1s = 1000 ms = 1000_000 us
    // set timer
    tcfg::set_periodic(true);
    // let init_val = get_ms_counter(500);
    let init_val = get_ms_counter(10000);
    tcfg::set_init_val(init_val);
    println!("loongarch64: timer_init: timer init value = {}", init_val);

    tcfg::set_en(true);

    let mut lie_ = ecfg::read().lie();
    lie_ = lie_ | LineBasedInterrupt::TIMER;
    ecfg::set_lie(lie_);
}

/// Translate exception code to string
pub fn ecode2str(ecode: usize, esubcode: usize) -> &'static str {
    match ecode {
        0x0 => "INT(Interrupt)",
        0x1 => "PIL(Page Illegal Load)",
        0x2 => "PIS(Page Illegal Store)",
        0x3 => "PIF(Page Illegal Fetch)",
        0x4 => "PME(Page Modify Exception)",
        0x5 => "PNR(Page Not Readable)",
        0x6 => "PNX(Page Not Executable)",
        0x7 => "PPI(Page Privilege Illegal)",
        0x8 => match esubcode {
            0x0 => "ADEF(Instruction Fetch Address Exception)",
            0x1 => "ADEM(Memory Access Address Exception)",
            _ => "error_esubcode",
        },
        0x9 => "ALE(Address Misaligned Exception)",
        0xa => "BCE(Edge Check Exception)",
        0xb => "SYS(System Call Exception)",
        0xc => "BRK(Breakpoint Exception)",
        0xd => "INE(Instruction Not Exist)",
        0xe => "IPE(Instruction Privilege Exception)",
        0xf => "FPD(Floating Point Disabled)",
        0x10 => "SXD(128-bit SIMD Disabled)",
        0x11 => "ASXD(256-bit SIMD Disabled)",
        0x12 => match esubcode {
            0x0 => "FPE(Floating Point Exception)",
            0x1 => "VFPE(Vector Floating Point Exception)",
            _ => "error_esubcode",
        },
        0x13 => match esubcode {
            0x0 => "WPEF(Watchpoint Exception Fetch)",
            0x1 => "WPEM(Watchpoint Exception Memory)",
            _ => "error_esubcode",
        },
        0x14 => "BTD(Binary Translation Disabled)",
        0x15 => "BTE(Binary Translation Exception)",
        0x16 => "GSPR(Guest Sensitive Privileged Resource)",
        0x17 => "HVC(Hypervisor Call)",
        0x18 => match esubcode {
            0x0 => "GCSC(Guest CSR Software Change)",
            0x1 => "GCHC(Guest CSR Hardware Change)",
            _ => "error_esubcode",
        },
        _ => "reserved_ecode",
    }
}

fn handle_page_modify_fault() {
    let badv_ = badv::read();
    info!(
        "(page_modify_fault) handling page modify exception, vaddr = 0x{:x}",
        badv_.vaddr()
    );
    info!("(page_modify_fault) ignoring this exception, todo: set dirty bit in page table entry");
}

#[no_mangle]
pub fn trap_handler(sp: usize) {
    let estat_ = estat::read();
    let ecode = estat_.ecode();
    let esubcode = estat_.esubcode();
    let is = estat_.is();
    let badv_ = badv::read();
    let badi_ = badi::read();
    debug!(
        "loongarch64: trap_handler: {}, ecode={:#x}, esubcode={:#x}, is={:#x}, badv=0x{:x}, badi=0x{:x}",
        ecode2str(ecode, esubcode),
        ecode,
        esubcode,
        is,
        badv_.vaddr(),
        badi_.inst()
    );
    ticlr::clear_timer_interrupt();
    _hyp_trap_return(sp);
}

#[no_mangle]
#[naked]
#[link_section = ".trap_entry"]
extern "C" fn _hyp_trap_vector() {
    unsafe {
        asm!(
          //traps from geust mode start here
          //in host mode, but with a vm page table
          //save guest $r3 to DESAVE, and get kernel $r3 from DESAVE

        //   "csrwr $r3, {LOONGARCH_CSR_DESAVE}",

          //parpare VmContext for zone_trap_handler
          //save 32 GPRS except $r3
          //save gcsrs managed by guest
          "addi.d $r3, $r3, -768",
          "st.d $r0, $r3, 0",
          "st.d $r1, $r3, 8",
          "st.d $r2, $r3, 16",
          "st.d $r4, $r3, 32",
          "st.d $r5, $r3, 40",
          "st.d $r6, $r3, 48",
          "st.d $r7, $r3, 56",
          "st.d $r8, $r3, 64",
          "st.d $r9, $r3, 72",
          "st.d $r10, $r3, 80",
          "st.d $r11, $r3, 88",
          "st.d $r12, $r3, 96",
          "st.d $r13, $r3, 104",
          "st.d $r14, $r3, 112",
          "st.d $r15, $r3, 120",
          "st.d $r16, $r3, 128",
          "st.d $r17, $r3, 136",
          "st.d $r18, $r3, 144",
          "st.d $r19, $r3, 152",
          "st.d $r20, $r3, 160",
          "st.d $r21, $r3, 168",
          "st.d $r22, $r3, 176",
          "st.d $r23, $r3, 184",
          "st.d $r24, $r3, 192",
          "st.d $r25, $r3, 200",
          "st.d $r26, $r3, 208",
          "st.d $r27, $r3, 216",
          "st.d $r28, $r3, 224",
          "st.d $r29, $r3, 232",
          "st.d $r30, $r3, 240",
          "st.d $r31, $r3, 248",
          // save ERA
          "csrrd $r12, {LOONGARCH_CSR_ERA}",
          "st.d $r12, $r3, 256",

          // save GCSRS
        //   "gcsrrd $r12, {LOONGARCH_GCSR_CRMD}",
        //   "st.d $r12, $r3, 256+8*1",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PRMD}",
        //   "st.d $r12, $r3, 256+8*2",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_EUEN}",
        //   "st.d $r12, $r3, 256+8*3",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_MISC}",
        //   "st.d $r12, $r3, 256+8*4",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_ECTL}",
        //   "st.d $r12, $r3, 256+8*5",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_ESTAT}",
        //   "st.d $r12, $r3, 256+8*6",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_ERA}",
        //   "st.d $r12, $r3, 256+8*7",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_BADV}",
        //   "st.d $r12, $r3, 256+8*8",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_BADI}",
        //   "st.d $r12, $r3, 256+8*9",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_EENTRY}",
        //   "st.d $r12, $r3, 256+8*10",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBIDX}",
        //   "st.d $r12, $r3, 256+8*11",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBEHI}",
        //   "st.d $r12, $r3, 256+8*12",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBELO0}",
        //   "st.d $r12, $r3, 256+8*13",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBELO1}",
        //   "st.d $r12, $r3, 256+8*14",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_ASID}",
        //   "st.d $r12, $r3, 256+8*15",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PGDL}",
        //   "st.d $r12, $r3, 256+8*16",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PGDH}",
        //   "st.d $r12, $r3, 256+8*17",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PGD}",
        //   "st.d $r12, $r3, 256+8*18",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PWCL}",
        //   "st.d $r12, $r3, 256+8*19",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PWCH}",
        //   "st.d $r12, $r3, 256+8*20",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_STLBPS}",
        //   "st.d $r12, $r3, 256+8*21",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_RAVCFG}",
        //   "st.d $r12, $r3, 256+8*22",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_CPUID}",
        //   "st.d $r12, $r3, 256+8*23",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG1}",
        //   "st.d $r12, $r3, 256+8*24",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG2}",
        //   "st.d $r12, $r3, 256+8*25",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_PRCFG3}",
        //   "st.d $r12, $r3, 256+8*26",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE0}",
        //   "st.d $r12, $r3, 256+8*27",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE1}",
        //   "st.d $r12, $r3, 256+8*28",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE2}",
        //   "st.d $r12, $r3, 256+8*29",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE3}",
        //   "st.d $r12, $r3, 256+8*30",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE4}",
        //   "st.d $r12, $r3, 256+8*31",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE5}",
        //   "st.d $r12, $r3, 256+8*32",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE6}",
        //   "st.d $r12, $r3, 256+8*33",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE7}",
        //   "st.d $r12, $r3, 256+8*34",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE8}",
        //   "st.d $r12, $r3, 256+8*35",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE9}",
        //   "st.d $r12, $r3, 256+8*36",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE10}",
        //   "st.d $r12, $r3, 256+8*37",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE11}",
        //   "st.d $r12, $r3, 256+8*38",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE12}",
        //   "st.d $r12, $r3, 256+8*39",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE13}",
        //   "st.d $r12, $r3, 256+8*40",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE14}",
        //   "st.d $r12, $r3, 256+8*41",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_SAVE15}",
        //   "st.d $r12, $r3, 256+8*42",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TID}",
        //   "st.d $r12, $r3, 256+8*43",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TCFG}",
        //   "st.d $r12, $r3, 256+8*44",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TVAL}",
        //   "st.d $r12, $r3, 256+8*45",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_CNTC}",
        //   "st.d $r12, $r3, 256+8*46",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TICLR}",
        //   "st.d $r12, $r3, 256+8*47",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_LLBCTL}",
        //   "st.d $r12, $r3, 256+8*48",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRENTRY}",
        //   "st.d $r12, $r3, 256+8*49",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRBADV}",
        //   "st.d $r12, $r3, 256+8*50",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRERA}",
        //   "st.d $r12, $r3, 256+8*51",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRSAVE}",
        //   "st.d $r12, $r3, 256+8*52",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRELO0}",
        //   "st.d $r12, $r3, 256+8*53",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRELO1}",
        //   "st.d $r12, $r3, 256+8*54",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBREHI}",
        //   "st.d $r12, $r3, 256+8*55",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_TLBRPRMD}",
        //   "st.d $r12, $r3, 256+8*56",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_DMW0}",
        //   "st.d $r12, $r3, 256+8*57",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_DMW1}",
        //   "st.d $r12, $r3, 256+8*58",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_DMW2}",
        //   "st.d $r12, $r3, 256+8*59",
        //   "gcsrrd $r12, {LOONGARCH_GCSR_DMW3}",
        //   "st.d $r12, $r3, 256+8*60",
        //   // now let's save the zone's pgd to ZoneContext
        //   "csrrd $r12, {LOONGARCH_CSR_PGDL}",
        //   "st.d $r12, $r3, 256+8*61", // PGDL
        //   "csrrd $r13, {LOONGARCH_CSR_PGDH}",
        //   "st.d $r13, $r3, 256+8*62", // PGDH
        //   // now let's switch KSAVE5 and KSAVE6, which should already
        //   // be set to kernel's pagetable base
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "csrwr $r13, {LOONGARCH_CSR_SAVE6}",
        //   "csrwr $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrwr $r13, {LOONGARCH_CSR_PGDH}",
        //   "invtlb 0, $r0, $r0",
        //   // save $r3 (previously saved in DESAVE)
        //   "csrrd $r12, {LOONGARCH_CSR_DESAVE}",
          "st.d $r12, $r3, 24",
          // $r3 -> a0, now the param of zone_trap_handler is ok
          "move $r4, $r3",
          "bl trap_handler",
        //   LOONGARCH_CSR_DESAVE = const 0x502,
          LOONGARCH_CSR_ERA = const 0x6,
        //   LOONGARCH_GCSR_CRMD = const 0x0,
        //   LOONGARCH_GCSR_PRMD = const 0x1,
        //   LOONGARCH_GCSR_EUEN = const 0x2,
        //   LOONGARCH_GCSR_MISC = const 0x3,
        //   LOONGARCH_GCSR_ECTL = const 0x4,
        //   LOONGARCH_GCSR_ESTAT = const 0x5,
        //   LOONGARCH_GCSR_ERA = const 0x6,
        //   LOONGARCH_GCSR_BADV = const 0x7,
        //   LOONGARCH_GCSR_BADI = const 0x8,
        //   LOONGARCH_GCSR_EENTRY = const 0xc,
        //   LOONGARCH_GCSR_TLBIDX = const 0x10,
        //   LOONGARCH_GCSR_TLBEHI = const 0x11,
        //   LOONGARCH_GCSR_TLBELO0 = const 0x12,
        //   LOONGARCH_GCSR_TLBELO1 = const 0x13,
        //   LOONGARCH_GCSR_ASID = const 0x18,
        //   LOONGARCH_GCSR_PGDL = const 0x19,
        //   LOONGARCH_GCSR_PGDH = const 0x1a,
        //   LOONGARCH_GCSR_PGD = const 0x1b,
        //   LOONGARCH_GCSR_PWCL = const 0x1c,
        //   LOONGARCH_GCSR_PWCH = const 0x1d,
        //   LOONGARCH_GCSR_STLBPS = const 0x1e,
        //   LOONGARCH_GCSR_RAVCFG = const 0x1f,
        //   LOONGARCH_GCSR_CPUID = const 0x20,
        //   LOONGARCH_GCSR_PRCFG1 = const 0x21,
        //   LOONGARCH_GCSR_PRCFG2 = const 0x22,
        //   LOONGARCH_GCSR_PRCFG3 = const 0x23,
        //   LOONGARCH_GCSR_SAVE0 = const 0x30,
        //   LOONGARCH_GCSR_SAVE1 = const 0x31,
        //   LOONGARCH_GCSR_SAVE2 = const 0x32,
        //   LOONGARCH_GCSR_SAVE3 = const 0x33,
        //   LOONGARCH_GCSR_SAVE4 = const 0x34,
        //   LOONGARCH_GCSR_SAVE5 = const 0x35,
        //   LOONGARCH_GCSR_SAVE6 = const 0x36,
        //   LOONGARCH_GCSR_SAVE7 = const 0x37,
        //   LOONGARCH_GCSR_SAVE8 = const 0x38,
        //   LOONGARCH_GCSR_SAVE9 = const 0x39,
        //   LOONGARCH_GCSR_SAVE10 = const 0x3a,
        //   LOONGARCH_GCSR_SAVE11 = const 0x3b,
        //   LOONGARCH_GCSR_SAVE12 = const 0x3c,
        //   LOONGARCH_GCSR_SAVE13 = const 0x3d,
        //   LOONGARCH_GCSR_SAVE14 = const 0x3e,
        //   LOONGARCH_GCSR_SAVE15 = const 0x3f,
        //   LOONGARCH_GCSR_TID = const 0x40,
        //   LOONGARCH_GCSR_TCFG = const 0x41,
        //   LOONGARCH_GCSR_TVAL = const 0x42,
        //   LOONGARCH_GCSR_CNTC = const 0x43,
        //   LOONGARCH_GCSR_TICLR = const 0x44,
        //   LOONGARCH_GCSR_LLBCTL = const 0x60,
        //   LOONGARCH_GCSR_TLBRENTRY = const 0x88,
        //   LOONGARCH_GCSR_TLBRBADV = const 0x89,
        //   LOONGARCH_GCSR_TLBRERA = const 0x8a,
        //   LOONGARCH_GCSR_TLBRSAVE = const 0x8b,
        //   LOONGARCH_GCSR_TLBRELO0 = const 0x8c,
        //   LOONGARCH_GCSR_TLBRELO1 = const 0x8d,
        //   LOONGARCH_GCSR_TLBREHI = const 0x8e,
        //   LOONGARCH_GCSR_TLBRPRMD = const 0x8f,
        //   LOONGARCH_GCSR_DMW0 = const 0x180,
        //   LOONGARCH_GCSR_DMW1 = const 0x181,
        //   LOONGARCH_GCSR_DMW2 = const 0x182,
        //   LOONGARCH_GCSR_DMW3 = const 0x183,
        //   LOONGARCH_CSR_PGDL = const 0x19,
        //   LOONGARCH_CSR_PGDH = const 0x1a,
        //   LOONGARCH_CSR_SAVE5 = const 0x35,
        //   LOONGARCH_CSR_SAVE6 = const 0x36,
        options(noreturn)
        );
    }
}

#[no_mangle]
extern "C" fn _hyp_trap_return(trap_addr: usize) {
    unsafe {
        asm!(
          // a0 -> sp
          "move  $r3, $r4",
          // restore ERA
          "ld.d $r12, $r3, 256",
          "csrwr $r12, {LOONGARCH_CSR_ERA}",
          // restore GCSRS
        //   "ld.d $r12, $r3, 256+8*1",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_CRMD}",
        //   "ld.d $r12, $r3, 256+8*2",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PRMD}",
        //   "ld.d $r12, $r3, 256+8*3",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_EUEN}",
        //   "ld.d $r12, $r3, 256+8*4",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_MISC}",
        //   "ld.d $r12, $r3, 256+8*5",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_ECTL}",
        //   "ld.d $r12, $r3, 256+8*6",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_ESTAT}",
        //   "ld.d $r12, $r3, 256+8*7",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_ERA}",
        //   "ld.d $r12, $r3, 256+8*8",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_BADV}",
        //   "ld.d $r12, $r3, 256+8*9",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_BADI}",
        //   "ld.d $r12, $r3, 256+8*10",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_EENTRY}",
        //   "ld.d $r12, $r3, 256+8*11",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBIDX}",
        //   "ld.d $r12, $r3, 256+8*12",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBEHI}",
        //   "ld.d $r12, $r3, 256+8*13",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBELO0}",
        //   "ld.d $r12, $r3, 256+8*14",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBELO1}",
        //   "ld.d $r12, $r3, 256+8*15",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_ASID}",
        //   "ld.d $r12, $r3, 256+8*16",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PGDL}",
        //   "ld.d $r12, $r3, 256+8*17",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PGDH}",
        //   "ld.d $r12, $r3, 256+8*18",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PGD}",
        //   "ld.d $r12, $r3, 256+8*19",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PWCL}",
        //   "ld.d $r12, $r3, 256+8*20",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PWCH}",
        //   "ld.d $r12, $r3, 256+8*21",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_STLBPS}",
        //   "ld.d $r12, $r3, 256+8*22",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_RAVCFG}",
        //   "ld.d $r12, $r3, 256+8*23",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_CPUID}",
        //   "ld.d $r12, $r3, 256+8*24",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG1}",
        //   "ld.d $r12, $r3, 256+8*25",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG2}",
        //   "ld.d $r12, $r3, 256+8*26",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_PRCFG3}",
        //   "ld.d $r12, $r3, 256+8*27",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE0}",
        //   "ld.d $r12, $r3, 256+8*28",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE1}",
        //   "ld.d $r12, $r3, 256+8*29",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE2}",
        //   "ld.d $r12, $r3, 256+8*30",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE3}",
        //   "ld.d $r12, $r3, 256+8*31",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE4}",
        //   "ld.d $r12, $r3, 256+8*32",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE5}",
        //   "ld.d $r12, $r3, 256+8*33",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE6}",
        //   "ld.d $r12, $r3, 256+8*34",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE7}",
        //   "ld.d $r12, $r3, 256+8*35",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE8}",
        //   "ld.d $r12, $r3, 256+8*36",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE9}",
        //   "ld.d $r12, $r3, 256+8*37",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE10}",
        //   "ld.d $r12, $r3, 256+8*38",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE11}",
        //   "ld.d $r12, $r3, 256+8*39",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE12}",
        //   "ld.d $r12, $r3, 256+8*40",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE13}",
        //   "ld.d $r12, $r3, 256+8*41",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE14}",
        //   "ld.d $r12, $r3, 256+8*42",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_SAVE15}",
        //   "ld.d $r12, $r3, 256+8*43",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TID}",
        //   "ld.d $r12, $r3, 256+8*44",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TCFG}",
        //   "ld.d $r12, $r3, 256+8*45",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TVAL}",
        //   "ld.d $r12, $r3, 256+8*46",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_CNTC}",
        //   "ld.d $r12, $r3, 256+8*47",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TICLR}",
        //   "ld.d $r12, $r3, 256+8*48",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_LLBCTL}",
        //   "ld.d $r12, $r3, 256+8*49",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRENTRY}",
        //   "ld.d $r12, $r3, 256+8*50",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRBADV}",
        //   "ld.d $r12, $r3, 256+8*51",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRERA}",
        //   "ld.d $r12, $r3, 256+8*52",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRSAVE}",
        //   "ld.d $r12, $r3, 256+8*53",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRELO0}",
        //   "ld.d $r12, $r3, 256+8*54",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRELO1}",
        //   "ld.d $r12, $r3, 256+8*55",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBREHI}",
        //   "ld.d $r12, $r3, 256+8*56",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_TLBRPRMD}",
        //   "ld.d $r12, $r3, 256+8*57",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_DMW0}",
        //   "ld.d $r12, $r3, 256+8*58",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_DMW1}",
        //   "ld.d $r12, $r3, 256+8*59",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_DMW2}",
        //   "ld.d $r12, $r3, 256+8*60",
        //   "gcsrwr $r12, {LOONGARCH_GCSR_DMW3}",
          LOONGARCH_CSR_ERA = const 0x6,
        //   LOONGARCH_GCSR_CRMD = const 0x0,
        //   LOONGARCH_GCSR_PRMD = const 0x1,
        //   LOONGARCH_GCSR_EUEN = const 0x2,
        //   LOONGARCH_GCSR_MISC = const 0x3,
        //   LOONGARCH_GCSR_ECTL = const 0x4,
        //   LOONGARCH_GCSR_ESTAT = const 0x5,
        //   LOONGARCH_GCSR_ERA = const 0x6,
        //   LOONGARCH_GCSR_BADV = const 0x7,
        //   LOONGARCH_GCSR_BADI = const 0x8,
        //   LOONGARCH_GCSR_EENTRY = const 0xc,
        //   LOONGARCH_GCSR_TLBIDX = const 0x10,
        //   LOONGARCH_GCSR_TLBEHI = const 0x11,
        //   LOONGARCH_GCSR_TLBELO0 = const 0x12,
        //   LOONGARCH_GCSR_TLBELO1 = const 0x13,
        //   LOONGARCH_GCSR_ASID = const 0x18,
        //   LOONGARCH_GCSR_PGDL = const 0x19,
        //   LOONGARCH_GCSR_PGDH = const 0x1a,
        //   LOONGARCH_GCSR_PGD = const 0x1b,
        //   LOONGARCH_GCSR_PWCL = const 0x1c,
        //   LOONGARCH_GCSR_PWCH = const 0x1d,
        //   LOONGARCH_GCSR_STLBPS = const 0x1e,
        //   LOONGARCH_GCSR_RAVCFG = const 0x1f,
        //   LOONGARCH_GCSR_CPUID = const 0x20,
        //   LOONGARCH_GCSR_PRCFG1 = const 0x21,
        //   LOONGARCH_GCSR_PRCFG2 = const 0x22,
        //   LOONGARCH_GCSR_PRCFG3 = const 0x23,
        //   LOONGARCH_GCSR_SAVE0 = const 0x30,
        //   LOONGARCH_GCSR_SAVE1 = const 0x31,
        //   LOONGARCH_GCSR_SAVE2 = const 0x32,
        //   LOONGARCH_GCSR_SAVE3 = const 0x33,
        //   LOONGARCH_GCSR_SAVE4 = const 0x34,
        //   LOONGARCH_GCSR_SAVE5 = const 0x35,
        //   LOONGARCH_GCSR_SAVE6 = const 0x36,
        //   LOONGARCH_GCSR_SAVE7 = const 0x37,
        //   LOONGARCH_GCSR_SAVE8 = const 0x38,
        //   LOONGARCH_GCSR_SAVE9 = const 0x39,
        //   LOONGARCH_GCSR_SAVE10 = const 0x3a,
        //   LOONGARCH_GCSR_SAVE11 = const 0x3b,
        //   LOONGARCH_GCSR_SAVE12 = const 0x3c,
        //   LOONGARCH_GCSR_SAVE13 = const 0x3d,
        //   LOONGARCH_GCSR_SAVE14 = const 0x3e,
        //   LOONGARCH_GCSR_SAVE15 = const 0x3f,
        //   LOONGARCH_GCSR_TID = const 0x40,
        //   LOONGARCH_GCSR_TCFG = const 0x41,
        //   LOONGARCH_GCSR_TVAL = const 0x42,
        //   LOONGARCH_GCSR_CNTC = const 0x43,
        //   LOONGARCH_GCSR_TICLR = const 0x44,
        //   LOONGARCH_GCSR_LLBCTL = const 0x60,
        //   LOONGARCH_GCSR_TLBRENTRY = const 0x88,
        //   LOONGARCH_GCSR_TLBRBADV = const 0x89,
        //   LOONGARCH_GCSR_TLBRERA = const 0x8a,
        //   LOONGARCH_GCSR_TLBRSAVE = const 0x8b,
        //   LOONGARCH_GCSR_TLBRELO0 = const 0x8c,
        //   LOONGARCH_GCSR_TLBRELO1 = const 0x8d,
        //   LOONGARCH_GCSR_TLBREHI = const 0x8e,
        //   LOONGARCH_GCSR_TLBRPRMD = const 0x8f,
        //   LOONGARCH_GCSR_DMW0 = const 0x180,
        //   LOONGARCH_GCSR_DMW1 = const 0x181,
        //   LOONGARCH_GCSR_DMW2 = const 0x182,
        //   LOONGARCH_GCSR_DMW3 = const 0x183,
        );
        // asm!(
        //   // vm-pagetable -> save5 and save6
        //   "ld.d $r12, $r3, 256+8*61",
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "ld.d $r12, $r3, 256+8*62",
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE6}",
        //   // kernel-pagetable -> r12 and r13
        //   "csrrd $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrrd $r13, {LOONGARCH_CSR_PGDH}",
        //   // kernel_pagetable -> save5 and save6
        //   // old save5/save6(vm_pagetable) -> r12/r13
        //   "csrwr $r12, {LOONGARCH_CSR_SAVE5}",
        //   "csrwr $r13, {LOONGARCH_CSR_SAVE6}",
        //   // change pagetable from kernel pagetable to vm page table
        //   "csrwr $r12, {LOONGARCH_CSR_PGDL}",
        //   "csrwr $r13, {LOONGARCH_CSR_PGDH}",
        //   "invtlb 0, $r0, $r0",
        //   LOONGARCH_CSR_SAVE5 = const 0x35,
        //   LOONGARCH_CSR_SAVE6 = const 0x36,
        //   LOONGARCH_CSR_PGDL = const 0x19,
        //   LOONGARCH_CSR_PGDH = const 0x1a,
        // );
        asm!(
          // restore sp
          "ld.d $r12, $r3, 24",
          "csrwr $r12, {LOONGARCH_CSR_DESAVE}",
          // restore 32 GPRS:
          "ld.d $r0, $r3, 0",
          "ld.d $r1, $r3, 8",
          "ld.d $r2, $r3, 16",
          //ld.d $r3, $r3, 24
          "ld.d $r4, $r3, 32",
          "ld.d $r5, $r3, 40",
          "ld.d $r6, $r3, 48",
          "ld.d $r7, $r3, 56",
          "ld.d $r8, $r3, 64",
          "ld.d $r9, $r3, 72",
          "ld.d $r10, $r3, 80",
          "ld.d $r11, $r3, 88",
          "ld.d $r12, $r3, 96",
          "ld.d $r13, $r3, 104",
          "ld.d $r14, $r3, 112",
          "ld.d $r15, $r3, 120",
          "ld.d $r16, $r3, 128",
          "ld.d $r17, $r3, 136",
          "ld.d $r18, $r3, 144",
          "ld.d $r19, $r3, 152",
          "ld.d $r20, $r3, 160",
          "ld.d $r21, $r3, 168",
          "ld.d $r22, $r3, 176",
          "ld.d $r23, $r3, 184",
          "ld.d $r24, $r3, 192",
          "ld.d $r25, $r3, 200",
          "ld.d $r26, $r3, 208",
          "ld.d $r27, $r3, 216",
          "ld.d $r28, $r3, 224",
          "ld.d $r29, $r3, 232",
          "ld.d $r30, $r3, 240",
          "ld.d $r31, $r3, 248",
          // restore stack(16-byte alignment)
          "addi.d $r3, $r3, 768",
          LOONGARCH_CSR_DESAVE = const 0x502
        );
        asm!(
            // now, we can store kernel stack $r3 to DESAVE
            //   "csrwr $r3, {LOONGARCH_CSR_DESAVE}",
            // ret to guest mode using ertn
            "ertn",
            //   LOONGARCH_CSR_DESAVE = const 0x502
        );
    }
}

fn extract_field(inst: usize, offset: usize, length: usize) -> usize {
    let mask = (1 << length) - 1;
    (inst >> offset) & mask
}

/// get the sign-extended imm12 to i64
fn imm12toi64(imm12: usize) -> isize {
    let imm12 = imm12 as isize;
    let imm12 = imm12 << 52;
    imm12 >> 52
}

const IPI_BIT: usize = 1 << 12;
const TIMER_BIT: usize = 1 << 11;

fn handle_interrupt(is: usize) {
    match is {
        _ if is & IPI_BIT != 0 => {
            info!("ipi interrupt");
        }
        _ if is & TIMER_BIT != 0 => {
            loongArch64::register::ticlr::clear_timer_interrupt();
        }
        _ => {
            info!("not handled interrupt");
        }
    }
}

fn handle_hvc(ctx: &mut ZoneContext) {
    // HVC
    // hvcl's code should always be 0! we use a7 as hvc call code
    // this convention should be followed by the guest os to properly use HVC call - wheatfox
    // and a0 to a6 are the arguments, a0 is the return val
    let hvc_id = ctx.get_a7();

    info!("HVC exception, HVC call code: {:#x}", hvc_id);
    // let retval = crate::hypercall::_hypercall(ctx, hvc_id);
    // let retval = crate::hypercall::hypercall(hvc_id, [ctx.get_a0(), ctx.get_a1(), ctx.get_a2()]);
    // ctx.set_a0(retval);
    ctx.sepc += 4;
    // jump to next instruction
}

fn emulate_cpucfg(ins: usize, ctx: &mut ZoneContext) {
    // cpucfg
    // now let get rd and rj, cpucfg rd[4:0], rj[9:5]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let cpucfg_target_idx = ctx.x[rj];
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let cpucfg_target_idx = ctx.x[rj];

    const KVM_MAX_CPUCFG_REGS: usize = 21;

    info!(
        "cpucfg emulation, target cpucfg index is {:#x}",
        cpucfg_target_idx
    );

    if cpucfg_target_idx >= KVM_MAX_CPUCFG_REGS {
        // invalid cpucfg target

        warn!("invalid cpucfg target");
        ctx.x[rd] = 0;
        // according to manual, we should set result to 0 if index is invalid
    } else {
        // just run cpucfg here
        let result: usize;
        unsafe {
            asm!("cpucfg {}, {}", out(reg) result, in(reg) cpucfg_target_idx);
        }
        ctx.x[rd] = result;
        // finish the emulation by tweaking the ZoneContext's registers
        // as ctx.sepc is already added by 4 which means we will jump to next instruction - wheatfox
    }
}

fn emulate_csrx(ins: usize, ctx: &mut ZoneContext) {
    // csrrd csrwr csrxchg

    // let ty = (ins >> 5) & 0x1f;
    // let rd = ins & 0x1f;
    // let csr = (ins >> 10) & 0x3fff;
    let ty = extract_field(ins, 5, 5);
    let rd = extract_field(ins, 0, 5);
    let csr = extract_field(ins, 10, 14);
    // ty: [9:5], 0 - csrrd, 1 - csrwr, else - csrxchg
    // rd [4:0]
    // csr [23:10] 14 bits
    match ty {
        0 => {
            // csrrd

            info!("csrrd emulation for CSR {:#x}", csr);
            ctx.x[rd] = 0;
            // just set it to 0
        }
        1 => {
            // csrwr

            info!("csrwr emulation for CSR {:#x}", csr);
            ctx.x[rd] = 0;
            // do nothing to GCSR, but we also need to set rd to 0
        }
        _ => {
            // csrxchg

            info!("csrxchg emulation for CSR {:#x}", csr);
            ctx.x[rd] = 0;
            // do nothing to GCSR, but we also need to set rd to 0
        }
    }
}

fn emulate_cacop(ins: usize, ctx: &mut ZoneContext) {
    // cacop code,rj,si12   0000011000 si12 rj[9:5] code[4:0]
    warn!("cacop emulation not implemented, skipped this instruction");
}

fn emulate_idle(ins: usize, ctx: &mut ZoneContext) {
    // idle level           0000011001 0010001 level[14:0]
    let level = extract_field(ins, 0, 15);
    warn!("guest request an idle at level {:#x}", level);
    ctx.sepc -= 4;
}

fn emulate_iocsr(ins: usize, ctx: &mut ZoneContext) {
    // iocsrrd.b rd, rj     0000011001 001000000000 rj[9:5] rd[4:0]
    // iocsrrd.h rd, rj     0000011001 001000000001 rj[9:5] rd[4:0]
    // iocsrrd.w rd, rj     0000011001 001000000010 rj[9:5] rd[4:0]
    // iocsrrd.d rd, rj     0000011001 001000000011 rj[9:5] rd[4:0]
    // iocsrwr.b rd, rj     0000011001 001000000100 rj[9:5] rd[4:0]
    // iocsrwr.h rd, rj     0000011001 001000000101 rj[9:5] rd[4:0]
    // iocsrwr.w rd, rj     0000011001 001000000110 rj[9:5] rd[4:0]
    // iocsrwr.d rd, rj     0000011001 001000000111 rj[9:5] rd[4:0]
    // let ty = (ins >> 10) & 0x7;
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    let ty = extract_field(ins, 10, 3);
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    info!("iocsr emulation, ty = {}, rd = {}, rj = {}", ty, rd, rj);
    info!("GPR[rd] = {:#x}, GPR[rj] = {:#x}", ctx.x[rd], ctx.x[rj]);
    warn!("iocsr emulation not enabled for debugging purpose(just for now)");
    panic!("wait!!!");
}

const UART0_BASE: usize = 0x1fe001e0;
const UART0_END: usize = 0x1fe001e8;

fn emulate_ld_b(ins: usize, ctx: &mut ZoneContext) {
    // ld.b   rd, rj, si12  opcode[31:22]=0010100000 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff; ??? should be 0xfff
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);

    info!("ld.b emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // byte = load (paddr, BYTE)
    // GR[rd] = byte
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
                                                         // let mut uart0 = UART_EMU.lock();
                                                         // let byte = uart0.read(offset);
                                                         // info!("byte = 0x{:x}", byte as usize);
                                                         // ctx.x[rd] = byte as usize;
}

fn emulate_st_b(ins: usize, ctx: &mut ZoneContext) {
    // st.b   rd, rj, si12  opcode[31:22]=0010100100 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff;
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);
    // info!("st.b emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // store (paddr, BYTE, GR[rd])
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    // info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
                                                         // for VGA
                                                         // let mut uart0 = UART_EMU.lock();
                                                         // let byte = ctx.x[rd] as u8;
                                                         // info!("byte = 0x{:x}", byte as usize);
                                                         // let cur_zone = current_vcpu().unwrap().get_zone().unwrap();
                                                         // let cur_zone_id = cur_zone.get_zone_id();
                                                         // uart0.write(offset, byte, false, (cur_zone_id - 1) as i32);
                                                         // drop(uart0); // !!!! very important
                                                         // cur_zone.inner.lock().uart_emu.write(offset, byte, true, 0);
}

fn emulate_ld_bu(ins: usize, ctx: &mut ZoneContext) {
    // ld.bu  rd, rj, si12  opcode[31:22]=0010101000 si12[21:10] rj[9:5] rd[4:0]
    // let rd = ins & 0x1f;
    // let rj = (ins >> 5) & 0x1f;
    // let si12 = (ins >> 10) & 0x3ff;
    let rd = extract_field(ins, 0, 5);
    let rj = extract_field(ins, 5, 5);
    let si12 = extract_field(ins, 10, 12);

    // info!("ld.bu emulation, rd = {}, rj = {}, si12 = {}", rd, rj, si12);
    // vaddr = GR[rj] + SignExt(si12, GRLEN(64))
    // paddr = translate(vaddr)
    // byte = load (paddr, BYTE)
    // GR[rd] = byte
    let vaddr = ctx.x[rj] as isize + imm12toi64(si12);
    // info!("vaddr = 0x{:x}", vaddr as usize);
    let offset = (vaddr - UART0_BASE as isize) as usize; // minus the UART0 base address
                                                         // let mut uart0 = UART_EMU.lock();
                                                         // let byte = uart0.read(offset);
                                                         // info!("byte = 0x{:x}", byte as usize);
                                                         // ctx.x[rd] = byte as usize;
}

fn check_op_type(inst: usize, opcode: usize, opcode_length: usize) -> bool {
    let mask = (1 << opcode_length) - 1;
    let shifted = inst >> (32 - opcode_length);
    (shifted & mask) == opcode
}

const OPCODE_CPUCFG: usize = 0b0000000000000000011011;
const OPCODE_CPUCFG_LENGTH: usize = 22;
const OPCODE_CACOP: usize = 0b0000011000;
const OPCODE_CACOP_LENGTH: usize = 10;
const OPCODE_IDLE: usize = 0b00000_11001_0010001;
const OPCODE_IDLE_LENGTH: usize = 17;
const OPCODE_CSRX: usize = 0b00000100;
const OPCODE_CSRX_LENGTH: usize = 8;
const OPCODE_IOCSR: usize = 0b00000_11001_001000000;
const OPCODE_IOCSR_LENGTH: usize = 19;
const OPCODE_LD_B: usize = 0b0010100000;
const OPCODE_LD_B_LENGTH: usize = 10;
const OPCODE_ST_B: usize = 0b0010100100;
const OPCODE_ST_B_LENGTH: usize = 10;
const OPCODE_LD_BU: usize = 0b0010101000;
const OPCODE_LD_BU_LENGTH: usize = 10;
type OpcodeHandler = fn(usize, &mut ZoneContext);

fn emulate_instruction(era: usize, ins: usize, ctx: &mut ZoneContext) {
    let pc = era;
    ctx.sepc = pc + 4;

    // after we emulate the instruction, we should jump to next instruction
    let opcodes = vec![
        (
            OPCODE_CPUCFG,
            OPCODE_CPUCFG_LENGTH,
            emulate_cpucfg as OpcodeHandler,
        ),
        (
            OPCODE_CACOP,
            OPCODE_CACOP_LENGTH,
            emulate_cacop as OpcodeHandler,
        ),
        (
            OPCODE_IDLE,
            OPCODE_IDLE_LENGTH,
            emulate_idle as OpcodeHandler,
        ),
        (
            OPCODE_CSRX,
            OPCODE_CSRX_LENGTH,
            emulate_csrx as OpcodeHandler,
        ),
        (
            OPCODE_IOCSR,
            OPCODE_IOCSR_LENGTH,
            emulate_iocsr as OpcodeHandler,
        ),
        (
            OPCODE_LD_B,
            OPCODE_LD_B_LENGTH,
            emulate_ld_b as OpcodeHandler,
        ),
        (
            OPCODE_ST_B,
            OPCODE_ST_B_LENGTH,
            emulate_st_b as OpcodeHandler,
        ),
        (
            OPCODE_LD_BU,
            OPCODE_LD_BU_LENGTH,
            emulate_ld_bu as OpcodeHandler,
        ),
    ];
    for &(code, length, handler) in &opcodes {
        if check_op_type(ins, code, length) {
            handler(ins, ctx);
            return;
        }
    }

    error!("Unexpected opcode encountered, ins = {:#x}", ins);
    loop {}
}

const ECODE_INT: usize = 0x0;
const ECODE_GSPR: usize = 0x16;
const ECODE_PIL: usize = 0x1;
const ECODE_PIS: usize = 0x2;
const ECODE_HVC: usize = 0x17;

fn handle_exception(
    ecode: usize,
    era: usize,
    is: usize,
    badi: usize,
    badv: usize,
    ctx: &mut ZoneContext,
) {
    match ecode {
        ECODE_INT => {
            // INT = 0x0,   Interrupt
            handle_interrupt(is);
        }
        ECODE_GSPR => {
            // according to kvm's code, we should emulate the instruction that cause the GSPR exception - wheatfox 2024.4.12
            // GSPR = 0x16, Guest Sensitive Privileged Resource
            info!("GSPR exception");
            emulate_instruction(era, badi, ctx);
        }
        ECODE_PIL | ECODE_PIS => {
            // PIL = 0x1,   Page Illegal Load
            // PIS = 0x2,   Page Illegal Store
            // info!("handling page invalid exception...");
            if badv >= UART0_BASE && badv < UART0_END {
                // info!("handling UART0 mmio emulation");
                emulate_instruction(era, badi, ctx);
            } else {
                if ecode == ECODE_PIL {
                    error!("Page Illegal Load");
                } else {
                    error!("Page Illegal Store");
                }
                loop {}
            }
        }
        ECODE_HVC => {
            // HVC = 0x17,  Hypervisor Call
            info!("handling HVC exception...");
            handle_hvc(ctx);
        }
        _ => {
            error!(
          "unhandled exception, ecode = {:#x}, era = {:#x}, is = {:#x}, badi = {:#x}, badv = {:#x}",
          ecode, era, is, badi, badv
        );
            loop {}
        }
    }
}

/* TLB REFILL HANDLER */
#[no_mangle]
#[naked]
#[link_section = ".tlbrefill_entry"]
extern "C" fn tlb_refill_handler() {
    unsafe {
        asm!(
        "csrwr      $r12, {LOONGARCH_CSR_TLBRSAVE}",
        "csrrd      $r12, {LOONGARCH_CSR_PGD}",
        "lddir      $r12, $r12, 3",
        "lddir      $r12, $r12, 2",
        "lddir      $r12, $r12, 1",
        "ldpte      $r12, 0",
        "ldpte      $r12, 1",
        "tlbfill",
        "csrrd      $r12, {LOONGARCH_CSR_TLBRSAVE}",
        "ertn",
        LOONGARCH_CSR_TLBRSAVE = const 0x8b,
        LOONGARCH_CSR_PGD = const 0x1b,
        options(noreturn)
        );
    }
}
