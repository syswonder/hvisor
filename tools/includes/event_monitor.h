#ifndef HVISOR_EVENT_H
#define HVISOR_EVENT_H
#include <sys/epoll.h>

struct hvisor_event {
    void		(*handler)(int, int, void *);
    void		*param;
    int			fd;
    int 		epoll_type;
};

int initialize_event_monitor(void);
struct hvisor_event *add_event(int fd, int epoll_type,
                          void (*handler)(int, int, void *), void *param);
#endif //HVISOR_EVENT_H
