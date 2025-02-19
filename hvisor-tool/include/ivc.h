#ifndef __IVC_H
#define __IVC_H
#include "def.h"
#include <linux/ioctl.h>
#include <linux/types.h>

#define HVISOR_IVC_USER_INFO _IOR('I', 0, ivc_uinfo_t *)

#define CONFIG_MAX_IVC_CONFIGS 2
#define HVISOR_HC_IVC_INFO 5

#define SIGIVC 40

struct ivc_control_table {
    volatile __u32 ivc_id;
    volatile __u32 max_peers;
    volatile __u32 rw_sec_size;
    volatile __u32 out_sec_size;
    volatile __u32 peer_id;
    volatile __u32 ipi_invoke;
} __attribute__((packed));
typedef struct ivc_control_table ivc_cttable_t;

struct ivc_user_info {
    int len;
    int ivc_ids[CONFIG_MAX_IVC_CONFIGS];
};
typedef struct ivc_user_info ivc_uinfo_t;
#endif