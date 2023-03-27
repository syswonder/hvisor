.global _start
.extern stack_top

.section ".text.boot"

_start:
    ldr   x30       , =stack_top      //; 栈顶指针
    mov   sp        , x30             //; 传递
    mrs   x5        , CurrentEL       //; 当前异常等级移动到x5
    ubfx  x5, x5, #2, #2
    cmp   x5        , 3         //; 判断是不是EL3
    b.eq  el3_entry                   //; 如果是，就进入el3_entry
    b     el2_entry                   //; 否则，进入el2_entry


    bl    init                    //; 跳转
    b     .                           //; 永远不会运行到这里