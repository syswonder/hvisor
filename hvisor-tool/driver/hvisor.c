// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      Guowei Li <2401213322@stu.pku.edu.cn>
 */
#include <asm/cacheflush.h>
#include <linux/eventfd.h>
#include <linux/gfp.h>
#include <linux/init.h>
#include <linux/interrupt.h>
#include <linux/io.h>
#include <linux/kernel.h>
#include <linux/miscdevice.h>
#include <linux/mm.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <linux/of_reserved_mem.h>
#include <linux/sched/signal.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/uaccess.h>
#include <linux/version.h>
#include <linux/vmalloc.h>
#include <linux/clk.h>
#include <linux/delay.h>
#include <linux/reset.h>

#include "hvisor.h"
#include "zone_config.h"

/* Clock provider phandle, hardcoded as requested */
#define CLOCK_PROVIDER_PHANDLE 2

/* Reset provider phandle, hardcoded as requested */
#define RESET_PROVIDER_PHANDLE 2

#define SCMI_RESET_DEASSERT	     0
#define SCMI_RESET_RESET	     (1 << 0)
#define SCMI_RESET_ASSERT        (1 << 1)

extern bool __clk_is_enabled(const struct clk *clk);
extern const char *__clk_get_name(const struct clk *clk);
extern unsigned long clk_get_rate(struct clk *clk);
extern int clk_set_rate(struct clk *clk, unsigned long rate);
extern int clk_prepare_enable(struct clk *clk);
extern void clk_disable_unprepare(struct clk *clk);

/* Reset controller functions */
#include <linux/reset.h>
#include <linux/reset-domains.h>

extern struct reset_control *reset_control_get(struct device *dev, const char *id);
extern struct reset_control *reset_control_get_optional(struct device *dev, const char *id);
extern struct reset_control *devm_reset_control_get(struct device *dev, const char *id);
extern void reset_control_put(struct reset_control *rstc);
extern int reset_control_reset(struct reset_control *rstc);
extern int reset_control_assert(struct reset_control *rstc);
extern int reset_control_deassert(struct reset_control *rstc);
extern struct reset_control *get_reset_domain_by_id(unsigned int reset_id);

/**
 * get_clock_provider_node - Get the clock provider node
 *
 * Return: Pointer to the clock provider node on success, NULL on failure
 */
static struct device_node *get_clock_provider_node(void) {
    struct device_node *provider_np = of_find_node_by_phandle(CLOCK_PROVIDER_PHANDLE);
    if (!provider_np) {
        pr_err("Failed to find clock provider node\n");
    }
    return provider_np;
}

static struct clk *get_clock_by_id(u32 clk_id, struct device_node *provider_np) {
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
            if (PTR_ERR(clk) == -EINVAL) break; // idx >= provider->data->clk_num, which means no more clocks
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
    struct device_node *provider_np = of_find_node_by_phandle(RESET_PROVIDER_PHANDLE);
    if (!provider_np) {
        pr_err("Failed to find reset provider node\n");
    }
    return provider_np;
}

static struct reset_control *get_reset_by_id(u32 reset_id, struct device_node *provider_np) {
    struct reset_control *rstc;

    // Use the reset-domains interface to get reset control by ID
    rstc = get_reset_domain_by_id(reset_id);
    if (IS_ERR(rstc)) {
        pr_err("Failed to get reset control for ID %u: %ld\n", reset_id, PTR_ERR(rstc));
        return rstc;
    }

    return rstc;
}

static int reset_domain(u32 reset_id, u32 flags, u32 reset_state) {
    struct reset_control *rstc;
    int ret = 0;

    // Get the reset control by ID
    rstc = get_reset_domain_by_id(reset_id);
    if (IS_ERR(rstc)) {
        pr_err("Failed to get reset control for ID %u: %ld\n", reset_id, PTR_ERR(rstc));
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
            pr_debug("Fallback to assert+deassert for reset domain %u\n", reset_id);
            ret = reset_control_assert(rstc);
            if (ret) {
                pr_err("Failed to assert reset domain %u: %d\n", reset_id, ret);
                break;
            }
            // Add a small delay between assert and deassert
            udelay(1);
            ret = reset_control_deassert(rstc);
            if (ret)
                pr_err("Failed to deassert reset domain %u: %d\n", reset_id, ret);
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


static int get_clock_config(u32 clock_id, u32 *config, u32 *extended_config_val) {
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

    pr_debug("clock[%u] config=0x%x, extended_config_val=0x%x\n", clock_id, *config, *extended_config_val);
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

static int get_clock_attributes(u32 clock_id, u32 *enabled, u32 *parent_id, char *clock_name, u32 *is_valid) {
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
        // In a more complete implementation, we'd need to find the parent's index
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

    pr_debug("clock[%u] name=%s enabled=%u parent_id=%d is_valid=%u\n", clock_id, clock_name, *enabled, *parent_id, *is_valid);
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
        pr_err("Failed to set clock[%u] rate to %llu Hz, error=%d\n", clock_id, rate, ret);
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

// Encapsulate the function to handle SCMI clock ioctl
static int hvisor_scmi_clock_ioctl(struct hvisor_scmi_clock_args __user *user_args) {
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
        if (copy_to_user(&user_args->u.clock_count, &args.u.clock_count, sizeof(u32)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_GET_ATTRIBUTES: {
        u32 enabled, parent_id, is_valid;
        char clock_name[64];
        int ret = get_clock_attributes(args.u.clock_attr.clock_id, &enabled, &parent_id, clock_name, &is_valid);
        args.u.clock_attr.enabled = enabled;
        args.u.clock_attr.parent_id = parent_id;
        args.u.clock_attr.is_valid = is_valid;
        strncpy(args.u.clock_attr.clock_name, clock_name, 63);
        args.u.clock_attr.clock_name[63] = '\0';
        if (copy_to_user(&user_args->u.clock_attr, &args.u.clock_attr, sizeof(args.u.clock_attr)))
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

        // if (copy_to_user(&user_args->u.clock_rates_info, &args.u.clock_rates_info, sizeof(args.u.clock_rates_info)))
        //     return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_RATE_GET: {
        u64 rate = 0;
        int ret = get_clock_rate(args.u.clock_rate_info.clock_id, &rate); // ignore return value
        args.u.clock_rate_info.rate = rate;
        if (copy_to_user(&user_args->u.clock_rate_info, &args.u.clock_rate_info, sizeof(args.u.clock_rate_info)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_RATE_SET: {
        return set_clock_rate(args.u.clock_rate_set_info.clock_id, args.u.clock_rate_set_info.rate);
    }
    case HVISOR_SCMI_CLOCK_CONFIG_GET: {
        u32 config, extended_config_val;
        int ret = get_clock_config(args.u.clock_config_info.clock_id, &config, &extended_config_val);
        if (ret < 0)
            return ret;
        args.u.clock_config_info.config = config;
        args.u.clock_config_info.extended_config_val = extended_config_val;
        if (copy_to_user(&user_args->u.clock_config_info, &args.u.clock_config_info, sizeof(args.u.clock_config_info)))
            return -EFAULT;
        return 0;
    }
    case HVISOR_SCMI_CLOCK_CONFIG_SET: {
        return set_clock_config(args.u.clock_config_info.clock_id, args.u.clock_config_info.config);
    }
    case HVISOR_SCMI_CLOCK_NAME_GET: {
        char name[64];
        int ret = get_clock_name(args.u.clock_name_info.clock_id, name);
        if (ret < 0)
            return ret;
        strncpy(args.u.clock_name_info.name, name, 63);
        args.u.clock_name_info.name[63] = '\0';
        if (copy_to_user(&user_args->u.clock_name_info, &args.u.clock_name_info, sizeof(args.u.clock_name_info)))
            return -EFAULT;
        return 0;
    }
    default:
        return -EINVAL;
    }
}

static int hvisor_scmi_reset_ioctl(struct hvisor_scmi_reset_args __user *user_args) {
    struct hvisor_scmi_reset_args args;
    
    if (copy_from_user(&args, user_args, sizeof(struct hvisor_scmi_reset_args)))
        return -EFAULT;

    switch (args.subcmd) {
    case HVISOR_SCMI_RESET_RESET: {
        int ret = reset_domain(args.u.reset_info.domain_id, args.u.reset_info.flags, 
                              args.u.reset_info.reset_state);
        return ret;
    }
    default:
        return -EINVAL;
    }
}

struct virtio_bridge *virtio_bridge;
int virtio_irq = -1;
static struct task_struct *task = NULL;
struct eventfd_ctx *virtio_irq_ctx = NULL;

// initial virtio el2 shared region
static int hvisor_init_virtio(void) {
    int err;
    if (virtio_irq == -1) {
        pr_err("virtio device is not available\n");
        return ENOTTY;
    }
    virtio_bridge = (struct virtio_bridge *)__get_free_pages(GFP_KERNEL, 0);
    if (virtio_bridge == NULL)
        return -ENOMEM;
    SetPageReserved(virt_to_page(virtio_bridge));
    // init device region
    memset(virtio_bridge, 0, sizeof(struct virtio_bridge));
    err = hvisor_call(HVISOR_HC_INIT_VIRTIO, __pa(virtio_bridge), 0);
    if (err)
        return err;
    return 0;
}

// finish virtio req and send result to el2
static int hvisor_finish_req(void) {
    int err;
    err = hvisor_call(HVISOR_HC_FINISH_REQ, 0, 0);
    if (err)
        return err;
    return 0;
}

// Flush mapped cache range to memory.
static int flush_cache_mapped(void *vaddr, __u64 size) {
    unsigned long start, end, addr, line_size;
    size = PAGE_ALIGN(size);
    start = (unsigned long)vaddr;
    end = start + size;

#if defined(ARM64)
    asm volatile("mrs %0, ctr_el0" : "=r"(line_size));
    line_size = 4 << ((line_size >> 16) & 0xf);
    // Clean and Invalidate Data Cache to Point of Coherency (PoC)
    addr = start & ~(line_size - 1);
    while (addr < end) {
        asm volatile("dc civac, %0" : : "r"(addr) : "memory");
        addr += line_size;
    }
    // barrier, confirm operations are completed and other cores can see the
    // changes.
    asm volatile("dsb sy" : : : "memory");
#elif defined(RISCV64)
    // TODO: implement riscv64 flush operation
#elif defined(LOONGARCH64)
    // TODO: implement loongarch64 flush operation
#elif defined(X86_64)
    // TODO: implement x86_64 flush operation
#else
    pr_err("hvisor.ko: unsupported architecture\n");
#endif
    return 0;
}

static int hvisor_load_image(struct hvisor_load_image_args __user *arg) {
    struct hvisor_load_image_args kargs;
    void *vaddr = NULL;
    __u64 map_phys;
    __u64 page_offs;
    __u64 map_size;
    void *dst;
    int ret = 0;

    if (copy_from_user(&kargs, arg, sizeof(kargs)))
        return -EFAULT;

    if (!kargs.user_buffer || !kargs.size)
        return -EINVAL;

    // Align to page boundary
    map_phys = kargs.load_paddr & PAGE_MASK;
    page_offs = kargs.load_paddr - map_phys;
    if (kargs.size > U64_MAX - page_offs)
        return -EINVAL;
    map_size = PAGE_ALIGN(kargs.size + page_offs);
    if (map_size < kargs.size)
        return -EINVAL;

    // This is physical RAM for image loading, not MMIO, so keep it WB
    // cacheable.
    vaddr = memremap(map_phys, map_size, MEMREMAP_WB);
    if (!vaddr) {
        return -ENOMEM;
    }

    dst = (char *)vaddr + page_offs;
    if (copy_from_user(dst, u64_to_user_ptr(kargs.user_buffer), kargs.size)) {
        ret = -EFAULT;
        goto out;
    }

    // Clean D-cache to PoC first so new contents are visible globally.
    ret = flush_cache_mapped(vaddr, map_size);
    if (ret)
        goto out;

    // Then invalidate I-cache for the written image range.
    flush_icache_range((unsigned long)dst, (unsigned long)dst + kargs.size);

out:
    memunmap(vaddr);
    return ret;
}

static int hvisor_zone_start(zone_config_t __user *arg) {
    int err = 0;
    int i = 0;

    zone_config_t *zone_config = kmalloc(sizeof(zone_config_t), GFP_KERNEL);

    if (zone_config == NULL) {
        pr_err("hvisor.ko: failed to allocate memory for zone_config\n");
    }

    if (copy_from_user(zone_config, arg, sizeof(zone_config_t))) {
        pr_err("hvisor.ko: failed to copy from user\n");
        kfree(zone_config);
        return -EFAULT;
    }

    pr_info("hvisor.ko: invoking hypercall to start the zone\n");

    err = hvisor_call(HVISOR_HC_START_ZONE, __pa(zone_config),
                      sizeof(zone_config_t));
    kfree(zone_config);
    return err;
}

// #ifndef LOONGARCH64
// static int is_reserved_memory(unsigned long phys, unsigned long size) {
//     struct device_node *parent, *child;
//     struct reserved_mem *rmem;
//     phys_addr_t mem_base;
//     size_t mem_size;
//     int count = 0;
//     parent = of_find_node_by_path("/reserved-memory");
//     count = of_get_child_count(parent);

//     for_each_child_of_node(parent, child) {
//         rmem = of_reserved_mem_lookup(child);
//         mem_base = rmem->base;
//         mem_size = rmem->size;
//         if (mem_base <= phys && (mem_base + mem_size) >= (phys + size)) {
//             return 1;
//         }
//     }
//     return 0;
// }
// #endif

static int hvisor_config_check(u64 __user *arg) {
    int err = 0;
    u64 *config;
    config = kmalloc(sizeof(u64), GFP_KERNEL);
    err = hvisor_call(HVISOR_HC_CONFIG_CHECK, __pa(config), 0);

    if (err != 0) {
        pr_err("hvisor.ko: failed to get hvisor config\n");
    }

    if (copy_to_user(arg, config, sizeof(u64))) {
        pr_err("hvisor.ko: failed to copy to user\n");
        kfree(config);
        return -EFAULT;
    }

    kfree(config);
    return err;
}

static int hvisor_zone_list(zone_list_args_t __user *arg) {
    int ret;
    zone_info_t *zones;
    zone_list_args_t args;

    /* Copy user provided arguments to kernel space */
    if (copy_from_user(&args, arg, sizeof(zone_list_args_t))) {
        pr_err("hvisor.ko: failed to copy from user\n");
        return -EFAULT;
    }

    zones = kmalloc(args.cnt * sizeof(zone_info_t), GFP_KERNEL);
    memset(zones, 0, args.cnt * sizeof(zone_info_t));

    ret = hvisor_call(HVISOR_HC_ZONE_LIST, __pa(zones), args.cnt);
    if (ret < 0) {
        pr_err("hvisor.ko: failed to get zone list\n");
        goto out;
    }
    // copy result back to user space
    if (copy_to_user(args.zones, zones, ret * sizeof(zone_info_t))) {
        pr_err("hvisor.ko: failed to copy to user\n");
        goto out;
    }
out:
    kfree(zones);
    return ret;
}

static long hvisor_ioctl(struct file *file, unsigned int ioctl,
                         unsigned long arg) {
    int err = 0;
    switch (ioctl) {
    case HVISOR_INIT_VIRTIO:
        err = hvisor_init_virtio();
        task = get_current(); // get hvisor user process
        break;
    case HVISOR_ZONE_START:
        err = hvisor_zone_start((zone_config_t __user *)arg);
        break;
    case HVISOR_ZONE_SHUTDOWN:
        err = hvisor_call(HVISOR_HC_SHUTDOWN_ZONE, arg, 0);
        break;
    case HVISOR_ZONE_LIST:
        err = hvisor_zone_list((zone_list_args_t __user *)arg);
        break;
    case HVISOR_FINISH_REQ:
        err = hvisor_finish_req();
        break;
    case HVISOR_CONFIG_CHECK:
        err = hvisor_config_check((u64 __user *)arg);
        break;
    case HVISOR_SET_EVENTFD: {
        struct eventfd_ctx *ctx = eventfd_ctx_fdget((int)arg);
        if (IS_ERR(ctx)) {
            err = PTR_ERR(ctx);
        } else {
            if (virtio_irq_ctx)
                eventfd_ctx_put(virtio_irq_ctx);
            virtio_irq_ctx = ctx;
        }
        break;
    }
    case HVISOR_LOAD_IMAGE:
        err = hvisor_load_image((struct hvisor_load_image_args __user *)arg);
        break;
    case HVISOR_SCMI_CLOCK_IOCTL:
        err = hvisor_scmi_clock_ioctl((struct hvisor_scmi_clock_args __user *)arg);
        break;
    case HVISOR_SCMI_RESET_IOCTL:
        err = hvisor_scmi_reset_ioctl((struct hvisor_scmi_reset_args __user *)arg);
        break;
#ifdef LOONGARCH64
    case HVISOR_CLEAR_INJECT_IRQ:
        err = hvisor_call(HVISOR_HC_CLEAR_INJECT_IRQ, 0, 0);
        break;
#endif
    default:
        err = -EINVAL;
        break;
    }
    return err;
}

// Kernel mmap handler
static int hvisor_map(struct file *filp, struct vm_area_struct *vma) {
    unsigned long phys;
    int err;
    if (vma->vm_pgoff == 0) {
        // virtio_bridge must be aligned to one page.
        phys = virt_to_phys(virtio_bridge);
        // vma->vm_flags |= (VM_IO | VM_LOCKED | (VM_DONTEXPAND | VM_DONTDUMP));
        // Not sure should we add this line.
        err = remap_pfn_range(vma, vma->vm_start, phys >> PAGE_SHIFT,
                              vma->vm_end - vma->vm_start, vma->vm_page_prot);
        if (err)
            return err;
        pr_info("virtio bridge mmap succeed!\n");
    } else {
        size_t size = vma->vm_end - vma->vm_start;
        // TODO: add check for non root memory region.
        // memremap(0x50000000, 0x30000000, MEMREMAP_WB);
        // vm_pgoff is the physical page number.
        // if (!is_reserved_memory(vma->vm_pgoff << PAGE_SHIFT, size)) {
        //     pr_err("The physical address to be mapped is not within the
        //     reserved memory\n"); return -EFAULT;
        // }
        err = remap_pfn_range(vma, vma->vm_start, vma->vm_pgoff, size,
                              vma->vm_page_prot);
        if (err)
            return err;
        pr_info("non root region mmap succeed!\n");
    }
    return 0;
}

static const struct file_operations hvisor_fops = {
    .owner = THIS_MODULE,
    .unlocked_ioctl = hvisor_ioctl,
    .compat_ioctl = hvisor_ioctl,
    .mmap = hvisor_map,
};

static struct miscdevice hvisor_misc_dev = {
    .minor = MISC_DYNAMIC_MINOR,
    .name = "hvisor",
    .fops = &hvisor_fops,
};

/**
 * @brief Virtio interrupt handler for hypervisor virtio devices
 *
 * This function handles virtio interrupts by signaling the userspace virtio
 * daemon via an eventfd. It validates the device context and signals the
 * eventfd to wake up the userspace process handling virtio operations.
 *
 * @param irq The interrupt number (unused in this handler)
 * @param dev_id Pointer to the device identifier structure
 * @return irqreturn_t IRQ_HANDLED if interrupt was processed successfully,
 *         IRQ_NONE if the interrupt was not for this device or context was
 * invalid
 */
static irqreturn_t virtio_irq_handler(int irq, void *dev_id) {
    // Check the device id and virtio_irq_ctx is valid.
    if (dev_id != &hvisor_misc_dev || !virtio_irq_ctx) {
        return IRQ_NONE;
    }

    // Wake up the userspace virtio daemon.
    // Linux 6.8+ simplified eventfd_signal to one argument and return void
    // static inline void eventfd_signal(struct eventfd_ctx *ctx) in eventfd.h
#if LINUX_VERSION_CODE >= KERNEL_VERSION(6, 8, 0)
    eventfd_signal(virtio_irq_ctx);
#else
    /* e.g. Linux 5.10: eventfd_signal(ctx, n) returns __u64 (amount
     * incremented). */
    if (eventfd_signal(virtio_irq_ctx, 1) == 0)
        pr_err("eventfd_signal: counter overflow or no increment\n");
#endif

    return IRQ_HANDLED;
}

/*
** Module Init function
*/
static int __init hvisor_init(void) {
    int err;
    struct device_node *node = NULL;
    // u32 *irq;
    err = misc_register(&hvisor_misc_dev);
    if (err) {
        pr_err("hvisor_misc_register failed!!!\n");
        return err;
    }
#ifndef X86_64
    // probe hvisor virtio device.
    // The irq number must be retrieved from dtb node, because it is different
    // from GIC's IRQ number.
    node = of_find_node_by_path("/hvisor_virtio_device");
    if (!node) {
        pr_err("Critical: Missing device tree node!\n");
        pr_err("   Please add the following to your device tree:\n");
        pr_err("   hvisor_virtio_device {\n");
        pr_err("       compatible = \"hvisor\";\n");
        pr_err("       interrupts = <0x00 0x20 0x01>;\n");
        pr_err("   };\n");
        return -ENODEV;
    }

    virtio_irq = of_irq_get(node, 0);
    err = request_irq(virtio_irq, virtio_irq_handler,
                      IRQF_SHARED | IRQF_TRIGGER_RISING, "hvisor_virtio_device",
                      &hvisor_misc_dev);
    if (err)
        goto err_out;

    of_node_put(node);
#else
    // we don't use device tree in x86_64, so we have to get IRQ using hypercall
    u32 *irq = kmalloc(sizeof(u32), GFP_KERNEL);
    err = hvisor_call(HVISOR_HC_GET_VIRTIO_IRQ, __pa(irq), 0);
    virtio_irq = *irq;
    err = request_irq(virtio_irq, virtio_irq_handler, IRQF_SHARED,
                      "hvisor_virtio_device", &hvisor_misc_dev);
    if (err)
        goto err_out;

    kfree(irq);
#endif /* X86_64 */
    pr_info("hvisor init done!!!\n");
    return 0;
err_out:
    pr_err("hvisor cannot register IRQ, err is %d\n", err);
    if (virtio_irq != -1)
        free_irq(virtio_irq, &hvisor_misc_dev);
    misc_deregister(&hvisor_misc_dev);
    return err;
}

/*
** Module Exit function
*/
static void __exit hvisor_exit(void) {
    if (virtio_irq != -1)
        free_irq(virtio_irq, &hvisor_misc_dev);
    if (virtio_irq_ctx)
        eventfd_ctx_put(virtio_irq_ctx);
    if (virtio_bridge != NULL) {
        ClearPageReserved(virt_to_page(virtio_bridge));
        free_pages((unsigned long)virtio_bridge, 0);
    }
    misc_deregister(&hvisor_misc_dev);
    pr_info("hvisor exit!!!\n");
}

module_init(hvisor_init);
module_exit(hvisor_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("KouweiLee <15035660024@163.com>");
MODULE_DESCRIPTION("The hvisor device driver");
MODULE_VERSION("1:0.0");