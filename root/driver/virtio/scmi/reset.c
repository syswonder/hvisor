#include "hvisor.h"
#include "server.h"
#include <linux/delay.h>
#include <linux/of.h>
#include <linux/reset.h>
#include <linux/uaccess.h>

#ifdef ENABLE_VIRTIO_SCMI

extern struct reset_control *get_reset_domain_by_id(unsigned int reset_id);

/* Reset provider phandle - configurable via ioctl */
static uint32_t reset_provider_phandle = 0;

static struct device_node *get_reset_provider_node(void) {
    struct device_node *provider_np =
        of_find_node_by_phandle(reset_provider_phandle);
    if (!provider_np) {
        pr_err("Failed to find reset provider node\n");
    }
    return provider_np;
}

static struct reset_control *get_reset_by_id(u32 reset_id,
                                             struct device_node *provider_np) {
    struct reset_control *rstc;

    rstc = get_reset_domain_by_id(reset_id);
    if (IS_ERR(rstc)) {
        pr_err("Failed to get reset control for ID %u: %ld\n", reset_id,
               PTR_ERR(rstc));
        return rstc;
    }

    return rstc;
}

static int reset_domain(u32 reset_id, u32 flags, u32 reset_state) {
    struct reset_control *rstc;
    int ret = 0;

    rstc = get_reset_domain_by_id(reset_id);
    if (IS_ERR(rstc)) {
        pr_err("Failed to get reset control for ID %u: %ld\n", reset_id,
               PTR_ERR(rstc));
        return PTR_ERR(rstc);
    }

    ret = reset_control_acquire(rstc);
    if (ret) {
        pr_err("Failed to acquire reset domain %u: %d\n", reset_id, ret);
        reset_control_put(rstc);
        return ret;
    }

    switch (flags) {
    case SCMI_RESET_DEASSERT:
        ret = reset_control_deassert(rstc);
        break;
    case SCMI_RESET_RESET:
        ret = reset_control_reset(rstc);
        if (ret == -ENOTSUPP) {
            ret = reset_control_assert(rstc);
            if (ret) {
                pr_err("Failed to assert reset domain %u: %d\n", reset_id, ret);
                break;
            }
            udelay(1);
            ret = reset_control_deassert(rstc);
            if (ret)
                pr_err("Failed to deassert reset domain %u: %d\n", reset_id,
                       ret);
        }
        break;
    case SCMI_RESET_ASSERT:
        ret = reset_control_assert(rstc);
        break;
    default:
        pr_err("Invalid reset type %u\n", flags);
        ret = -EINVAL;
        break;
    }

    reset_control_release(rstc);

    if (ret)
        pr_err("Reset operation failed for domain %u: %d\n", reset_id, ret);

    reset_control_put(rstc);

    return ret;
}

int hvisor_scmi_reset_ioctl(struct hvisor_scmi_reset_args __user *user_args) {
    struct hvisor_scmi_reset_args args;

    if (copy_from_user(&args, user_args,
                       sizeof(struct hvisor_scmi_reset_args))) {
        return -EFAULT;
    }

    switch (args.subcmd) {
    case HVISOR_SCMI_RESET_RESET: {
        int ret =
            reset_domain(args.u.reset_info.domain_id, args.u.reset_info.flags,
                         args.u.reset_info.reset_state);
        return ret;
    }
    case HVISOR_SCMI_RESET_SET_PHANDLE: {
        reset_provider_phandle = args.u.reset_phandle_info.phandle;
        pr_info("Reset provider phandle set to %u\n", reset_provider_phandle);
        return 0;
    }
    default:
        pr_err("Invalid SCMI reset subcommand: %d\n", args.subcmd);
        return -EINVAL;
    }
}

#endif /* ENABLE_VIRTIO_SCMI */
