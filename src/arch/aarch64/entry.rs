use crate::consts::{HV_HEADER_PTR, PER_CPU_SIZE};
use core::arch::global_asm; // 支持内联汇编
pub unsafe extern "C" fn el2_entry() -> i32 {
    core::arch::asm!(
        "
        mrs	x1, esr_el2      //  Exception Syndrome Register
        lsr	x1, x1, #26      // EC, bits [31:26]
        cmp	x1, #0x16           // hvc ec value
        b.ne	.		/* not hvc */
        b {enable_mmu}
        bl {entry}     
        eret
    ",
        entry = sym crate::entry,
        sym enable_mmu,
        options(noreturn),

    );
}
pub unsafe extern "C" fn enable_mmu() -> i32 {
    core::arch::asm!(
        "
        
    ",
        options(noreturn),
    );
}
global_asm!(
    include_str!("./bootvec.S"),
    sym el2_entry
);
#[naked]
#[no_mangle]
pub unsafe extern "C" fn arch_entry() -> i32 {
    core::arch::asm!(
        "

        mov	x16, x0                             //x16 cpuid
	    mov	x17, x30                            //x17 linux ret addr
        /* skip for now
        TODO:1 change header or config read step into a singe not naked func
        *      2 just read them depend on offset         
        adr     x0, HV_HEADER_PTR               //store header addr for get info from it
        adrp	x1, __core_end                  //get page pool addr for calculate config addr 
        ldrh	w2, [x0, #HEADER_MAX_CPUS]      //get cpu num
        mov	x3, #PER_CPU_SIZE                   //get percpu size
        
        * sysconfig = pool + max_cpus * percpu_size
        madd	x1, x2, x3, x1 //get config addr
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
        b .                                     //do not return here
        mov	x30, x17                            /* we go back to linux */
        //mov x0, -22                           
        ret

    ",
        options(noreturn),
    );
}
