#ifndef HVISOR_TOOLS_H
#define HVISOR_TOOLS_H

/// check circular queue is full. size must be a power of 2
int is_queue_full(unsigned int front, unsigned int rear, unsigned int size);
int is_queue_empty(unsigned int front, unsigned int rear);
#endif //HVISOR_TOOLS_H
