ENTRY(_start)
SECTIONS
{
    . = 0x40000000; /* QEMU的物理内存地址，文件加载地址 */
    .text.boot : {*(.text.boot)} /* 指定start.s编译后start.o文件的代码段.text输出为.startup段, 如果不刻意声明这一段，那么容易出现起始代码不是start.s文件*/
    .text : {*(.text)}
    .data : {*(.data)}
    .rodata : {*(.rodata)}
    .bss : {*(.bss)}

    . = ALIGN(8); /* 8字节对齐，下面放栈 */

    . = . + 0x4000; /* 栈的大小是0x4000=16KB */
    stack_top = .; /* 对应start.s中的stack_top */

}