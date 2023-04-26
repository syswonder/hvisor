use core::arch::global_asm; // 支持内联汇编

pub unsafe extern "C" fn el2_entry() -> i32 {
    core::arch::asm!(
        "
        bl {entry}     
        eret
    ",
        entry = sym crate::entry,
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

        mov	x16, x0
	    mov	x17, x30
        ldr	x13, =BASE_ADDRESS 
        ldr	x12, =0x7fc00000    //config file hv.phy_mem
        sub	x11, x12, x13
        ldr	x1, =bootstrap_vectors
        virt2phys x1       

        /* choose opcode */
        mov	x0, 0
        hvc	#0   //install bootstrap vec

        hvc	#0	/* bootstrap vectors enter EL2 at el2_entry */
        mov	x30, x17 /* we go back to linux */
        //mov x0, -22 //return EINVAL ?driver
        ret

    ",
        options(noreturn),
    );
}
