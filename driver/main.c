#include<linux/kernel.h>
#include<linux/init.h>
#include<linux/module.h>
#include<linux/miscdevice.h>
#include<linux/mm.h>
#include<linux/interrupt.h>
#include<linux/slab.h>   
// #include <linux/ioctl.h>
#include <asm/io.h>
#include "hvisor.h"
#include <linux/sched/signal.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <asm/page.h>
#include <linux/gfp.h>
// #include <linux/fs.h>
// #include <linux/err.h>
static struct task_struct *task = NULL;

struct hvisor_device_region *device_region; 

// initial virtio el2 shared region
int hvisor_init_virtio(void) 
{
	int err;
    // unsigned long pa = __get_free_pages(GFP_KERNEL, 0);
	// device_region = kmalloc(MMAP_SIZE, GFP_KERNEL);
    // pr_info("device_region pa is %x\n", pa);
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
int hvisor_finish_req(void) 
{
    pr_info("hvisor finish request\n");
    int err;
    err = hvisor_call(HVISOR_HC_FINISH_REQ);
    if (err)
        return err;
    return 0;
}

static long hvisor_ioctl(struct file *file, unsigned int ioctl,
			    unsigned long arg)
{
    switch (ioctl)
    {
    case HVISOR_INIT_VIRTIO:
        hvisor_init_virtio(); 
        task = get_current(); // get hvisor user process
        break;
    case HVISOR_FINISH:
        hvisor_finish_req();
        break;
    default:
        break;
    }
    return 0;
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

// Interrupt handler for IRQ.
static irqreturn_t irq_handler(int irq, void *dev_id) 
{
    if (dev_id != &hvisor_misc_dev) {
        return IRQ_NONE;
    }
    struct siginfo info;
    pr_info("el2 IRQ occurred\n");

    memset(&info, 0, sizeof(struct siginfo));
    info.si_signo = SIGHVI;
    info.si_code = SI_QUEUE;
    info.si_int = 1;
    // Send signale SIGHVI to hvisor user task
    if (task != NULL) {
        pr_info("send signal to hvisor device\n");
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
//    struct device_node *node = NULL;
//    node = of_find_node_by_path("/vm_service");
//    if (!node) {
//        pr_err("vm_service not found\n");
//        return -1;
//    }
//
//    int irq = of_irq_get(node, 0);
//    err = request_irq(irq, irq_handler, IRQF_SHARED | IRQF_TRIGGER_RISING, "hvisor", &hvisor_misc_dev);
//    if (err) {
//        pr_err("hvisor cannot register IRQ, err is %d\n", err);
//        goto irq;
//    }
    printk("hvisor init done!!!\n");
    return 0;

//irq:
//    free_irq(irq,(void *)(irq_handler));
//    return -1;
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