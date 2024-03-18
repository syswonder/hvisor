#ifndef HVISOR_MEVENT_H
#define HVISOR_MEVENT_H
enum ev_type {
    EVF_READ,
    EVF_WRITE,
};
struct mevent {
    void		(*run)(int, enum ev_type, void *);
    void		*run_param;
    int			me_fd;
    enum ev_type me_type;
    int me_state;
};
int mevent_init(void);
struct mevent *mevent_add(int fd, enum ev_type type,
                          void (*run)(int, enum ev_type, void *), void *run_param);
#endif //HVISOR_MEVENT_H
