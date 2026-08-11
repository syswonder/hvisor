#include "hvisor.h"
#include "server.h"
#include <linux/of.h>
#include <linux/platform_device.h>
#include <linux/pm_domain.h>
#include <linux/pm_runtime.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/version.h>

struct power_domain_dev {
    struct device *virt_dev;
    bool attached;
};

static struct power_domain_dev *power_devices;
static u32 power_max_num;

/* Control platform device - carrier for the hvisor DT node */
static struct platform_device *ctrl_pdev;

int power_init(void) {
    struct device_node *np;
    int c;

    np = hvisor_get_node();
    if (!np)
        return -ENODEV;

    c = of_count_phandle_with_args(np, "power-domains", "#power-domain-cells");
    if (c == -ENOENT) {
        pr_debug("no power-domains property in hvisor node, power protocol "
                 "disabled\n");
        power_devices = NULL;
        power_max_num = 0;
        ctrl_pdev = NULL;
        return 0;
    }
    if (c < 0)
        return c;
    power_max_num = (u32)c;
    power_devices = kcalloc(power_max_num, sizeof(*power_devices), GFP_KERNEL);
    if (!power_devices)
        return -ENOMEM;

    ctrl_pdev = platform_device_alloc("hvisor-pd", -1);
    if (!ctrl_pdev) {
        kfree(power_devices);
        power_devices = NULL;
        return -ENOMEM;
    }
    ctrl_pdev->dev.of_node = np;
    if (platform_device_add(ctrl_pdev)) {
        platform_device_put(ctrl_pdev);
        ctrl_pdev = NULL;
        kfree(power_devices);
        power_devices = NULL;
        return -ENODEV;
    }

    return 0;
}

static int power_domain_get_count(u32 *count) {
    *count = power_max_num;
    return 0;
}

static int power_domain_init(u32 domain_id) {
    struct device *virt_dev;

    if (domain_id >= power_max_num)
        return -ENOENT;

    if (power_devices[domain_id].attached)
        return 0;

    virt_dev = genpd_dev_pm_attach_by_id(&ctrl_pdev->dev, domain_id);
    if (IS_ERR_OR_NULL(virt_dev)) {
        pr_err("genpd_dev_pm_attach_by_id(%u) failed: %ld\n", domain_id,
               PTR_ERR(virt_dev));
        return virt_dev ? PTR_ERR(virt_dev) : -ENODEV;
    }

    power_devices[domain_id].virt_dev = virt_dev;
    power_devices[domain_id].attached = true;

    return 0;
}

void power_ctrl_finish(void) {
    u32 i;

    if (power_devices) {
        for (i = 0; i < power_max_num; i++) {
            if (power_devices[i].attached && power_devices[i].virt_dev) {
                dev_pm_domain_detach(power_devices[i].virt_dev, true);
                power_devices[i].virt_dev = NULL;
                power_devices[i].attached = false;
            }
        }
        kfree(power_devices);
        power_devices = NULL;
        power_max_num = 0;
    }

    if (ctrl_pdev) {
        platform_device_unregister(ctrl_pdev);
        ctrl_pdev = NULL;
    }
}

static int power_domain_state_set(u32 domain_id, u32 power_state) {
    struct device *dev;
    int ret;

    if (domain_id >= power_max_num || !power_devices[domain_id].attached)
        return -ENODEV;

    dev = power_devices[domain_id].virt_dev;

    if (power_state == SCMI_POWER_STATE_GENERIC_ON) {
/*
 * dd8088d5a896 ("PM: runtime: Add pm_runtime_resume_and_get to deal with
 * usage counter") introduced pm_runtime_resume_and_get() in v5.10.
 * Before that, callers must handle the error path themselves.
 */
#if LINUX_VERSION_CODE >= KERNEL_VERSION(5, 10, 0)
        ret = pm_runtime_resume_and_get(dev);
#else
        ret = pm_runtime_get_sync(dev);
        if (ret < 0)
            pm_runtime_put_noidle(dev);
#endif
        return ret;
    }
    if (power_state == SCMI_POWER_STATE_GENERIC_OFF)
        return pm_runtime_put_sync(dev);

    return -EINVAL;
}

static int power_domain_state_get(u32 domain_id, u32 *power_state) {
    int ret;

    if (domain_id >= power_max_num)
        return -ENODEV;

    if (!power_devices[domain_id].attached) {
        ret = power_domain_init(domain_id);
        if (ret)
            return ret;
    }

    *power_state = pm_runtime_active(power_devices[domain_id].virt_dev)
                       ? SCMI_POWER_STATE_GENERIC_ON
                       : SCMI_POWER_STATE_GENERIC_OFF;
    return 0;
}

static int power_domain_get_attributes(u32 domain_id, u32 *flags, char *name) {
    int ret;

    if (domain_id >= power_max_num)
        return -ENODEV;

    if (!power_devices[domain_id].attached) {
        ret = power_domain_init(domain_id);
        if (ret)
            return ret;
    }

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
        int ret = power_domain_state_get(domain_id, &power_state);
        if (ret < 0)
            return ret;

        args.u.power_state_info.power_state = power_state;
        if (copy_to_user(&user_args->u.power_state_info,
                         &args.u.power_state_info,
                         sizeof(args.u.power_state_info)))
            return -EFAULT;
        return 0;
    }

    default:
        pr_err("Invalid SCMI power subcommand: %d\n", args.subcmd);
        return -EINVAL;
    }
}
