use crate::consts::{HV_HEADER_PTR, PER_CPU_SIZE};
use core::arch::global_asm; // 支持内联汇编
use crate::percpu::PerCpu;
global_asm!(
    include_str!("./page_table.S"),);
global_asm!(
    include_str!("./bootvec.S"),
    sym el2_entry
);
global_asm!(
    include_str!("./hyp_vec.S"),
    sym crate::arch_handle_exit);

    
pub unsafe extern "C" fn boot_pt() -> i32 {
    core::arch::asm!(
        "
        adrp	x0, __trampoline_start  //phy addr

        /* map the l1 table that includes the firmware and the uart */
        get_index x2, x13, 0
        set_table bootstrap_pt_l0, x2, bootstrap_pt_l1_hyp_uart
    
        /* map the l1 table that includes the trampoline */
        get_index x3, x0, 0
        set_table bootstrap_pt_l0, x3, bootstrap_pt_l1_trampoline
    
        /*  fill the l1 tables */
        get_index x2, x13, 1
        set_table bootstrap_pt_l1_hyp_uart, x2, bootstrap_pt_l2_hyp_uart
        get_index x4, x0, 1
        set_block bootstrap_pt_l1_trampoline, x4, x0, 1
    
        get_index x2, x13, 2
        set_block bootstrap_pt_l2_hyp_uart, x2, x12, 2
    
        adrp	x0, bootstrap_pt_l0  //phy addr
        
        ret
    
        
        
    ",
        options(noreturn),
    );
}
#[link_section = ".trampoline"]
pub unsafe extern "C" fn enable_mmu() -> i32 {
    core::arch::asm!(
        "
        /*
        * x0: u64 ttbr0_el2
        */
   
       /* setup the MMU for EL2 hypervisor mappings */
       ldr	x1, =MAIR_FLAG
       msr	mair_el2, x1
       ldr	x1, =TCR_FLAG
	    msr	tcr_el2, x1

	    msr	ttbr0_el2, x0

	    isb
	    tlbi	alle2
	    dsb	nsh

	    /* Enable MMU, allow cacheability for instructions and data */
	    ldr	x1, =SCTLR_FLAG
	    msr	sctlr_el2, x1

	    isb
	    tlbi	alle2
	    dsb	nsh

	    ret        
    ",
        options(noreturn),
    );
}
pub unsafe extern "C" fn vmreturn() -> i32 {
    core::arch::asm!(
        "
        mov	x30, x17 
        ret        //return to linux
        
    ",
        options(noreturn),
    );
}
pub unsafe extern "C" fn switch_stack() -> i32 {
    let per_cpu_size=PER_CPU_SIZE;
    let cpu_data = match PerCpu::new() {
        Ok(c) => c,
        Err(e) => return e.code(),
    };
    let hv_sp = cpu_data.stack_top();
    core::arch::asm!(
        "
        /* install the final vectors */
        adr	x1, hyp_vectors
        msr	vbar_el2, x1
    
        mov	x0, x16		/* preserved cpuid, will be passed to entry */
        adrp	x1, __core_end
        mov	x2, {per_cpu_size}
        /*
         * percpu data = pool + cpuid * percpu_size
         */
        madd	x1, x2, x0, x1
    
        /* set up the stack and push the root cell's callee saved registers */
        mov	sp, {hv_sp}
        stp	x29, x17, [sp, #-16]!	/* note: our caller lr is in x17 */
        stp	x27, x28, [sp, #-16]!
        stp	x25, x26, [sp, #-16]!
        stp	x23, x24, [sp, #-16]!
        stp	x21, x22, [sp, #-16]!
        stp	x19, x20, [sp, #-16]!
        /*
         * We pad the stack, so we can consistently access the guest
         * registers from either the initialization, or the exception
         * handling code paths. 19 caller saved registers plus the
         * exit_reason, which we don't use on entry.
         */
        sub	sp, sp, 20 * 8
    
        mov	x29, xzr	/* reset fp,lr */
        mov	x30, xzr
    
        /* Call entry(cpuid, struct per_cpu*). Should not return. */
        bl {entry}
        eret        //back to ?arch_entry hvc0
        mov	x30, x17 
        ret        //return to linux

    
                
    ",
    per_cpu_size=in(reg) per_cpu_size,
    hv_sp=in(reg) hv_sp,
        //vmreturn=sym vmreturn,
        entry = sym crate::entry,
        options(noreturn),
    );
}
pub unsafe extern "C" fn el2_entry() -> i32 {
    core::arch::asm!(
        "
        mrs	x1, esr_el2      //  Exception Syndrome Register
        lsr	x1, x1, #26      // EC, bits [31:26]
        cmp	x1, #0x16           // hvc ec value
        b.ne	.		/* not hvc */
        bl {0}
        adr	x0, bootstrap_pt_l0
	    adr	x30, {2}	/* set lr switch_stack phy*/
	    phys2virt x30		
	    b	{1}     
        eret
    ",
        sym boot_pt,
        sym enable_mmu,
        sym switch_stack,

        options(noreturn),

    );
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn arch_entry() -> i32 {
    core::arch::asm!(
        "

        mov	x16, x0                             //x16 cpuid
	    mov	x17, x30                            //x17 linux ret addr
        /*
        *TODO:1 change header or config read step into a singe not naked func
        *      2 just read them depend on offset         
        */
        ldr	x13, =BASE_ADDRESS 
        ldr	x12, =0x7fc00000                    //should be read from config file
        sub	x11, x12, x13                       //x11= (el2 mmu)virt-phy offset 
        ldr	x1, =bootstrap_vectors
        virt2phys x1       

        /* choose opcode */
        mov	x0, 0
        hvc	#0                                  //install bootstrap vec

        hvc	#0	                                /* bootstrap vectors enter EL2 at el2_entry */
                                     //tmp return here
        mov	x30, x17                            /* we go back to linux */
        //mov x0, -22                           
        ret

    ",
        options(noreturn),
    );
}
