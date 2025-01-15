#ifndef __HVISOR_DEF_H
#define __HVISOR_DEF_H

#ifdef __aarch64__
#define ARM64
#endif

#if defined(__riscv) && (__riscv_xlen == 64)
#define RISCV64
#endif

#ifdef __loongarch64__
#define LOONGARCH64
#endif

#ifdef RISCV64

// according to the riscv sbi spec
// SBI return has the following format:
// struct sbiret
//  {
//  long error;
//  long value;
// };

// a0: error, a1: value
static inline __u64 hvisor_call(__u64 code,__u64 arg0, __u64 arg1) {
	register __u64 a0 asm("a0") = code;
	register __u64 a1 asm("a1") = arg0;
	register __u64 a2 asm("a2") = arg1;
	register __u64 a7 asm("a7") = 0x114514;
	asm volatile ("ecall"
	        : "+r" (a0), "+r" (a1)
			: "r" (a2), "r" (a7)
			: "memory");
	return a1;
}
#endif

#ifdef ARM64
static inline __u64 hvisor_call(__u64 code, __u64 arg0, __u64 arg1) {
	register __u64 x0 asm("x0") = code;
	register __u64 x1 asm("x1") = arg0;
	register __u64 x2 asm("x2") = arg1;

	asm volatile ("hvc #0x4856"
	        : "+r" (x0)
			: "r" (x1), "r" (x2)
			: "memory");
	return x0;
}
#endif /* ARM64 */
#endif /* __HVISOR_DEF_H */