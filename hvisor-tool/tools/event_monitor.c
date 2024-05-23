#include "event_monitor.h"
#include "log.h"
#include <pthread.h>
#include <errno.h>
#include <stdlib.h>
#include <unistd.h>
static int epoll_fd;
static int events_num;
pthread_t emonitor_tid;
int closing;
#define	MAX_EVENTS	16
struct hvisor_event *events[MAX_EVENTS];
static void *epoll_loop()
{
    struct epoll_event events[MAX_EVENTS];
    struct hvisor_event *hevent;
    int ret, i;
    for (;;) {
        ret = epoll_wait(epoll_fd, events, MAX_EVENTS, -1);
		log_debug("ret is %d, errno is %d", ret, errno);
        if (ret < 0 && errno != EINTR)
            log_error("epoll_wait failed, errno is %d", errno);
        for (i = 0; i < ret; ++i) {
            // handle active hvisor_event
            hevent = events[i].data.ptr;
            if (hevent == NULL) 
                log_error("hevent shouldn't be null");
			hevent->handler(hevent->fd, hevent->epoll_type, hevent->param);
        }
    }
	pthread_exit(NULL);
	return NULL;
}

struct hvisor_event *add_event(int fd, int epoll_type,
        void (*handler)(int, int, void *), void *param)
{
    struct hvisor_event *hevent;
    struct epoll_event eevent;
    int ret;
	if (events_num >= MAX_EVENTS) {
		log_error("events are full");
		return NULL;
	}
    if (fd < 0 || handler == NULL) {
		log_error("invalid fd or handler");
        return NULL;
	}
    hevent = calloc(1, sizeof(struct hvisor_event));
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
		events[events_num] = hevent;
		events_num++;
        return hevent;
    }
}

// Create a thread monitoring events.
int initialize_event_monitor()
{
    epoll_fd = epoll_create1(0);
    log_debug("create epoll_fd is %d", epoll_fd);
    pthread_create(&emonitor_tid, NULL, epoll_loop, NULL);
    if (epoll_fd >= 0)
        return 0;
    else {
        log_error("hvisor_event init failed");
        return -1;
    }
}

void destroy_event_monitor() {
	int i;
	for (i = 0; i < events_num; i++)
		epoll_ctl(epoll_fd, EPOLL_CTL_DEL, events[i]->fd, NULL);
	close(epoll_fd);
	// When the main thread exits, the epoll thread will also exit. Therefore, we do not directly terminate the epoll thread here.
}
