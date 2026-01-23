## UR-DP1000

Physical PLIC has one hardware bug(claim register).
But Virtual PLIC can be treated as a standard PLIC, this can improve some performance.

```c
// https://github.com/RVCK-Project/rvck/blob/rvck-6.6/drivers/irqchip/irq-sifive-plic.c

static bool plic_check_enable_first_pending(u32 ie[])
{
        struct plic_handler *handler = this_cpu_ptr(&plic_handlers);
        void __iomem *enable = handler->enable_base;
        void __iomem *pending = handler->priv->regs + PENDING_BASE;
        int nr_irqs = handler->priv->nr_irqs;
        int nr_irq_groups = (nr_irqs + 31) / 32;
        bool is_pending = false;
        int i, j;
        raw_spin_lock(&handler->enable_lock);
        // Read current interrupt enables
        for (i = 0; i < nr_irq_groups; i++)
                ie[i] = readl(enable + i * sizeof(u32));
        // Check for pending interrupts and enable only the first one found
        for (i = 0; i < nr_irq_groups; i++) {
                u32 pending_irqs = readl(pending + i * sizeof(u32)) & ie[i];
                if (pending_irqs) {
                        int nbit = __ffs(pending_irqs);
                        for (j = 0; j < nr_irq_groups; j++)
                                writel((i == j)?(1 << nbit):0, enable + j * sizeof(u32));
                        is_pending = true;
                        break;
                }
        }
        raw_spin_unlock(&handler->enable_lock);
        return is_pending;
}

static void plic_restore_enable_state(u32 ie[])
{
        struct plic_handler *handler = this_cpu_ptr(&plic_handlers);
        void __iomem *enable = handler->enable_base;
        int nr_irqs = handler->priv->nr_irqs;
        int nr_irq_groups = (nr_irqs + 31) / 32;
        int i;

        raw_spin_lock(&handler->enable_lock);

        for (i = 0; i < nr_irq_groups; i++)                        // restore original enable bits
                writel(ie[i], enable + i * sizeof(u32));

        raw_spin_unlock(&handler->enable_lock);
}


static irq_hw_number_t plic_get_hwirq(void)
{
        struct plic_handler *handler = this_cpu_ptr(&plic_handlers);
        struct plic_priv *priv = handler->priv;
        void __iomem *claim = handler->hart_base + CONTEXT_CLAIM;
        irq_hw_number_t hwirq;
        u32 ie[32] = {0};    // 32x32 = 1024
        /*
         * Due to the implementation of the claim register in the UltraRISC DP1000
         * platform PLIC not conforming to the specification, this is a hardware
         * bug. Therefore, when an interrupt is pending, we need to disable the other
         * interrupts before reading the claim register. After processing the interrupt,
         * we should then restore the enable register.
         */
        if (test_bit(PLIC_QUIRK_CLAIM_REGISTER, &priv->plic_quirks)) {
                hwirq = plic_check_enable_first_pending(ie)?readl(claim):0;
                plic_restore_enable_state(ie);
        } else {
                hwirq = readl(claim);
        }
        return hwirq;
}
```