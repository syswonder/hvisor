#include "hvisor.h"
#include "server.h"
#include <linux/delay.h>
#include <linux/of.h>
#include <linux/reset.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/version.h>

static struct reset_control **reset_cache;
static int reset_cache_count;

int reset_init(void) {
    struct device_node *np;
    int count;

    np = hvisor_get_node();
    if (!np) {
        pr_err("hvisor_virtio_device node not found\n");
        return -ENODEV;
    }

    count = of_count_phandle_with_args(np, "resets", "#reset-cells");
    if (count == -ENOENT) {
        pr_debug(
            "no resets property in hvisor node, reset protocol disabled\n");
        reset_cache = NULL;
        reset_cache_count = 0;
        return 0;
    }
    if (count < 0) {
        pr_err("failed to parse resets property in hvisor node: %d\n", count);
        return count;
    }

    reset_cache = kcalloc(count, sizeof(*reset_cache), GFP_KERNEL);
    if (!reset_cache)
        return -ENOMEM;

    reset_cache_count = count;
    return 0;
}

static struct reset_control *reset_get_cached(u32 reset_id) {
    struct device_node *np;

    if (reset_id >= reset_cache_count)
        return ERR_PTR(-ENOENT);

    if (reset_cache[reset_id])
        return reset_cache[reset_id];

    np = hvisor_get_node();
    if (!np)
        return ERR_PTR(-ENODEV);

/*
 * 9035073d0ef1 ("reset: convert reset core to using firmware nodes")
 * removed __of_reset_control_get in v7.1-rc1, use
 * __fwnode_reset_control_get instead.
 */
#if LINUX_VERSION_CODE >= KERNEL_VERSION(7, 1, 0)
    reset_cache[reset_id] = __fwnode_reset_control_get(
        of_fwnode_handle(np), NULL, reset_id, RESET_CONTROL_EXCLUSIVE);
/*
 * c84b0326d5e4 ("reset: add acquired/released state for exclusive reset
 * controls") added the 'acquired' parameter in v5.2.  Before that the
 * function had 5 arguments (no acquired).
 */
#elif LINUX_VERSION_CODE >= KERNEL_VERSION(5, 2, 0)
    reset_cache[reset_id] =
        __of_reset_control_get(np, NULL, reset_id, false, false, true);
#else
    reset_cache[reset_id] =
        __of_reset_control_get(np, NULL, reset_id, false, false);
#endif

    return reset_cache[reset_id];
}

void reset_ctrl_finish(void) {
    int i;

    if (!reset_cache)
        return;

    for (i = 0; i < reset_cache_count; i++) {
        if (reset_cache[i] && !IS_ERR(reset_cache[i])) {
            reset_control_put(reset_cache[i]);
            reset_cache[i] = NULL;
        }
    }

    kfree(reset_cache);
    reset_cache = NULL;
    reset_cache_count = 0;
}

static int reset_domain(u32 reset_id, u32 flags, u32 reset_state) {
    int ret;
    struct reset_control *rstc;

    rstc = reset_get_cached(reset_id);
    if (IS_ERR(rstc))
        return PTR_ERR(rstc);

    ret = reset_control_acquire(rstc);
    if (ret) {
        pr_err("Failed to acquire reset domain %u: %d\n", reset_id, ret);
        return ret;
    }

    switch (flags) {
    case SCMI_RESET_DEASSERT:
        ret = reset_control_deassert(rstc);
        break;
    case SCMI_RESET_ASSERT:
        ret = reset_control_assert(rstc);
        break;
    case SCMI_RESET_RESET:
        ret = reset_control_reset(rstc);
        if (ret == -ENOTSUPP) {
            ret = reset_control_assert(rstc);
            if (ret)
                goto out;
            udelay(1);
            ret = reset_control_deassert(rstc);
        }
        break;
    default:
        ret = -EINVAL;
        break;
    }

out:
    reset_control_release(rstc);
    return ret;
}

int hvisor_scmi_reset_ioctl(struct hvisor_scmi_reset_args __user *user_args) {
    struct hvisor_scmi_reset_args args;

    if (copy_from_user(&args, user_args, sizeof(struct hvisor_scmi_reset_args)))
        return -EFAULT;

    switch (args.subcmd) {
    case HVISOR_SCMI_RESET_GET_COUNT: {
        args.u.reset_count = (u32)reset_cache_count;
        if (copy_to_user(&user_args->u.reset_count, &args.u.reset_count,
                         sizeof(u32)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_RESET_RESET:
        return reset_domain(args.u.reset_info.domain_id,
                            args.u.reset_info.flags,
                            args.u.reset_info.reset_state);

    default:
        pr_err("Invalid SCMI reset subcommand: %d\n", args.subcmd);
        return -EINVAL;
    }
}
