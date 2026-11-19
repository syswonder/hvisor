#ifndef _HVISOR_SCMI_SERVER_H
#define _HVISOR_SCMI_SERVER_H

#include "hvisor.h"
#include <linux/clk.h>
#include <linux/reset.h>

/* SCMI reset constants */
#define SCMI_RESET_DEASSERT 0
#define SCMI_RESET_RESET (1 << 0)
#define SCMI_RESET_ASSERT (1 << 1)

/* Return the hvisor_virtio_device DT node (compatible "hvisor").
 * Result is cached across calls; callers must NOT of_node_put() it.
 * Call hvisor_put_node() once at module exit to release the cache.
 */
struct device_node *hvisor_get_node(void);
void hvisor_put_node(void);

/* Init — called once at module init (before misc_register) */
int hvisor_scmi_init(void);
int clock_init(void);
int reset_init(void);
int power_init(void);

/* Function declarations */
int hvisor_scmi_clock_ioctl(struct hvisor_scmi_clock_args __user *user_args);
int hvisor_scmi_reset_ioctl(struct hvisor_scmi_reset_args __user *user_args);
int hvisor_scmi_power_ioctl(struct hvisor_scmi_power_args __user *user_args);

/* Cleanup — called from hvisor_exit() to release SCMI resources */
void hvisor_scmi_cleanup(void);

#endif /* _HVISOR_SCMI_SERVER_H */
