#include "scmi_server.h"
#include "hvisor.h"
#include <linux/delay.h>
#include <linux/of.h>
#include <linux/reset.h>
#include <linux/uaccess.h>

#ifdef ENABLE_VIRTIO_SCMI

/* Clock provider phandle - configurable via ioctl */
static uint32_t clock_provider_phandle = 0;

/* Reset provider phandle - configurable via ioctl */
static uint32_t reset_provider_phandle = 0;

extern bool __clk_is_enabled(const struct clk *clk);
extern const char *__clk_get_name(const struct clk *clk);
extern struct reset_control *get_reset_domain_by_id(unsigned int reset_id);
/**
 * get_clock_provider_node - Get the clock provider node
 *
 * Return: Pointer to the clock provider node on success, NULL on failure
 */
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
                break; // idx >= provider->data->clk_num, which means no more
                       // clocks
        } else {
            clk_put(clk);
        }

        count++;
    }

    of_node_put(provider_np);
    return count;
}

/**
 * get_reset_provider_node - Get the reset provider node
 *
 * Return: Pointer to the reset provider node on success, NULL on failure
 */
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

    // Use the reset-domains interface to get reset control by ID
    rstc = get_reset_domain_by_id(reset_id);
    if (IS_ERR(rstc)) {
        pr_err("Failed to get reset control for ID %u: %ld\n", reset_id,
               PTR_ERR(rstc));
        return rstc;
    }

    return rstc;
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

    // Bit 0: Clock enabled status
    if (__clk_is_enabled(clk)) {
        clk_config |= 1;
    }

    *config = clk_config;
    *extended_config_val = 0; // Not implemented

    clk_put(clk);
    of_node_put(provider_np);

    pr_debug("clock[%u] config=0x%x, extended_config_val=0x%x\n", clock_id,
             *config, *extended_config_val);
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

    pr_debug("clock[%u] name=%s\n", clock_id, name);
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
        *is_valid = 0; // Clock is invalid
        *enabled = 0;
        *parent_id = -1;
        snprintf(clock_name, 64, "invalid_clock_%u", clock_id);
        return 0;
    }

    *is_valid = 1; // Clock is valid

    // Get clock enable status
    *enabled = __clk_is_enabled(clk) ? 1 : 0;

    // Get parent clock
    parent_clk = clk_get_parent(clk);
    if (IS_ERR(parent_clk) || !parent_clk) {
        *parent_id = -1; // No parent
    } else {
        // For now, we'll set a default parent_id
        // In a more complete implementation, we'd need to find the parent's
        // index
        *parent_id = -1; // Simplified for now
    }

    // Get clock name
    name = __clk_get_name(clk);
    if (name) {
        strncpy(clock_name, name, 63);
        clock_name[63] = '\0';
    } else {
        snprintf(clock_name, 64, "unknown_clk_%u", clock_id);
    }

    clk_put(clk);
    of_node_put(provider_np);

    pr_debug("clock[%u] name=%s enabled=%u parent_id=%d is_valid=%u\n",
             clock_id, clock_name, *enabled, *parent_id, *is_valid);
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

    pr_debug("clock[%u] rate=%llu Hz\n", clock_id, *rate);
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

    if (ret == 0) {
        pr_debug("clock[%u] rate set to %llu Hz\n", clock_id, rate);
    } else {
        pr_err("Failed to set clock[%u] rate to %llu Hz, error=%d\n", clock_id,
               rate, ret);
    }

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

    // Bit 0: Clock enable/disable
    if (config & 1) {
        // Enable clock
        ret = clk_prepare_enable(clk);
        if (ret) {
            pr_err("Failed to enable clock[%u], error=%d\n", clock_id, ret);
        }
    } else {
        // Disable clock
        clk_disable_unprepare(clk);
        pr_debug("clock[%u] disabled\n", clock_id);
    }

    clk_put(clk);
    of_node_put(provider_np);

    return ret;
}

static int reset_domain(u32 reset_id, u32 flags, u32 reset_state) {
    struct reset_control *rstc;
    int ret = 0;

    // Get the reset control by ID
    rstc = get_reset_domain_by_id(reset_id);
    if (IS_ERR(rstc)) {
        pr_err("Failed to get reset control for ID %u: %ld\n", reset_id,
               PTR_ERR(rstc));
        return PTR_ERR(rstc);
    }

    // Acquire the reset control
    ret = reset_control_acquire(rstc);
    if (ret) {
        pr_err("Failed to acquire reset domain %u: %d\n", reset_id, ret);
        reset_control_put(rstc);
        return ret;
    }

    // Perform the requested reset operation
    switch (flags) {
    case SCMI_RESET_DEASSERT:
        pr_debug("Deasserting reset domain %u\n", reset_id);
        ret = reset_control_deassert(rstc);
        break;
    case SCMI_RESET_RESET:
        pr_debug("Resetting domain %u\n", reset_id);
        ret = reset_control_reset(rstc);
        if (ret == -ENOTSUPP) {
            // Fallback to assert + deassert if reset is not supported
            pr_debug("Fallback to assert+deassert for reset domain %u\n",
                     reset_id);
            ret = reset_control_assert(rstc);
            if (ret) {
                pr_err("Failed to assert reset domain %u: %d\n", reset_id, ret);
                break;
            }
            // Add a small delay between assert and deassert
            udelay(1);
            ret = reset_control_deassert(rstc);
            if (ret)
                pr_err("Failed to deassert reset domain %u: %d\n", reset_id,
                       ret);
        }
        break;
    case SCMI_RESET_ASSERT:
        pr_debug("Asserting reset domain %u\n", reset_id);
        ret = reset_control_assert(rstc);
        break;
    default:
        pr_err("Invalid reset type %u\n", flags);
        ret = -EINVAL;
        break;
    }

    // Release the reset control
    reset_control_release(rstc);

    if (ret) {
        pr_err("Reset operation failed for domain %u: %d\n", reset_id, ret);
    }

    // Put the reset control
    reset_control_put(rstc);

    return ret;
}

int hvisor_scmi_clock_ioctl(struct hvisor_scmi_clock_args __user *user_args) {
    struct hvisor_scmi_clock_args args;

    if (copy_from_user(&args, user_args, sizeof(struct hvisor_scmi_clock_args)))
        return -EFAULT;

    pr_debug("SCMI clock ioctl, subcmd=%d\n", args.subcmd);

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
        // u32 num_rates, remaining;
        // u64 rates[8];

        // int ret = get_clock_rates(args.u.clock_rates_info.clock_id,
        //                          args.u.clock_rates_info.rate_index,
        //                          &num_rates, &remaining, rates);
        // if (ret < 0)
        //     return ret;

        // args.u.clock_rates_info.num_rates = num_rates;
        // args.u.clock_rates_info.remaining = remaining;

        // for (int i = 0; i < num_rates; i++) {
        //     args.u.clock_rates_info.rates[i] = rates[i];
        // }

        // if (copy_to_user(&user_args->u.clock_rates_info,
        // &args.u.clock_rates_info, sizeof(args.u.clock_rates_info)))
        //     return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_RATE_GET: {
        u64 rate = 0;
        int ret = get_clock_rate(args.u.clock_rate_info.clock_id,
                                 &rate); // ignore return value
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