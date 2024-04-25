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
#include <linux/string.h> 

struct virtio_bridge *virtio_bridge; 
int hvisor_irq;
static struct task_struct *task = NULL;

// initial virtio el2 shared region
static int hvisor_init_virtio(void) 
{
	int err;
	virtio_bridge = __get_free_pages(GFP_KERNEL, 0);
	if (virtio_bridge == NULL)
		return -ENOMEM;
    SetPageReserved(virt_to_page(virtio_bridge));
    // init device region
	memset(virtio_bridge, 0, sizeof(struct virtio_bridge));
	err = hvisor_call_arg1(HVISOR_HC_INIT_VIRTIO, __pa(virtio_bridge));
	if (err)
		return err;
	return 0;
}

// finish virtio req and send result to el2
static int hvisor_finish_req(void) 
{
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
    pr_info("hvisor load image to %llx, offset_in_page: %llx, size: %llx\n", phys_start, offset_in_page, size);

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
	zone_info->zone_id = zone_load.zone_id;
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
		task = get_current(); // get hvisor user process
        break;
    case HVISOR_ZONE_START:
        err = hvisor_zone_start((struct hvisor_zone_load __user*) arg);
        break;
	case HVISOR_ZONE_SHUTDOWN:
		err = hvisor_call_arg1(HVISOR_HC_SHUTDOWN_ZONE, arg);	
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
static int hvisor_map(struct file * filp, struct vm_area_struct *vma) 
{
    unsigned long phys;
    
    // virtio_bridge must be aligned to one page.
    phys = virt_to_phys(virtio_bridge);
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

// Interrupt handler for IRQ.
static irqreturn_t irq_handler(int irq, void *dev_id) 
{
    if (dev_id != &hvisor_misc_dev) {
        return IRQ_NONE;
    }
    struct siginfo info;

    memset(&info, 0, sizeof(struct siginfo));
    info.si_signo = SIGHVI;
    info.si_code = SI_QUEUE;
    info.si_int = 1;
    // Send signale SIGHVI to hvisor user task
    if (task != NULL) {
        // pr_info("send signal to hvisor device\n");
        if(send_sig_info(SIGHVI, (struct kernel_siginfo *)&info, task) < 0) {
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
    err = misc_register(&hvisor_misc_dev);
    if (err) {
        pr_err("hvisor_misc_register failed!!!\n");
        return err;
    }
	// The irq number must be retrieved from dtb node, because it is different from GIC's IRQ number.
    struct device_node *node = NULL;
    node = of_find_node_by_path("/hvisor_device");
    if (!node) {
        pr_err("hvisor_device not found\n");
        return -1;
    }

    hvisor_irq = of_irq_get(node, 0);
    err = request_irq(hvisor_irq, irq_handler, IRQF_SHARED | IRQF_TRIGGER_RISING, "hvisor", &hvisor_misc_dev);
    if (err) {
        pr_err("hvisor cannot register IRQ, err is %d\n", err);
    	free_irq(hvisor_irq,(void *)(irq_handler));
        return -1;
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

	free_irq(hvisor_irq,(void *)(irq_handler));

	ClearPageReserved(virt_to_page(virtio_bridge));

	free_pages((unsigned long)virtio_bridge, 0);
	
    pr_info("hvisor exit!!!\n");
}
 
module_init(hvisor_init);
module_exit(hvisor_exit);
 
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KouweiLee <15035660024@163.com>");
MODULE_DESCRIPTION("The hvisor device driver");
MODULE_VERSION("1:0.0");