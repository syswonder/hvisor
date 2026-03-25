// SPDX-License-Identifier: GPL-2.0-or-later
/*
 * Reset Domains Array for SCMI Reset Server
 *
 * This module dynamically queries reset_control objects that can be
 * accessed by hvisor-tool driver using reset ID directly.
 */

#include <linux/module.h>
#include <linux/reset.h>
#include <linux/reset-controller.h>
#include <linux/mutex.h>
#include <linux/of.h>
#include <linux/gfp.h>
#include <linux/errno.h>
#include <linux/device.h>
#include <linux/slab.h>
#include <linux/types.h>
#include <linux/list.h>

/* Forward declarations from core.c */
extern struct mutex reset_list_mutex;
extern struct list_head reset_controller_list;
extern struct reset_control *__reset_control_get_internal(struct reset_controller_dev *rcdev,
						 unsigned int index, bool shared, bool acquired);

/* Helper function to get reset controller name */
static inline const char *rcdev_name(struct reset_controller_dev *rcdev)
{
	if (rcdev->dev)
		return dev_name(rcdev->dev);

	if (rcdev->of_node)
		return of_node_full_name(rcdev->of_node);

	return "unknown";
}

/* Helper function to check if this is SCMI reset protocol (protocol@16) */
static bool is_scmi_reset_protocol(struct reset_controller_dev *rcdev)
{
	if (!rcdev->of_node)
		return false;

	/* Check if the reset controller is using protocol@16 (SCMI reset) */
	return of_device_is_compatible(rcdev->of_node, "arm,scmi-reset");
}

/**
 * get_reset_domain_by_id - Get reset_control by reset ID
 * @reset_id: Reset ID to look up
 *
 * Dynamically traverses all registered reset controllers and their reset signals,
 * finding the reset_control for the given reset ID.
 *
 * Returns the reset_control for the given reset ID, or ERR_PTR on failure.
 */
struct reset_control *get_reset_domain_by_id(unsigned int reset_id)
{
	struct reset_controller_dev *rcdev;
	unsigned int current_idx = 0;
	unsigned int i;

	mutex_lock(&reset_list_mutex);
	list_for_each_entry(rcdev, &reset_controller_list, list) {
		/* Skip SCMI reset protocol (protocol@16) */
		if (is_scmi_reset_protocol(rcdev)) {
			pr_debug("Skipping SCMI reset protocol: %s\n", rcdev_name(rcdev));
			continue;
		}

		for (i = 0; i < rcdev->nr_resets; i++) {
			if (current_idx == reset_id) {
				struct reset_control *rstc;
				
				/* Get the reset control */
				rstc = __reset_control_get_internal(rcdev, i, false, false);
				mutex_unlock(&reset_list_mutex);
				
				if (IS_ERR(rstc)) {
					pr_warn("Failed to get reset control for rcdev %s, id %d: %ld\n",
					       rcdev_name(rcdev), i, PTR_ERR(rstc));
				}
				
				return rstc;
			}
			current_idx++;
		}
	}
	mutex_unlock(&reset_list_mutex);

	pr_err("Reset ID %u out of range (max: %u)\n", reset_id, current_idx - 1);
	return ERR_PTR(-EINVAL);
}
// EXPORT_SYMBOL_GPL(get_reset_domain_by_id);

