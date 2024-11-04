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
struct ivc_info {
	__u64 len;
	__u64 ivc_ct_ipas[CONFIG_MAX_IVC_CONFIGS];
    __u64 ivc_shmem_ipas[CONFIG_MAX_IVC_CONFIGS];
	__u32 ivc_ids[CONFIG_MAX_IVC_CONFIGS];
}__attribute__((packed));
typedef struct ivc_info ivc_info_t;

ivc_info_t *ivc_info;
int ivc_irq = -1;

static struct task_struct *task = NULL;

static int hvisor_ivc_info(void)
{
    int err = 0, i;
    if(ivc_info == NULL)
        ivc_info = kmalloc(sizeof(ivc_info_t), GFP_KERNEL);
    err = hvisor_call(HVISOR_HC_IVC_INFO, __pa(ivc_info), sizeof(ivc_info_t));
    if (ivc_info->len > 1) {
        pr_warn("Now we can only support 1 ivc_id\n");
    }
    return err;
}

static int ivc_open(struct inode *inode, struct file *file) {
    int err;
    if (ivc_info == NULL)
        err = hvisor_ivc_info();
    task = get_current();
    return 0;
}

static int hvisor_user_ivc_info(ivc_uinfo_t __user* uinfo) {
    int err = 0, i;
    uinfo->len = ivc_info->len;
    for(i = 0; i < ivc_info->len; i++) {
        uinfo->ivc_ids[i] = ivc_info->ivc_ids[i];
    }
    return 0;
}

static long ivc_ioctl(struct file *file, unsigned int ioctl,
                         unsigned long arg)
{
    int err = 0;
    switch (ioctl)
    {
    case HVISOR_IVC_USER_INFO:
        err = hvisor_user_ivc_info((ivc_uinfo_t __user*)arg);
        break;
    default:
        err = -EINVAL;
        break;
    }
    return err;
}

static int ivc_map(struct file *filp, struct vm_area_struct *vma)
{
    unsigned long long phys, offset;
    int i, err = 0, is_control_table = 0;
    size_t size = vma->vm_end - vma->vm_start;
    phys = vma->vm_pgoff << PAGE_SHIFT;
    // TODO: Now we can only support ivc_id is 0.
    if (phys == 0) {
        // control table
        if(size != 0x1000) {
            pr_err("Invalid size for control table\n");
            return -EINVAL;
        }
        vma->vm_page_prot = pgprot_noncached(vma->vm_page_prot);
        err = remap_pfn_range(vma,
                    vma->vm_start,
                    ivc_info->ivc_ct_ipas[0] >> PAGE_SHIFT,
                    size,
                    vma->vm_page_prot);
    } else {
        // TODO: add check for memory
        offset = phys - 0x1000;
        err = remap_pfn_range(vma,
            vma->vm_start,
            (ivc_info->ivc_shmem_ipas[0] + offset) >> PAGE_SHIFT,
            size,
            vma->vm_page_prot);
    }
    if (err)
        return err;
    pr_info("ivc region mmap succeed!\n");
    return 0;
}
static const struct file_operations ivc_fops = {
    .owner = THIS_MODULE,
    .open = ivc_open,
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
    of_node_put(node);
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