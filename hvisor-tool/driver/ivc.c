#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/mm.h>
#include <linux/miscdevice.h>
#include <linux/sched/signal.h>
#include <linux/of.h>
#include <linux/of_irq.h>
#include <linux/interrupt.h>

#include "ivc.h"
int ivc_len;
__u64 ivc_ct_ipas[CONFIG_MAX_IVC_CONFIGS];
int ivc_irq = -1;

static struct task_struct *task = NULL;

static int hvisor_ivc_info(ivc_info_t __user *arg)
{
    int err = 0, i;
    ivc_info_t *ivc_info;
    ivc_info = kmalloc(sizeof(ivc_info_t), GFP_KERNEL);

    err = hvisor_call(HVISOR_HC_IVC_INFO, __pa(ivc_info), sizeof(ivc_info_t));
    ivc_len = ivc_info->len;
    for(i=0; i<ivc_len; i++) 
        ivc_ct_ipas[i] = ivc_info->ivc_ct_ipas[i];
    copy_to_user(arg, ivc_info, sizeof(ivc_info_t));
    return err;
}

static long ivc_ioctl(struct file *file, unsigned int ioctl,
                         unsigned long arg)
{
    int err = 0;
    switch (ioctl)
    {
    case HVISOR_IVC_INFO:
        err = hvisor_ivc_info((ivc_info_t __user *)arg);
        task = get_current();
        break;
    default:
        err = -EINVAL;
        break;
    }
    return err;
}

static int ivc_map(struct file *filp, struct vm_area_struct *vma)
{
    unsigned long long phys;
    int i, err = 0, is_control_table = 0;
    size_t size = vma->vm_end - vma->vm_start;
    phys = vma->vm_pgoff;
    for(i=0; i<ivc_len; i++) {
        if (phys == ivc_ct_ipas[i]) {
            is_control_table = true;
            break;
        }
    }
    if (is_control_table)
        vma->vm_page_prot = pgprot_noncached(vma->vm_page_prot);
    // TODO: add check for memory
    err = remap_pfn_range(vma,
                        vma->vm_start,
                        vma->vm_pgoff,
                        size,
                        vma->vm_page_prot);
    if (err)
        return err;
    pr_info("ivc region mmap succeed!\n");
    return 0;
}
static const struct file_operations ivc_fops = {
    .owner = THIS_MODULE,
    .unlocked_ioctl = ivc_ioctl,
    .compat_ioctl = ivc_ioctl,
    .mmap = ivc_map,
};

static struct miscdevice ivc_dev = {
    .minor = MISC_DYNAMIC_MINOR,
    .name = "hvisorivc",
    .fops = &ivc_fops,
};

static irqreturn_t ivc_irq_handler(int irq, void *dev_id)
{
    if (dev_id != &ivc_dev)
        return IRQ_NONE;
    struct siginfo info;
    memset(&info, 0, sizeof(struct siginfo));
    info.si_signo = SIGIVC;
    info.si_code = SI_QUEUE;
    info.si_int = 1;
    if (task != NULL)
    {
        // pr_info("send signal to hvisor device\n");
        if (send_sig_info(SIGIVC, (struct kernel_siginfo *)&info, task) < 0)
        {
            pr_err("Unable to send signal\n");
        }
    }
    return IRQ_HANDLED;
}

static int __init ivc_init(void) {
    int err;
    struct device_node *node = NULL;
    misc_register(&ivc_dev);

    // probe ivc device
    node = of_find_node_by_path("/hvisor_ivc_device");
    if (!node) {
        pr_info("hvisor_ivc_device node not found in dtb, can't use ivc\n");
    } else {
        ivc_irq = of_irq_get(node, 0);
        err = request_irq(ivc_irq, ivc_irq_handler, IRQF_SHARED | IRQF_TRIGGER_RISING, "hvisor_ivc_device", &ivc_dev);
        if (err) goto err_out;
    }
    
    pr_info("ivc init!!!\n");
    return 0;
err_out:
    if (ivc_irq != -1) 
        free_irq(ivc_irq, &ivc_dev);
    misc_deregister(&ivc_dev);
}

static void __exit ivc_exit(void)
{
    if (ivc_irq != -1) 
        free_irq(ivc_irq, &ivc_dev);
    misc_deregister(&ivc_dev);
    pr_info("ivc exit done!!!\n");
}
module_init(ivc_init);
module_exit(ivc_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("KouweiLee <15035660024@163.com>");
MODULE_DESCRIPTION("The hvisor device driver");
MODULE_VERSION("1:0.0");