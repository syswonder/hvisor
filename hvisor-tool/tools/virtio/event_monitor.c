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
#define _GNU_SOURCE
#include <errno.h>
#include <pthread.h>
#include <sched.h>
#include <stdlib.h>
#include <unistd.h>

#include "event_monitor.h"
#include "hvisor.h"
#include "log.h"

static int epoll_fd;
static int events_num;
pthread_t emonitor_tid;
int closing;
#define MAX_EVENTS MAX_DEVS
struct hvisor_event *events[MAX_EVENTS];
static pthread_mutex_t event_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t event_done = PTHREAD_COND_INITIALIZER;

static struct hvisor_event *find_event_locked(int fd) {
    for (int i = 0; i < events_num; i++)
        if (events[i] && events[i]->fd == fd)
            return events[i];
    return NULL;
}

static void *epoll_loop(void *arg) {
    (void)arg;
    struct epoll_event ready[MAX_EVENTS];
    int ret;
    for (;;) {
        ret = epoll_wait(epoll_fd, ready, MAX_EVENTS, -1);
        if (ret < 0) {
            if (errno == EINTR)
                continue;
            log_error("epoll_wait failed, errno is %d", errno);
            break;
        }
        for (int i = 0; i < ret; ++i) {
            /*
             * epoll_event.data.ptr would become stale if remove_event()
             * freed the event after epoll_wait returned.  Look the event up
             * by fd under event_lock instead, and hold a per-event active
             * count while the handler runs.
             */
            int fd = ready[i].data.fd;
            pthread_mutex_lock(&event_lock);
            struct hvisor_event *hevent = find_event_locked(fd);
            if (!hevent) {
                pthread_mutex_unlock(&event_lock);
                continue;
            }
            hevent->active++;
            pthread_mutex_unlock(&event_lock);

            hevent->handler(fd, ready[i].events, hevent->param);

            pthread_mutex_lock(&event_lock);
            hevent->active--;
            if (hevent->active == 0)
                pthread_cond_broadcast(&event_done);
            pthread_mutex_unlock(&event_lock);
        }
    }
    pthread_exit(NULL);
    return NULL;
}

struct hvisor_event *add_event(int fd, int epoll_type,
                               void (*handler)(int, int, void *), void *param) {
    struct hvisor_event *hevent;
    struct epoll_event eevent;
    int ret;
    if (fd < 0 || handler == NULL) {
        log_error("invalid fd or handler");
        return NULL;
    }
    hevent = calloc(1, sizeof(struct hvisor_event));
    hevent->handler = handler;
    hevent->param = param;
    hevent->fd = fd;
    hevent->epoll_type = epoll_type;
    hevent->active = 0;

    eevent.events = epoll_type;
    eevent.data.fd = fd;

    pthread_mutex_lock(&event_lock);
    int slot = -1;
    for (int i = 0; i < MAX_EVENTS; i++) {
        if (!events[i]) {
            slot = i;
            break;
        }
    }
    if (slot < 0) {
        pthread_mutex_unlock(&event_lock);
        log_error("events are full");
        free(hevent);
        return NULL;
    }
    ret = epoll_ctl(epoll_fd, EPOLL_CTL_ADD, hevent->fd, &eevent);
    if (ret < 0) {
        pthread_mutex_unlock(&event_lock);
        log_error("epoll_ctl failed, errno is %d", errno);
        free(hevent);
        return NULL;
    }
    events[slot] = hevent;
    if (slot >= events_num)
        events_num = slot + 1;
    pthread_mutex_unlock(&event_lock);
    return hevent;
}

// Create a thread monitoring events.
int initialize_event_monitor() {
    epoll_fd = epoll_create1(0);
    log_debug("create epoll_fd %d", epoll_fd);
    pthread_create(&emonitor_tid, NULL, epoll_loop, NULL);
    cpu_set_t cpuset;
    CPU_ZERO(&cpuset);
    if (sched_getaffinity(0, sizeof(cpu_set_t), &cpuset) == 0) {
        int last_cpu = -1;
        for (int i = CPU_SETSIZE - 1; i >= 1; i--) {
            if (CPU_ISSET(i, &cpuset)) {
                last_cpu = i;
                break;
            }
        }

        if (last_cpu != -1) {
            cpu_set_t set;
            CPU_ZERO(&set);
            CPU_SET(last_cpu, &set);
            if (pthread_setaffinity_np(emonitor_tid, sizeof(cpu_set_t), &set) !=
                0) {
                log_warn("failed to set epoll_loop thread to cpu %d", last_cpu);
            } else {
                log_info("epoll_loop thread set to cpu %d", last_cpu);
            }
        } else {
            log_warn("No available CPU other than CPU0");
        }
    } else {
        log_warn("failed to get cpu affinity: %d", errno);
    }
    if (epoll_fd >= 0)
        return 0;
    else {
        log_error("hvisor_event init failed");
        return -1;
    }
}

void remove_event(struct hvisor_event *hevent) {
    if (!hevent)
        return;

    pthread_mutex_lock(&event_lock);
    for (int i = 0; i < events_num; i++) {
        if (events[i] == hevent) {
            epoll_ctl(epoll_fd, EPOLL_CTL_DEL, hevent->fd, NULL);
            while (hevent->active > 0)
                pthread_cond_wait(&event_done, &event_lock);
            events[i] = NULL;
            while (events_num > 0 && events[events_num - 1] == NULL)
                events_num--;
            pthread_mutex_unlock(&event_lock);
            return;
        }
    }
    pthread_mutex_unlock(&event_lock);
}

void destroy_event_monitor() {
    int i;
    for (i = 0; i < events_num; i++) {
        if (events[i] != NULL)
            epoll_ctl(epoll_fd, EPOLL_CTL_DEL, events[i]->fd, NULL);
        events[i] = NULL;
    }
    close(epoll_fd);
    // When the main thread exits, the epoll thread will also exit. Therefore,
    // we do not directly terminate the epoll thread here.
}
