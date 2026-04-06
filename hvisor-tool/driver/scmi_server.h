#ifndef _HVISOR_SCMI_SERVER_H
#define _HVISOR_SCMI_SERVER_H

#ifdef ENABLE_VIRTIO_SCMI

#include "../include/hvisor.h"
#include <linux/clk.h>
#include <linux/reset.h>

/* SCMI reset constants */
#define SCMI_RESET_DEASSERT 0
#define SCMI_RESET_RESET (1 << 0)
#define SCMI_RESET_ASSERT (1 << 1)

/* Clock provider phandle, hardcoded as requested */
#define CLOCK_PROVIDER_PHANDLE 2

/* Reset provider phandle hardcoded as requested */
#define RESET_PROVIDER_PHANDLE 2

/* Function declarations */
int hvisor_scmi_clock_ioctl(struct hvisor_scmi_clock_args __user *user_args);
int hvisor_scmi_reset_ioctl(struct hvisor_scmi_reset_args __user *user_args);

#endif /* ENABLE_VIRTIO_SCMI */

#endif /* _HVISOR_SCMI_SERVER_H */