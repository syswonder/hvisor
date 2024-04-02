#include "tools.h"
#include "log.h"
inline int is_queue_full(unsigned int front, unsigned int rear, unsigned int size)
{
    if (((rear + 1) & (size - 1)) == front) {
        log_trace("queue is full, front is %u, rear is %u", front, rear);
        return 1;
    } else {
        return 0;
    }
}

inline int is_queue_empty(unsigned int front, unsigned int rear)
{
    return rear == front;
}