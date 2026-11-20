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
#include <linux/cdev.h>
#include <linux/device.h>
#include <linux/init.h>
#include <linux/interrupt.h>
#include <linux/kdev_t.h>
#include <linux/kernel.h>
#include <linux/miscdevice.h>
#include <linux/mm.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_fdt.h>
#include <linux/of_irq.h>
#include <linux/poll.h>
#include <linux/sched/signal.h>
#include <linux/slab.h>
#include <linux/types.h>
#include <linux/uaccess.h>
#include <linux/version.h>
#include <linux/wait.h>

#include "hvisor.h"
#include "ivc.h"

#ifdef ARM64

struct ivc_info {
    __u64 len;
    __u64 ivc_ct_ipas[CONFIG_MAX_IVC_CONFIGS];
    __u64 ivc_shmem_ipas[CONFIG_MAX_IVC_CONFIGS];
    __u32 ivc_ids[CONFIG_MAX_IVC_CONFIGS];
    __u32 ivc_irqs[CONFIG_MAX_IVC_CONFIGS];
} __attribute__((packed));
typedef struct ivc_info ivc_info_t;

struct ivc_dev {
    dev_t dev_id;
    struct cdev cdev;
    struct device *device;
    struct task_struct *task;
    wait_queue_head_t wq;
    int idx;
    int ivc_id;
    int ivc_irq;
    int received_irq; // receive irq count
};

ivc_info_t *ivc_info;

dev_t mdev_id;
static int dev_len;
static struct ivc_dev *ivc_devs;
static struct class *ivc_class;

extern u8 __dtb_hivc_template_begin[], __dtb_hivc_template_end[];

static int hvisor_ivc_info(void) {
    int err = 0;
    if (ivc_info == NULL) {
        ivc_info = kzalloc(sizeof(ivc_info_t), GFP_KERNEL);
        if (!ivc_info)
            return -ENOMEM;
    }
    err = hvisor_call(HVISOR_HC_IVC_INFO, __pa(ivc_info), sizeof(ivc_info_t));
    if (err)
        return err;
    if (ivc_info->len == 0 || ivc_info->len > CONFIG_MAX_IVC_CONFIGS)
        return -EINVAL;
    return err;
}

static int ivc_open(struct inode *inode, struct file *file) {
    struct ivc_dev *dev = container_of(inode->i_cdev, struct ivc_dev, cdev);
    dev->task = get_current();
    file->private_data = dev;
    return 0;
}

static int hvisor_user_ivc_info(ivc_uinfo_t __user *uinfo) {
    int i;
    uinfo->len = ivc_info->len;
    for (i = 0; i < ivc_info->len; i++) {
        uinfo->ivc_ids[i] = ivc_info->ivc_ids[i];
    }
    return 0;
}

static long ivc_ioctl(struct file *file, unsigned int ioctl,
                      unsigned long arg) {
    int err = 0;
    switch (ioctl) {
    case HVISOR_IVC_USER_INFO:
        err = hvisor_user_ivc_info((ivc_uinfo_t __user *)arg);
        break;
    default:
        err = -EINVAL;
        break;
    }
    return err;
}

static int ivc_map(struct file *filp, struct vm_area_struct *vma) {
    unsigned long long phys, offset;
    int err = 0, idx;
    size_t size = vma->vm_end - vma->vm_start;
    struct ivc_dev *dev = filp->private_data;
    idx = dev->idx;
    phys = vma->vm_pgoff << PAGE_SHIFT;

    if (phys == 0) {
        // control table
        if (size != 0x1000) {
            pr_err("Invalid size for control table\n");
            return -EINVAL;
        }
        vma->vm_page_prot = pgprot_noncached(vma->vm_page_prot);
        err = remap_pfn_range(vma, vma->vm_start,
                              ivc_info->ivc_ct_ipas[idx] >> PAGE_SHIFT, size,
                              vma->vm_page_prot);
    } else {
        // TODO: add check for memory
        offset = phys - 0x1000;
        err = remap_pfn_range(vma, vma->vm_start,
                              (ivc_info->ivc_shmem_ipas[idx] + offset) >>
                                  PAGE_SHIFT,
                              size, vma->vm_page_prot);
    }
    if (err)
        return err;
    pr_info("ivc region mmap succeed!\n");
    return 0;
}

static unsigned int ivc_poll(struct file *filp,
                             struct poll_table_struct *wait) {
    __poll_t mask = 0;
    struct ivc_dev *this_dev = (struct ivc_dev *)filp->private_data;
    poll_wait(filp, &this_dev->wq, wait);
    if (this_dev->received_irq) {
        mask |= POLLIN;
        this_dev->received_irq = 0;
    }
    return mask;
}

static const struct file_operations ivc_fops = {
    .owner = THIS_MODULE,
    .open = ivc_open,
    .unlocked_ioctl = ivc_ioctl,
    .compat_ioctl = ivc_ioctl,
    .mmap = ivc_map,
    .poll = ivc_poll,
};

static irqreturn_t ivc_irq_handler(int irq, void *dev_id) {
    int i;
    struct ivc_dev *this_dev = NULL;
    for (i = 0; i < dev_len; i++)
        if (dev_id == &ivc_devs[i])
            this_dev = (struct ivc_dev *)dev_id;
    if (!this_dev)
        return IRQ_NONE;
    this_dev->received_irq++;
    wake_up(&this_dev->wq);
    return IRQ_HANDLED;
}

static int __init ivc_init(void) {
    int err, i, soft_irq;
    struct device_node *node = NULL;
    err = hvisor_ivc_info();
    if (err)
        goto err1;
    dev_len = ivc_info->len;

    pr_info("ivc: hypervisor reports %d channel(s)\n", dev_len);
    ivc_devs = kcalloc(dev_len, sizeof(struct ivc_dev), GFP_KERNEL);
    if (!ivc_devs) {
        err = -ENOMEM;
        goto err1;
    }
    err = alloc_chrdev_region(&mdev_id, 0, dev_len, "hivc");
    if (err)
        goto err1;
    pr_info("ivc get major id: %d\n", MAJOR(mdev_id));

#if LINUX_VERSION_CODE >= KERNEL_VERSION(6, 4, 0)
    // Linux 6.4+: class_create(name) - kernel commit 1aaba11da9aa
    ivc_class = class_create("hivc");
#else
    // older: class_create(THIS_MODULE, name)
    ivc_class = class_create(THIS_MODULE, "hivc");
#endif
    if (IS_ERR(ivc_class)) {
        err = PTR_ERR(ivc_class);
        goto err1;
    }

    for (i = 0; i < dev_len; i++) {
        ivc_devs[i].ivc_id = ivc_info->ivc_ids[i];
        ivc_devs[i].dev_id = MKDEV(MAJOR(mdev_id), i);
        ivc_devs[i].idx = i;
        ivc_devs[i].ivc_irq = -1;
        ivc_devs[i].cdev.owner = THIS_MODULE;
        ivc_devs[i].received_irq = 0;
        init_waitqueue_head(&ivc_devs[i].wq);
        cdev_init(&ivc_devs[i].cdev, &ivc_fops);
        err = cdev_add(&ivc_devs[i].cdev, ivc_devs[i].dev_id, 1);
        if (err)
            goto err2;
        ivc_devs[i].device = device_create(ivc_class, NULL, ivc_devs[i].dev_id,
                                           NULL, "hivc%d", ivc_devs[i].ivc_id);
        if (IS_ERR(ivc_devs[i].device)) {
            err = PTR_ERR(ivc_devs[i].device);
            goto err2;
        }
    }
    node = of_find_node_by_path("/hvisor_ivc_device");
    if (!node) {
        // add_ivc_device_node();
        pr_info("hvisor_ivc_device node not found in dtb, can't use ivc\n");
    } else {
        for (i = 0; i < dev_len; i++) {
            soft_irq = of_irq_get(node, i);
            if (soft_irq < 0) {
                pr_err("ivc: of_irq_get channel %d failed (%d)\n", i, soft_irq);
                continue;
            }
            err = request_irq(soft_irq, ivc_irq_handler,
                              IRQF_SHARED | IRQF_TRIGGER_RISING,
                              "hvisor_ivc_device", &ivc_devs[i]);
            if (err) {
                pr_err("ivc: request irq channel %d failed (%d)\n", i, err);
                goto err2;
            }
            ivc_devs[i].ivc_irq = soft_irq;
            pr_info("ivc: ch%d id=%d linux_irq=%d hv_irq=%u\n", i,
                    ivc_devs[i].ivc_id, soft_irq, ivc_info->ivc_irqs[i]);
        }
    }
    of_node_put(node);
    pr_info("ivc init!!!\n");
    return 0;

err2:
    for (i = 0; i < dev_len; i++) {
        if (ivc_devs[i].ivc_irq >= 0) {
            free_irq(ivc_devs[i].ivc_irq, &ivc_devs[i]);
            ivc_devs[i].ivc_irq = -1;
        }
        cdev_del(&ivc_devs[i].cdev);
        if (ivc_class && !IS_ERR(ivc_class))
            device_destroy(ivc_class, ivc_devs[i].dev_id);
    }
    class_destroy(ivc_class);
    unregister_chrdev_region(mdev_id, dev_len);
err1:
    kfree(ivc_info);
    kfree(ivc_devs);
    return err;
}

static void __exit ivc_exit(void) {
    int i;

    if (!ivc_devs || dev_len <= 0) {
        kfree(ivc_info);
        ivc_info = NULL;
        pr_info("ivc exit done!!!\n");
        return;
    }
    /* Free the IRQ first to prevent user-space and kernel paths from being
     * woken. */
    for (i = 0; i < dev_len; i++) {
        if (ivc_devs[i].ivc_irq >= 0) {
            free_irq(ivc_devs[i].ivc_irq, &ivc_devs[i]);
            ivc_devs[i].ivc_irq = -1;
        }
    }

    /* Destroy the character devices and sysfs device nodes. */
    if (ivc_class && !IS_ERR(ivc_class)) {
        for (i = 0; i < dev_len; i++) {
            cdev_del(&ivc_devs[i].cdev);
            device_destroy(ivc_class, ivc_devs[i].dev_id);
        }
        class_destroy(ivc_class);
    }
    unregister_chrdev_region(mdev_id, dev_len);

    kfree(ivc_devs);
    ivc_devs = NULL;
    kfree(ivc_info);
    ivc_info = NULL;
    dev_len = 0;

    pr_info("ivc exit done!!!\n");
}

#else
// for other architecture we implement empty functions
// because different linux versions has different kernel interfaces
// TODO: add support for other architectures
static int __init ivc_init(void) { return 0; }
static void __exit ivc_exit(void) { return; }

#endif

module_init(ivc_init);
module_exit(ivc_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("KouweiLee <15035660024@163.com>");
MODULE_DESCRIPTION("The hvisor device driver");
MODULE_VERSION("1:0.0");
