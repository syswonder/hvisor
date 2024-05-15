#ifndef __HVISOR_H
#define __HVISOR_H
#include <linux/ioctl.h>
#include <linux/types.h>
#define MMAP_SIZE 4096
#define MAX_REQ 32
#define MAX_DEVS 4
#define MAX_CPUS 16

#define SIGHVI 10
// used when start a zone.
struct hvisor_zone_info {
	__u64 zone_id;
	__u64 image_phys_addr;
	__u64 dtb_phys_addr;
};
struct hvisor_zone_load {
	__u64 zone_id;
	__u32 images_num;
	__u32 padding;
	struct hvisor_image_desc* images;
};

struct hvisor_image_desc {
	__u64 source_address; // image address in user space
	__u64 target_address; // image physical address to load
	__u64 size;
};

// receive request from el2
struct device_req {
	__u64 src_cpu;
	__u64 address; // zone's ipa
	__u64 size;
	__u64 value;
	__u32 src_zone;
	__u8 is_write;
	__u8 need_interrupt;
	__u16 padding;
};

struct device_res {
    __u32 target_zone;
    __u32 irq_id;
};

struct virtio_bridge {
	__u32 req_front;
	__u32 req_rear;
    __u32 res_front;
    __u32 res_rear;
	struct device_req req_list[MAX_REQ];
    struct device_res res_list[MAX_REQ];
	__u8 cfg_flags[MAX_CPUS];
	__u64 cfg_values[MAX_CPUS];
	// When config is okay to use, remove these
	__u64 mmio_addrs[MAX_DEVS];
	__u8 mmio_avail;
	__u8 need_wakeup;
};

#define HVISOR_INIT_VIRTIO  _IO(1, 0) // virtio device init
#define HVISOR_GET_TASK _IO(1, 1)	
#define HVISOR_FINISH_REQ _IO(1, 2)		  // finish one virtio req	
#define HVISOR_ZONE_START _IOW(1, 3, struct hvisor_zone_load*)
#define HVISOR_ZONE_SHUTDOWN _IOW(1, 4, __u64)
// hypercall
#define HVISOR_CALL_HVC        "hvc #0x4856"

#define HVISOR_HC_INIT_VIRTIO 0
#define HVISOR_HC_FINISH_REQ 1
#define HVISOR_HC_START_ZONE 2
#define HVISOR_HC_SHUTDOWN_ZONE 3

static inline __u64 hvisor_call(__u64 code)
{
	register __u64 code_result asm("x0") = code;

	asm volatile(
		HVISOR_CALL_HVC
		: "=r" (code_result)
		: "r" (code_result)
		: "memory");
	return code_result;
}

static inline __u64 hvisor_call_arg1(__u64 code, __u64 arg0)
{
	register __u64 code_result asm("x0") = code;
	register __u64 __arg0 asm("x1") = arg0;

	asm volatile(
		HVISOR_CALL_HVC
		: "=r" (code_result)
		: "r" (code_result), "r" (__arg0)
		: "memory");
	return code_result;
}



#endif /* __HVISOR_H */
