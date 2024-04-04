#ifndef __HVISOR_H
#define __HVISOR_H
#include <linux/ioctl.h>
#include <linux/types.h>
#define MMAP_SIZE 4096
#define MAX_REQ 32
#define MAX_CPUS 20

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
	__u64 address; // cell's ipa
	__u64 size;
	__u64 value;
	__u32 src_cell;
	__u8 is_write;
	__u8 need_interrupt;
};

struct device_res {
    __u32 target_cell;
    __u32 irq_id;
};

struct hvisor_device_region {
	__u32 req_front;
	__u32 req_rear;
    __u32 res_front;
    __u32 res_rear;
	struct device_req req_list[MAX_REQ];
    struct device_res res_list[MAX_REQ];
	__u8 cfg_flags[MAX_CPUS];
	__u64 cfg_values[MAX_CPUS];
};

#define HVISOR_INIT_VIRTIO  _IO(1, 0) // virtio device init
#define HVISOR_GET_TASK _IO(1, 1)	
#define HVISOR_FINISH _IO(1, 2)		  // finish one virtio req	
#define HVISOR_ZONE_START _IOW(1, 3, struct hvisor_zone_load*)
#define HVISOR_ZONE_SHUTDOWN _IOW(1, 4, __u64)
// hypercall
#define HVISOR_CALL_NUM_RESULT "x0"
#define HVISOR_CALL_ARG1       "x1"
#define HVISOR_CALL_INS        "hvc #0x4a48"

#define HVISOR_HC_INIT_VIRTIO 9
#define HVISOR_HC_FINISH_REQ 10
#define HVISOR_HC_START_ZONE 11
#define HVISOR_HC_SHUTDOWN_ZONE 12

static inline __u64 hvisor_call(__u64 num)
{
	register __u64 num_result asm(HVISOR_CALL_NUM_RESULT) = num;

	asm volatile(
		HVISOR_CALL_INS
		: "=r" (num_result)
		: "r" (num_result)
		: "memory");
	return num_result;
}

static inline __u64 hvisor_call_arg1(__u64 num, __u64 arg1)
{
	register __u64 num_result asm(HVISOR_CALL_NUM_RESULT) = num;
	register __u64 __arg1 asm(HVISOR_CALL_ARG1) = arg1;

	asm volatile(
		HVISOR_CALL_INS
		: "=r" (num_result)
		: "r" (num_result), "r" (__arg1)
		: "memory");
	return num_result;
}



#endif /* __HVISOR_H */
