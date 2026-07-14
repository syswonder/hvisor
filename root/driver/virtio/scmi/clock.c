#include "hvisor.h"
#include "server.h"
#include <linux/clk.h>
#include <linux/of.h>
#include <linux/uaccess.h>

#ifdef ENABLE_VIRTIO_SCMI

extern bool __clk_is_enabled(const struct clk *clk);
extern const char *__clk_get_name(const struct clk *clk);

/* Clock provider phandle - configurable via ioctl */
static uint32_t clock_provider_phandle = 0;

static struct device_node *get_clock_provider_node(void) {
    struct device_node *provider_np =
        of_find_node_by_phandle(clock_provider_phandle);
    if (!provider_np) {
        pr_err("Failed to find clock provider node\n");
    }
    return provider_np;
}

static struct clk *get_clock_by_id(u32 clk_id,
                                   struct device_node *provider_np) {
    struct of_phandle_args clkspec;

    if (!provider_np) {
        return ERR_PTR(-ENODEV);
    }

    clkspec.np = provider_np;
    clkspec.args[0] = clk_id;
    clkspec.args_count = 1;

    return of_clk_get_from_provider(&clkspec);
}

static int get_clock_count(void) {
    struct device_node *provider_np;
    struct clk *clk;
    int count = 0;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    while (1) {
        clk = get_clock_by_id(count, provider_np);

        if (IS_ERR(clk)) {
            if (PTR_ERR(clk) == -EINVAL)
                break;
        } else {
            clk_put(clk);
        }

        count++;
    }

    of_node_put(provider_np);
    return count;
}

static int get_clock_config(u32 clock_id, u32 *config,
                            u32 *extended_config_val) {
    struct device_node *provider_np;
    struct clk *clk;
    u32 clk_config = 0;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    clk = get_clock_by_id(clock_id, provider_np);
    if (IS_ERR(clk)) {
        of_node_put(provider_np);
        if (PTR_ERR(clk) == -ENOENT)
            return -ENOENT;
        return PTR_ERR(clk);
    }

    if (__clk_is_enabled(clk))
        clk_config |= 1;

    *config = clk_config;
    *extended_config_val = 0;

    clk_put(clk);
    of_node_put(provider_np);

    return 0;
}

static int get_clock_name(u32 clock_id, char *name) {
    struct device_node *provider_np;
    struct clk *clk;
    const char *clk_name;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    clk = get_clock_by_id(clock_id, provider_np);
    if (IS_ERR(clk)) {
        of_node_put(provider_np);
        if (PTR_ERR(clk) == -ENOENT)
            return -ENOENT;
        return PTR_ERR(clk);
    }

    clk_name = __clk_get_name(clk);
    if (clk_name) {
        strncpy(name, clk_name, 63);
        name[63] = '\0';
    } else {
        snprintf(name, 64, "unknown_clk_%u", clock_id);
    }

    clk_put(clk);
    of_node_put(provider_np);

    return 0;
}

static int get_clock_attributes(u32 clock_id, u32 *enabled, u32 *parent_id,
                                char *clock_name, u32 *is_valid) {
    struct device_node *provider_np;
    struct clk *clk, *parent_clk;
    const char *name;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    clk = get_clock_by_id(clock_id, provider_np);
    if (IS_ERR(clk)) {
        of_node_put(provider_np);
        *is_valid = 0;
        *enabled = 0;
        *parent_id = -1;
        snprintf(clock_name, 64, "inv_%u", clock_id);
        return 0;
    }

    *is_valid = 1;
    *enabled = __clk_is_enabled(clk) ? 1 : 0;

    parent_clk = clk_get_parent(clk);
    if (IS_ERR(parent_clk) || !parent_clk) {
        *parent_id = -1;
    } else {
        *parent_id = -1;
    }

    name = __clk_get_name(clk);
    if (name) {
        strncpy(clock_name, name, 63);
        clock_name[63] = '\0';
    } else {
        snprintf(clock_name, 64, "unknown_clk_%u", clock_id);
    }

    clk_put(clk);
    of_node_put(provider_np);

    return 0;
}

static int get_clock_rate(u32 clock_id, u64 *rate) {
    struct device_node *provider_np;
    struct clk *clk;
    unsigned long clk_rate;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    clk = get_clock_by_id(clock_id, provider_np);
    if (IS_ERR(clk)) {
        of_node_put(provider_np);
        if (PTR_ERR(clk) == -ENOENT)
            return -ENOENT;
        return PTR_ERR(clk);
    }

    clk_rate = clk_get_rate(clk);
    *rate = (u64)clk_rate;

    clk_put(clk);
    of_node_put(provider_np);

    return 0;
}

static int set_clock_rate(u32 clock_id, u64 rate) {
    struct device_node *provider_np;
    struct clk *clk;
    int ret;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    clk = get_clock_by_id(clock_id, provider_np);
    if (IS_ERR(clk)) {
        of_node_put(provider_np);
        if (PTR_ERR(clk) == -ENOENT)
            return -ENOENT;
        return PTR_ERR(clk);
    }

    ret = clk_set_rate(clk, (unsigned long)rate);

    clk_put(clk);
    of_node_put(provider_np);

    return ret;
}

static int set_clock_config(u32 clock_id, u32 config) {
    struct device_node *provider_np;
    struct clk *clk;
    int ret = 0;

    provider_np = get_clock_provider_node();
    if (!provider_np) {
        return -ENODEV;
    }

    clk = get_clock_by_id(clock_id, provider_np);
    if (IS_ERR(clk)) {
        of_node_put(provider_np);
        if (PTR_ERR(clk) == -ENOENT)
            return -ENOENT;
        return PTR_ERR(clk);
    }

    if (config & 1)
        ret = clk_prepare_enable(clk);
    else
        clk_disable_unprepare(clk);

    clk_put(clk);
    of_node_put(provider_np);

    return ret;
}

int hvisor_scmi_clock_ioctl(struct hvisor_scmi_clock_args __user *user_args) {
    struct hvisor_scmi_clock_args args;

    if (copy_from_user(&args, user_args, sizeof(struct hvisor_scmi_clock_args)))
        return -EFAULT;

    switch (args.subcmd) {
    case HVISOR_SCMI_CLOCK_GET_COUNT: {
        int ret = get_clock_count();
        if (ret < 0)
            return ret;
        args.u.clock_count = (u32)ret;
        if (copy_to_user(&user_args->u.clock_count, &args.u.clock_count,
                         sizeof(u32)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_GET_ATTRIBUTES: {
        u32 enabled, parent_id, is_valid;
        char clock_name[64];
        int ret = get_clock_attributes(args.u.clock_attr.clock_id, &enabled,
                                       &parent_id, clock_name, &is_valid);
        args.u.clock_attr.enabled = enabled;
        args.u.clock_attr.parent_id = parent_id;
        args.u.clock_attr.is_valid = is_valid;
        strncpy(args.u.clock_attr.clock_name, clock_name, 63);
        args.u.clock_attr.clock_name[63] = '\0';
        if (copy_to_user(&user_args->u.clock_attr, &args.u.clock_attr,
                         sizeof(args.u.clock_attr)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_DESCRIBE_RATES: {
        return 0;
    }
    case HVISOR_SCMI_CLOCK_RATE_GET: {
        u64 rate = 0;
        int ret = get_clock_rate(args.u.clock_rate_info.clock_id, &rate);
        args.u.clock_rate_info.rate = rate;
        if (copy_to_user(&user_args->u.clock_rate_info, &args.u.clock_rate_info,
                         sizeof(args.u.clock_rate_info)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_RATE_SET: {
        return set_clock_rate(args.u.clock_rate_set_info.clock_id,
                              args.u.clock_rate_set_info.rate);
    }
    case HVISOR_SCMI_CLOCK_CONFIG_GET: {
        u32 config, extended_config_val;
        int ret = get_clock_config(args.u.clock_config_info.clock_id, &config,
                                   &extended_config_val);
        if (ret < 0)
            return ret;
        args.u.clock_config_info.config = config;
        args.u.clock_config_info.extended_config_val = extended_config_val;
        if (copy_to_user(&user_args->u.clock_config_info,
                         &args.u.clock_config_info,
                         sizeof(args.u.clock_config_info)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_CONFIG_SET: {
        return set_clock_config(args.u.clock_config_info.clock_id,
                                args.u.clock_config_info.config);
    }
    case HVISOR_SCMI_CLOCK_NAME_GET: {
        char name[64];
        int ret = get_clock_name(args.u.clock_name_info.clock_id, name);
        if (ret < 0)
            return ret;
        strncpy(args.u.clock_name_info.name, name, 63);
        args.u.clock_name_info.name[63] = '\0';
        if (copy_to_user(&user_args->u.clock_name_info, &args.u.clock_name_info,
                         sizeof(args.u.clock_name_info)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_SET_PHANDLE: {
        clock_provider_phandle = args.u.clock_phandle_info.phandle;
        pr_info("Clock provider phandle set to %u\n", clock_provider_phandle);
        return 0;
    }
    default:
        return -EINVAL;
    }
}

#endif /* ENABLE_VIRTIO_SCMI */
