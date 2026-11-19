// SPDX-License-Identifier: GPL-2.0-only
#include <stdio.h>
#include <string.h>

#include "log.h"
#include "virtio.h"

int main(int argc, char *argv[]) {
    multithread_log_init();
    initialize_log();

    if (argc < 3 || strcmp(argv[1], "virtio") != 0) {
        fprintf(stderr, "usage: hvisor-virtio-daemon virtio start|add <config.json>\n");
        return 1;
    }

    if (strcmp(argv[2], "start") == 0)
        return virtio_start(argc, argv) ? 1 : 0;

    if (strcmp(argv[2], "add") == 0)
        return virtio_add(argc, argv) ? 1 : 0;

    fprintf(stderr, "usage: hvisor-virtio-daemon virtio start|add <config.json>\n");
    return 1;
}
