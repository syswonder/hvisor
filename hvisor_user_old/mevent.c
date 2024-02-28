#include "mevent.h"
#include "log.h"
#include <sys/epoll.h>
#include <pthread.h>
#include <errno.h>
#include <stdlib.h>
static int epoll_fd;
#define	MEVENT_MAX	64

static void *mevent_loop(void *param)
{
    struct epoll_event eventlist[MEVENT_MAX];
    int ret;
    for (;;) {
        ret = epoll_wait(epoll_fd, eventlist, MEVENT_MAX, -1);
        if (ret == -1 && errno != EINTR)
            log_error("Error return from epoll_wait");
        struct mevent *mevp;
        for (int i = 0; i < ret; ++i) {
            // handle active mevent
            mevp = eventlist[i].data.ptr;
            (mevp->run)(mevp->me_fd, mevp->me_type, mevp->run_param);
        }
    }
}

// transform me_type to epoll type
static int
mevent_get_epoll_event(struct mevent *mevp)
{
    int retval;
    retval = 0;
    if (mevp->me_type == EVF_READ)
        retval = EPOLLIN;
    if (mevp->me_type == EVF_WRITE)
        retval = EPOLLOUT;
    return retval;
}

struct mevent *mevent_add(int fd, enum ev_type type,
        void (*run)(int, enum ev_type, void *), void *run_param)
{
    struct mevent *mevp;
    struct epoll_event ee;
    int ret;
    mevp = calloc(1, sizeof(struct mevent));
    if (mevp == NULL)
        return NULL;
    mevp->me_fd = fd;
    mevp->me_type = type;
    mevp->run = run;
    mevp->run_param = run_param;
    ee.events = mevent_get_epoll_event(mevp);
    ee.data.ptr = mevp;
    ret = epoll_ctl(epoll_fd, EPOLL_CTL_ADD, mevp->me_fd, &ee);
    if (ret)
        return mevp;
    else {
        free(mevp);
        return NULL;
    }
}

int mevent_init()
{
    epoll_fd = epoll_create1(0);
    pthread_t mevent_tid;
    pthread_create(&mevent_tid, NULL, mevent_loop, NULL);
    if (epoll_fd >= 0)
        return 0;
    else {
        log_error("mevent init failed");
        return -1;
    }
}

