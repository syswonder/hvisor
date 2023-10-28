use core::arch::global_asm; // 支持内联汇编
#[naked]
#[no_mangle]
#[link_section = ".header"]
pub unsafe extern "C" fn el1_entry() -> i32 {
    core::arch::asm!(
        "
    
        ldr	x0, =stack_top
        mov	sp, x0
    
        mov	x0, #(3 << 20)
        msr	cpacr_el1, x0
    
        msr	daif, xzr
    
        isb
    
        b	{start}
    
        ",
        start = sym crate::start,
        options(noreturn),
    );
}
