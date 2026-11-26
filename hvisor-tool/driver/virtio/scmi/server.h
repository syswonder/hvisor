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

/* Per-zone lifecycle for host-refcounted SCMI resources (clocks via
 * clk_prepare_enable and power domains via pm_runtime_resume_and_get).
 *
 * Reset controls (reset.c) are intentionally excluded: they use exclusive
 * handles acquired and released within each ioctl (idempotent line-level
 * operations with no persistent kernel refcount), and neither asserting nor
 * deasserting a reset line on shutdown is universally safe for an unowned
 * passthrough device; the next guest driver idempotently resets/deasserts
 * its own lines on probe. */
int clock_zone_release(u32 zone, u32 *released);
int power_zone_release(u32 zone, u32 *released);
/* Release SCMI clock and power-domain references recorded for zone @zone:
 * disable its clocks first, then power off its domains. Idempotent; safe to
 * call before a zone starts (any recorded state then belongs to a dead
 * instance) and after it is shut down. Returns the number of resources
 * released. */
int scmi_release_zone(u32 zone);

#endif /* _HVISOR_SCMI_SERVER_H */
