#ifndef __HVISOR_DEF_H
#define __HVISOR_DEF_H

#ifdef __aarch64__
#define ARM64
#endif

#if defined(__riscv) && (__riscv_xlen == 64)
#define RISCV64
#endif

#endif /* __HVISOR_DEF_H */