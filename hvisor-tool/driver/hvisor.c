#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/miscdevice.h>
#include <linux/mm.h>
#include <linux/interrupt.h>
#include <linux/slab.h>
// #include <asm/io.h>
#include <linux/io.h>
#include <linux/sched/signal.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <linux/gfp.h>
#include <linux/vmalloc.h>
#include <linux/string.h>
#include <asm/cacheflush.h>
#include <linux/string.h> 
#include <linux/of_reserved_mem.h>
#include "hvisor.h"
#include "zone_config.h"

struct virtio_bridge *virtio_bridge;
int hvisor_irq = -1;
static struct task_struct *task = NULL;

// initial virtio el2 shared region
static int hvisor_init_virtio(void)
{
    int err;
    if (hvisor_irq == -1)
    {
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
static int hvisor_finish_req(void)
{
    int err;
    err = hvisor_call(HVISOR_HC_FINISH_REQ, 0, 0);
    if (err)
        return err;
    return 0;
}

// static int flush_cache(__u64 phys_start, __u64 size)
// {
//     struct vm_struct *vma;
//     int err = 0;
//     size = PAGE_ALIGN(size);
//     vma = __get_vm_area(size, VM_IOREMAP, VMALLOC_START, VMALLOC_END);
//     if (!vma)
//     {
//         pr_err("hvisor: failed to allocate virtual kernel memory for image\n");
//         return -ENOMEM;
//     }
//     vma->phys_addr = phys_start;

//     if (ioremap_page_range((unsigned long)vma->addr, (unsigned long)(vma->addr + size), phys_start, PAGE_KERNEL_EXEC))
//     {
//         pr_err("hvisor: failed to ioremap image\n");
//         err = -EFAULT;
//         goto unmap_vma;
//     }
//     // flush icache will also flush dcache
//     flush_icache_range((unsigned long)(vma->addr), (unsigned long)(vma->addr + size));

// unmap_vma:
//     vunmap(vma->addr);
//     return err;
// }

static int hvisor_zone_start(zone_config_t __user *arg)
{
    int err = 0;
    printk("hvisor_zone_start\n");
    zone_config_t *zone_config = kmalloc(sizeof(zone_config_t), GFP_KERNEL);

    if (zone_config == NULL)
    {
        pr_err("hvisor: failed to allocate memory for zone_config\n");
    }

    if (copy_from_user(zone_config, arg, sizeof(zone_config_t)))
    {
        pr_err("hvisor: failed to copy from user\n");
        kfree(zone_config);
        return -EFAULT;
    }

    // flush_cache(zone_config->kernel_load_paddr, zone_config->kernel_size);
    // flush_cache(zone_config->dtb_load_paddr, zone_config->dtb_size);

    err = hvisor_call(HVISOR_HC_START_ZONE, __pa(zone_config), 0);
    kfree(zone_config);
    return err;
}

static int is_reserved_memory(unsigned long phys, unsigned long size) {
    struct device_node *parent, *child;
    struct reserved_mem *rmem;
    phys_addr_t mem_base;
    size_t mem_size;
    int count = 0;
    parent = of_find_node_by_path("/reserved-memory");
    count = of_get_child_count(parent);

    for_each_child_of_node(parent, child) {
        rmem = of_reserved_mem_lookup(child);
        mem_base = rmem->base;
        mem_size = rmem->size;
        if (mem_base <= phys && (mem_base + mem_size) >= (phys + size)) {
            return 1;
        }
    }
    return 0;
}

static int hvisor_zone_list(zone_list_args_t __user *arg)
{
    int ret;
    zone_info_t *zones;
    zone_list_args_t args;

    /* Copy user provided arguments to kernel space */
    if (copy_from_user(&args, arg, sizeof(zone_list_args_t)))
    {
        pr_err("hvisor: failed to copy from user\n");
        return -EFAULT;
    }

    zones = kmalloc(args.cnt * sizeof(zone_info_t), GFP_KERNEL);
    memset(zones, 0, args.cnt * sizeof(zone_info_t));

    ret = hvisor_call(HVISOR_HC_ZONE_LIST, __pa(zones), args.cnt);
    if (ret < 0)
    {
        pr_err("hvisor: failed to get zone list\n");
        goto out;
    }
    // copy result back to user space
    if (copy_to_user(args.zones, zones, ret * sizeof(zone_info_t)))
    {
        pr_err("hvisor: failed to copy to user\n");
        goto out;
    }
out:
    kfree(zones);
    return ret;
}

static long hvisor_ioctl(struct file *file, unsigned int ioctl,
                         unsigned long arg)
{
    int err = 0;
    switch (ioctl)
    {
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
    default:
        err = -EINVAL;
        break;
    }
    return err;
}

// Kernel mmap handler
static int hvisor_map(struct file *filp, struct vm_area_struct *vma)
{
    unsigned long phys;
    int err;

    pr_info("[WHEATFOX] hvior mmap handler, vmarea start: %lx, end: %lx, pgoff: %lx\n", vma->vm_start, vma->vm_end, vma->vm_pgoff);

    if (vma->vm_pgoff == 0)
    {
        // virtio_bridge must be aligned to one page.
        phys = virt_to_phys(virtio_bridge);
        // vma->vm_flags |= (VM_IO | VM_LOCKED | (VM_DONTEXPAND | VM_DONTDUMP)); Not sure should we add this line.
        err = remap_pfn_range(vma,
                              vma->vm_start,
                              phys >> PAGE_SHIFT,
                              vma->vm_end - vma->vm_start,
                              vma->vm_page_prot);
        if (err)
            return err;
        pr_info("virtio bridge mmap succeed!\n");
    } else {
	    size_t size = vma->vm_end - vma->vm_start;
#ifdef LOONGARCH64
        // according to embeded fdt restriction, the reserved memory check is 
        // disabled for loongarch64. please make sure your region in JSON
        // is strictly inside root linux's reserved memory!!!
#else
        // vm_pgoff is the physical page number.
        if (!is_reserved_memory(vma->vm_pgoff << PAGE_SHIFT, size)) {
            pr_err("The physical address to be mapped is not within the reserved memory\n");
            return -EFAULT;
        }
#endif
        err = remap_pfn_range(vma,
                              vma->vm_start,
                              vma->vm_pgoff,
                              size,
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

// Interrupt handler for IRQ.
static irqreturn_t irq_handler(int irq, void *dev_id)
{
    struct siginfo info;
    if (dev_id != &hvisor_misc_dev)
    {
        return IRQ_NONE;
    }

    memset(&info, 0, sizeof(struct siginfo));
    info.si_signo = SIGHVI;
    info.si_code = SI_QUEUE;
    info.si_int = 1;
    // Send signale SIGHVI to hvisor user task
    if (task != NULL)
    {
        // pr_info("send signal to hvisor device\n");
        if (send_sig_info(SIGHVI, (struct kernel_siginfo *)&info, task) < 0)
        {
            pr_err("Unable to send signal\n");
        }
    }
    return IRQ_HANDLED;
}

/*
** Module Init function
*/
static int __init hvisor_init(void)
{
    int err;
    struct device_node *node = NULL;
    err = misc_register(&hvisor_misc_dev);
    if (err)
    {
        pr_err("hvisor_misc_register failed!!!\n");
        return err;
    }
    // The irq number must be retrieved from dtb node, because it is different from GIC's IRQ number.
    node = of_find_node_by_path("/hvisor_device");

    if (!node)
    {
        pr_info("hvisor_device node not found in dtb, can't use virtio devices\n");
        // return -ENODEV;
    }
    else
    {
        hvisor_irq = of_irq_get(node, 0);
        pr_info("hvisor_irq in dtb is %d\n", hvisor_irq);
        err = request_irq(hvisor_irq, irq_handler, IRQF_SHARED | IRQF_TRIGGER_RISING, "hvisor", &hvisor_misc_dev);
        if (err)
        {
            pr_err("hvisor cannot register IRQ, err is %d\n", err);
            free_irq(hvisor_irq, &hvisor_misc_dev);
            misc_deregister(&hvisor_misc_dev);
            return err;
        }
    }
    pr_info("hvisor init done!!!\n");
    return 0;
}

/*
** Module Exit function
*/
static void __exit hvisor_exit(void)
{
    if (hvisor_irq != -1)
        free_irq(hvisor_irq, &hvisor_misc_dev);

    if (virtio_bridge != NULL)
    {
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