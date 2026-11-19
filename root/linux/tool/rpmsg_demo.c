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
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <sched.h>
#include <signal.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <unistd.h>

#include <time.h>

#include <linux/rpmsg.h>

#define RPMSG_MASTER_ADDR (40)
#define RPMSG_REMOTE_ADDR (30)

static int g_efd = 0, g_local_endpt = 0;

static int rpmsg_create_eptdev(char *eptdev_name, int local_port,
                               int remote_port) {
    int fd, ret;
    struct rpmsg_endpoint_info ept_info = {0};

    fd = open("/dev/rpmsg_ctrl0", O_RDWR);
    if (fd < 0) {
        fprintf(stderr, "%s: could not open rpmsg_ctrl0 dev node\n", __func__);
        return fd;
    }

    ept_info.src = local_port;
    ept_info.dst = remote_port;
    sprintf(ept_info.name, "%s", eptdev_name);

    ret = ioctl(fd, RPMSG_CREATE_EPT_IOCTL, &ept_info);
    if (ret) {
        fprintf(stderr, "%s: ctrl_fd ioctl: %s\n", __func__, strerror(errno));
    }

    // close(fd);

    return ret;
}

static int rpmsg_open_eptdev(char *eptdev_name, int flags) {
    (void)eptdev_name;
    int efd;

    efd = open("/dev/rpmsg0", O_RDWR | flags);
    if (efd < 0) {
        fprintf(stderr, "failed to open eptdev /dev/rpmsg0\n");
        return -errno;
    }

    g_efd = efd;
    g_local_endpt = RPMSG_MASTER_ADDR;

    return 0;
}

int send_msg(int fd, char *msg, int len) {
    int ret = 0;

    ret = write(fd, msg, len);
    if (ret < 0) {
        perror("Can't write to rpmsg endpt device\n");
        return -1;
    }

    return ret;
}

int recv_msg(int fd, int len, char *reply_msg, int *reply_len) {
    int ret = 0;

    /* Note: len should be max length of response expected */
    ret = read(fd, reply_msg, len);
    if (ret < 0) {
        perror("Can't read from rpmsg endpt device\n");
        return -1;
    } else {
        *reply_len = ret;
    }

    return 0;
}

static int rpmsg_destroy_eptdev(int fd) {
    int ret;

    ret = ioctl(fd, RPMSG_DESTROY_EPT_IOCTL, NULL);
    if (ret) {
        fprintf(stderr, "%s: could not destroy endpt %d\n", __func__, fd);
        return ret;
    }

    close(fd);
    return 0;
}
int main() {
    int ret, packet_len;
    char packet_buf[512] = {0};
    printf("rpmsg_demo start\n");
    // ret = rpmsg_create_eptdev("rpmsg-master-a53", RPMSG_MASTER_ADDR,
    // RPMSG_REMOTE_ADDR);
    ret = rpmsg_create_eptdev("rpmsg-remote-m7", RPMSG_MASTER_ADDR,
                              RPMSG_REMOTE_ADDR);

    if (ret) {
        fprintf(stderr, "%s: could not create end-point rpmsg-master-a53\n",
                __func__);
        return ret;
    }
    printf("rpmsg_crtl open success\n");
    ret = rpmsg_open_eptdev("rpmsg-remote-m7", 0);
    printf("rpmsg_optdev open success\n");
    memset(packet_buf, 0, sizeof(packet_buf));
    sprintf(packet_buf, "hello, I am a53!");
    packet_len = strlen(packet_buf);
    printf("Sending message: %s\n", packet_buf);
    ret = send_msg(g_efd, (char *)packet_buf, packet_len);
    if (ret < 0) {
        printf("send_msg failed, ret = %d\n", ret);
        goto out;
    }

    memset(packet_buf, 0, sizeof(packet_buf));
    ret = recv_msg(g_efd, 256, (char *)packet_buf, &packet_len);
    printf("Receiving message: %s\n", packet_buf);
    if (ret < 0) {
        printf("recv_msg failed, ret = %d\n", ret);
        goto out;
    }
    return 0;

out:
    ret = rpmsg_destroy_eptdev(g_efd);
    if (ret < 0)
        perror("Can't delete the endpoint device\n");

    return ret;
}