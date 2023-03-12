# Todo and Link

This is a file to record the todo list and the reference links.

1. http://syuu1228.github.io/howto_implement_hypervisor/ : A tutorial about x86 hypervisor.
2. https://github.com/syuu1228/howto_implement_hypervisor: The repo above.
3. https://qemu-project.gitlab.io/qemu/system/arm/virt.html : ‘virt’ generic virtual platform (virt)
4. https://qiita.com/matsud224/items/7ce824d62152054eec41: The greatly tutorial about aarch64 virtulization Todo. 👍 👀️
5. https://github.com/matsud224/raspvisor : The repo above. 👍
6. http://blog.itpub.net/70005277/viewspace-2872524/ : The tutorial about arm64 assembly.
7. https://doc.rust-lang.org/book/ : Rust Tutorial.
8. https://rustwiki.org/zh-CN/rust-by-example/mod.html : Rust Tutorial with Chinese.
9. https://developer.arm.com/documentation/100942/0100/Hypervisor-software : Arm64 official doc about virtualization.
10. https://www.zhihu.com/people/gmn23362: Rust for OS Record Blog.
11. https://lowenware.com/blog/aarch64-bare-metal-program-in-rust/ 👍 C for arm64 OS and some useful skills.
12. https://blog.csdn.net/u011011827/article/details/123844962 👍 Qemu for arm64 about baremetal.

## Todolist

Q: How to do next?

A: 总结Arm64的主要寄存器的使用方法，然后根据虚拟化文档，列出虚拟化的步骤，然后代码实现。如第一步需要配置什么寄存器，第二步需要开启什么功能。。。

然后进入Rust主函数，需要思考怎么来分配模块和crate，此时需要再学习一遍rust语言的特性，把编程风格改为rust。

最后，正式开始设计hypervisor所需要的架构与模块实现。
