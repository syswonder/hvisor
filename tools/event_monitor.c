#include "event_monitor.h"
#include "log.h"
#include <pthread.h>
#include <errno.h>
#include <stdlib.h>
static int epoll_fd;
#define	MAX_EVENTS	16
static void *epoll_loop(void *param)
{
    struct epoll_event events[MAX_EVENTS];
    struct hvisor_event *hevent;
    int ret, i;
    for (;;) {
        ret = epoll_wait(epoll_fd, events, MAX_EVENTS, -1);
        if (ret == -1 && errno != EINTR)
            log_error("epoll_wait failed, errno is %d", errno);
        for (i = 0; i < ret; ++i) {
            // handle active hvisor_event
            hevent = events[i].data.ptr;
            if (hevent == NULL) 
                log_error("hevent shouldn't be null");
			hevent->handler(hevent->fd, hevent->epoll_type, hevent->param);
        }
    }
	return NULL;
}

struct hvisor_event *add_event(int fd, int epoll_type,
        void (*handler)(int, int, void *), void *param)
{
    struct hvisor_event *hevent;
    struct epoll_event eevent;
    int ret;
    if (fd < 0 || handler == NULL)
        return NULL;
    hevent = calloc(1, sizeof(struct hvisor_event));
    if (hevent == NULL)
        return NULL;
	hevent->handler = handler;
    hevent->param = param;
    hevent->fd = fd;
    hevent->epoll_type = epoll_type;

    eevent.events = epoll_type;
    eevent.data.ptr = hevent;
    ret = epoll_ctl(epoll_fd, EPOLL_CTL_ADD, hevent->fd, &eevent);
    if (ret < 0) {
        log_error("epoll_ctl failed, errno is %d", errno);
        free(hevent);
        return NULL;
    }
    else {
        return hevent;
    }
}

// Create a thread monitoring events.
int initialize_event_monitor()
{
    epoll_fd = epoll_create1(0);
    log_debug("create epoll_fd is %d", epoll_fd);
    pthread_t thread_id;
    pthread_create(&thread_id, NULL, epoll_loop, NULL);
    if (epoll_fd >= 0)
        return 0;
    else {
        log_error("hvisor_event init failed");
        return -1;
    }
}

