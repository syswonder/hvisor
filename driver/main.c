#include<linux/kernel.h>
#include<linux/init.h>
#include<linux/module.h>
#include<linux/miscdevice.h>
#include<linux/mm.h>
#include<linux/interrupt.h>
#include<linux/slab.h>   
// #include <asm/io.h>
#include <linux/io.h>
#include "hvisor.h"
#include <linux/sched/signal.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <asm/page.h>
#include <linux/gfp.h>
#include <linux/vmalloc.h>
#include <asm/cacheflush.h>

struct hvisor_device_region *device_region; 
// initial virtio el2 shared region
static int hvisor_init_virtio(void) 
{
	int err;
	device_region = __get_free_pages(GFP_KERNEL, 0);
    SetPageReserved(virt_to_page(device_region));
    // init device region
    device_region->req_front = 0;
    device_region->req_rear = 0;
    device_region->res_front = 0;
    device_region->res_rear = 0;
	if (device_region == NULL)
		return -ENOMEM;
	err = hvisor_call_arg1(HVISOR_HC_INIT_VIRTIO, __pa(device_region));
	if (err)
		return err;
	return 0;
}

// finish virtio req and send result to el2
static int hvisor_finish_req(void) 
{
    pr_info("hvisor finish request\n");
    int err;
    err = hvisor_call(HVISOR_HC_FINISH_REQ);
    if (err)
        return err;
    return 0;
}

static int load_image(struct hvisor_image_desc __user *arg, __u64 *phys_addr) {
    struct hvisor_image_desc image;
    struct vm_struct *vma;
    int err = 0;
    __u64 phys_start, offset_in_page;
    unsigned long size;
    
    if (copy_from_user(&image, arg, sizeof(struct hvisor_image_desc)))
        return -EFAULT;
    *phys_addr = image.target_address;
    phys_start = image.target_address & PAGE_MASK;
    offset_in_page = image.target_address & ~PAGE_MASK;
    size = PAGE_ALIGN(image.size + offset_in_page);

    vma = __get_vm_area(size, VM_IOREMAP, VMALLOC_START, VMALLOC_END);
    if (!vma) {
        pr_err("hvisor: failed to allocate virtual kernel memory for image\n");
        return -ENOMEM;
    }
    vma->phys_addr = phys_start;
    if (ioremap_page_range(vma->addr, vma->addr + size, phys_start, PAGE_KERNEL_EXEC)) {
        pr_err("hvisor: failed to ioremap image\n");
        err = -EFAULT;
        goto out_unmap_vma;
    }

    if(copy_from_user((void *)(vma->addr + offset_in_page), image.source_address, image.size)) {
        err = -EFAULT;
        goto out_unmap_vma;
    }
    // Make sure the data is in memory before we start executing it.
    flush_icache_range(vma->addr + offset_in_page, vma->addr + offset_in_page + image.size);

out_unmap_vma:
    vunmap(vma->addr);
    return err;
}

static int hvisor_zone_start(struct hvisor_zone_load __user* arg) {
    struct hvisor_zone_load zone_load;
    struct hvisor_image_desc __user *images = arg->images;
    struct hvisor_zone_info *zone_info;
    zone_info = kmalloc(sizeof(struct hvisor_zone_info), GFP_KERNEL);

    if (zone_info == NULL) {
        pr_err("hvisor: failed to allocate memory for zone_info\n");
        return -ENOMEM;
    }
    int err = 0;
    if (copy_from_user(&zone_load, arg, sizeof(zone_load))) 
        return -EFAULT;
    // load image
    err = load_image(images, &zone_info->image_phys_addr);
    // load dtb
    err = load_image(++images, &zone_info->dtb_phys_addr);
    if (err)
        return err;
    err = hvisor_call_arg1(HVISOR_HC_START_ZONE, __pa(zone_info));
    return err;
}

static long hvisor_ioctl(struct file *file, unsigned int ioctl,
			    unsigned long arg)
{
    int err = 0;
    switch (ioctl)
    {
    case HVISOR_INIT_VIRTIO:
        err = hvisor_init_virtio(); 
        break;
    case HVISOR_ZONE_START:
        err = hvisor_zone_start((struct hvisor_zone_load __user*) arg);
        break;
    case HVISOR_FINISH:
        err = hvisor_finish_req();
        break;
    default:
        err = -EINVAL;
        break;
    }
    return err;
}

// Kernel mmap handler
static int hvisor_map(struct file * filp, struct vm_area_struct *vma) 
{
    unsigned long phys;
    
    // device_region must be aligned to one page.
    phys = virt_to_phys(device_region);
    // vma->vm_flags |= (VM_IO | VM_LOCKED | (VM_DONTEXPAND | VM_DONTDUMP)); Not sure should we add this line.
    if(remap_pfn_range(vma, 
                    vma->vm_start,
                    phys >> PAGE_SHIFT,
                    vma->vm_end - vma->vm_start,
                    vma->vm_page_prot))
        return -1;
    pr_info("hvisor mmap succeed!\n");
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

/*
** Module Init function
*/
static int __init hvisor_init(void)
{
    int err;
    err = misc_register(&hvisor_misc_dev);
    if (err) {
        pr_err("hvisor_misc_register failed!!!\n");
        return err;
    }
    printk("hvisor init done!!!\n");
    return 0;
}

/*
** Module Exit function
*/
static void __exit hvisor_exit(void)
{
    misc_deregister(&hvisor_misc_dev);
    pr_info("hvisor exit!!!\n");
}
 
module_init(hvisor_init);
module_exit(hvisor_exit);
 
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KouweiLee <15035660024@163.com>");
MODULE_DESCRIPTION("The hvisor device driver");
MODULE_VERSION("1:0.0");