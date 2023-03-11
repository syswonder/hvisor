.global _start
.extern stack_top

.section ".text.boot"

_start:
    ldr   x30       , =stack_top  //; 栈顶指针
    mov   sp        , x30         //; 传递
    bl    not_main                //; 跳转
    b     .                       //; 永远不会运行到这里