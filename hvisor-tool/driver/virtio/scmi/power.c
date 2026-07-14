#include "hvisor.h"
#include "server.h"
#include <linux/of.h>
#include <linux/platform_device.h>
#include <linux/pm_domain.h>
#include <linux/pm_runtime.h>
#include <linux/uaccess.h>

#ifdef ENABLE_VIRTIO_SCMI

static uint32_t power_provider_phandle = 0;

#define MAX_POWER_DOMAINS 128

struct power_domain_dev {
    struct platform_device *pdev;
    bool attached;
};

static struct power_domain_dev power_devices[MAX_POWER_DOMAINS];
static struct device_node *power_provider_np = NULL;

static struct device_node *get_power_provider_node(void) {
    if (!power_provider_np) {
        power_provider_np = of_find_node_by_phandle(power_provider_phandle);
        if (!power_provider_np)
            pr_err("Failed to find power provider node (phandle=%u)\n",
                   power_provider_phandle);
    }
    return power_provider_np;
}

static int power_domain_get_count(u32 *count) {
    *count = 0;
    return 0;
}

static int power_domain_init(u32 domain_id) {
    struct of_phandle_args args;
    struct platform_device *pdev;
    int ret;

    if (domain_id >= MAX_POWER_DOMAINS)
        return -ENOMEM;

    if (power_devices[domain_id].attached)
        return 0;

    if (!power_provider_np) {
        ret = -ENODEV;
        goto err;
    }

    pdev = platform_device_alloc("hvisor-power", domain_id);
    if (!pdev) {
        ret = -ENOMEM;
        goto err;
    }

    ret = platform_device_add(pdev);
    if (ret) {
        platform_device_put(pdev);
        goto err;
    }

    args.np = power_provider_np;
    args.args[0] = domain_id;
    args.args_count = 1;

    ret = of_genpd_add_device(&args, &pdev->dev);
    if (ret) {
        pr_err("Failed to attach device to power domain %u: %d\n", domain_id,
               ret);
        platform_device_unregister(pdev);
        goto err;
    }

    pm_runtime_enable(&pdev->dev);

    power_devices[domain_id].pdev = pdev;
    power_devices[domain_id].attached = true;

    return 0;

err:
    power_devices[domain_id].pdev = NULL;
    power_devices[domain_id].attached = false;
    return ret;
}

static int power_domain_state_set(u32 domain_id, u32 power_state) {
    struct device *dev;
    int ret;

    if (domain_id >= MAX_POWER_DOMAINS || !power_devices[domain_id].attached)
        return -ENODEV;

    dev = &power_devices[domain_id].pdev->dev;

    if (power_state == SCMI_POWER_STATE_GENERIC_ON) {
        ret = pm_runtime_resume_and_get(dev);
        if (ret < 0)
            pr_err("Failed to power on domain %u: %d\n", domain_id, ret);
        else
            pr_debug("Power domain %u turned ON\n", domain_id);
        return ret;
    } else if (power_state == SCMI_POWER_STATE_GENERIC_OFF) {
        ret = pm_runtime_put_sync(dev);
        if (ret < 0)
            pr_err("Failed to power off domain %u: %d\n", domain_id, ret);
        else
            pr_debug("Power domain %u turned OFF\n", domain_id);
        return (ret < 0) ? ret : 0;
    }

    return -EINVAL;
}

static int power_domain_state_get(u32 domain_id, u32 *power_state) {
    struct device *dev;

    if (domain_id >= MAX_POWER_DOMAINS || !power_devices[domain_id].attached)
        return -ENODEV;

    dev = &power_devices[domain_id].pdev->dev;

    *power_state = pm_runtime_active(dev) ? SCMI_POWER_STATE_GENERIC_ON
                                          : SCMI_POWER_STATE_GENERIC_OFF;

    return 0;
}

static int power_domain_get_attributes(u32 domain_id, u32 *flags, char *name) {
    if (domain_id >= MAX_POWER_DOMAINS || !power_devices[domain_id].attached)
        return -ENODEV;

    *flags = 0;
    snprintf(name, 63, "pd_%u", domain_id);
    name[63] = '\0';

    return 0;
}

int hvisor_scmi_power_ioctl(struct hvisor_scmi_power_args __user *user_args) {
    struct hvisor_scmi_power_args args;

    if (copy_from_user(&args, user_args, sizeof(struct hvisor_scmi_power_args)))
        return -EFAULT;

    switch (args.subcmd) {
    case HVISOR_SCMI_POWER_GET_COUNT: {
        u32 count = 0;
        int ret = power_domain_get_count(&count);
        if (ret < 0)
            return ret;
        args.u.power_count = count;
        if (copy_to_user(&user_args->u.power_count, &args.u.power_count,
                         sizeof(u32)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_POWER_GET_ATTRIBUTES: {
        u32 flags;
        char name[64];
        int ret;

        ret = power_domain_init(args.u.power_attr.domain_id);
        if (ret < 0)
            return ret;

        ret = power_domain_get_attributes(args.u.power_attr.domain_id, &flags,
                                          name);
        if (ret < 0)
            return ret;

        args.u.power_attr.flags = flags;
        strncpy(args.u.power_attr.name, name, 63);
        args.u.power_attr.name[63] = '\0';

        if (copy_to_user(&user_args->u.power_attr, &args.u.power_attr,
                         sizeof(args.u.power_attr)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_POWER_STATE_SET: {
        u32 domain_id = args.u.power_state_info.domain_id;
        int ret;

        ret = power_domain_init(domain_id);
        if (ret < 0)
            return ret;

        return power_domain_state_set(domain_id,
                                      args.u.power_state_info.power_state);
    }
    case HVISOR_SCMI_POWER_STATE_GET: {
        u32 domain_id = args.u.power_state_info.domain_id;
        u32 power_state;
        int ret;

        if (!power_devices[domain_id].attached)
            return -ENODEV;

        ret = power_domain_state_get(domain_id, &power_state);
        if (ret < 0)
            return ret;

        args.u.power_state_info.power_state = power_state;
        if (copy_to_user(&user_args->u.power_state_info,
                         &args.u.power_state_info,
                         sizeof(args.u.power_state_info)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_POWER_SET_PHANDLE: {
        power_provider_phandle = args.u.power_phandle_info.phandle;
        power_provider_np = NULL;
        pr_info("Power provider phandle set to %u\n", power_provider_phandle);
        return 0;
    }
    default:
        pr_err("Invalid SCMI power subcommand: %d\n", args.subcmd);
        return -EINVAL;
    }
}

#endif /* ENABLE_VIRTIO_SCMI */
