#include "server.h"
#include <linux/of.h>

static struct device_node *hvisor_np;

struct device_node *hvisor_get_node(void) {
    if (!hvisor_np)
        hvisor_np = of_find_compatible_node(NULL, NULL, "hvisor");
    return hvisor_np;
}

void hvisor_put_node(void) {
    if (hvisor_np) {
        of_node_put(hvisor_np);
        hvisor_np = NULL;
    }
}

/* Forward declarations of per-domain cleanup (defined in clock.c, reset.c,
 * power.c) */
extern void clock_ctrl_finish(void);
extern void reset_ctrl_finish(void);
extern void power_ctrl_finish(void);

int hvisor_scmi_init(void) {
    int ret;
    ret = clock_init();
    if (ret)
        return ret;
    ret = reset_init();
    if (ret) {
        clock_ctrl_finish();
        return ret;
    }
    ret = power_init();
    if (ret) {
        reset_ctrl_finish();
        clock_ctrl_finish();
        return ret;
    }
    return 0;
}

void hvisor_scmi_cleanup(void) {
    power_ctrl_finish();
    reset_ctrl_finish();
    clock_ctrl_finish();
}
