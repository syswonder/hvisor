#ifndef __IVC_H
#define __IVC_H
#include <linux/ioctl.h>
#include <linux/types.h>
#include "def.h"

#define HVISOR_IVC_INFO _IOR('I', 0, ivc_info_t*)

#define CONFIG_MAX_IVC_CONFIGS     2
#define HVISOR_HC_IVC_INFO         5

#define SIGIVC 40

struct ivc_info {
	__u64 len;
	__u64 ivc_ct_ipas[CONFIG_MAX_IVC_CONFIGS];
	__u32 ivc_ids[CONFIG_MAX_IVC_CONFIGS];
}__attribute__((packed));
typedef struct ivc_info ivc_info_t;

struct ivc_control_table {
    volatile __u32 ivc_id;
    volatile __u32 max_peers;
    volatile __u64 shared_mem_ipa;
    volatile __u32 rw_sec_size;
    volatile __u32 out_sec_size;
    volatile __u32 peer_id;
    volatile __u32 ipi_invoke;
}__attribute__((packed));
typedef struct ivc_control_table ivc_cttable_t;

#endif