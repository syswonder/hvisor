#ifndef __HVISOR_VIRTIO_H
#define __HVISOR_VIRTIO_H
#include "cJSON.h"
#include "hvisor.h"
#include <linux/virtio_config.h>
#include <linux/virtio_mmio.h>
#include <linux/virtio_ring.h>
#include <stdbool.h>
#include <stdint.h>
#include <sys/uio.h>
#include <unistd.h>

#define VIRT_QUEUE_SIZE 512

typedef struct VirtMmioRegs {
  uint32_t device_id; // 设备类型id，见VirtioDeviceType
  uint32_t
      dev_feature_sel; // device_features_selection，驱动前端写该寄存器获得某一组32位的device_features
  uint32_t
      drv_feature_sel; // driver_features_selection，驱动前端写该寄存器指明某一组32位的激活的device_features
  uint32_t queue_sel; // 驱动前端写该寄存器来选择使用哪个virtqueue
  uint32_t interrupt_status;
  // interrupt_count doesn't exist in virtio specifiction,
  // only used for ensuring the correctness of interrupt_status.
  uint32_t interrupt_count;
  uint32_t status;
  uint32_t generation;
  uint64_t dev_feature; // device_features
  uint64_t drv_feature;
} VirtMmioRegs;

typedef enum {
  VirtioTNone,
  VirtioTNet,
  VirtioTBlock,
  VirtioTConsole,
  VirtioTGPU = 16
} VirtioDeviceType;

// 将VirtioDeviceType转换为const char *
const char *virtio_device_type_to_string(VirtioDeviceType type);

typedef struct vring_desc VirtqDesc;
typedef struct vring_avail VirtqAvail;
typedef struct vring_used_elem VirtqUsedElem;
typedef struct vring_used VirtqUsed;

struct VirtIODevice;
typedef struct VirtIODevice VirtIODevice;
struct VirtQueue;
typedef struct VirtQueue VirtQueue;

struct VirtQueue {
  VirtIODevice *dev;      // virtqueue所属的设备
  uint64_t vq_idx;        // virtqueue的idx
  uint64_t num;           // 由驱动前端决定的virtqueue容量大小
  uint32_t queue_num_max; // virtqueue的最大容量，由后端告知

  uint64_t desc_table_addr; // 描述符表地址(zonex设置的物理地址)
  uint64_t avail_addr;      // 可用环地址(zonex设置的物理地址)
  uint64_t used_addr;       // 已用环地址(zonex设置的物理地址)

  // 由get_virt_addr转换得到
  volatile VirtqDesc *desc_table;  // 描述符表(zone0设置的物理地址)
  volatile VirtqAvail *avail_ring; // 可用环(zone0设置的物理地址)
  volatile VirtqUsed *used_ring;   // 已用环(zone0设置的物理地址)
  int (*notify_handler)(VirtIODevice *vdev,
                        VirtQueue *vq); // 当virtqueue有可处理请求时，调用该函数

  uint16_t last_avail_idx; // 上一次处理时可用环队尾idx(队尾/最新请求idx)
  uint16_t last_used_idx; // 上一次处理时已用环队尾idx(队尾/最新响应idx)

  uint8_t ready;             // 是否就绪
  uint8_t event_idx_enabled; // VIRTIO_RING_F_EVENT_IDX特性是否启用
                             // 该特性使得设备可以在Used
                             // Ring的最后一个元素中记录当前处理的Avail
                             // Ring上的元素位置，从而告知前端驱动后端处理的进度
                             // 启用该特性会改变avail_ring的flags字段
  pthread_mutex_t used_ring_lock; // 已用环锁
};

// The highest representations of virtio device
struct VirtIODevice {
  uint32_t vqs_len;   // virtqueue的数量
  uint32_t zone_id;   // 所属的zone的id create_virtio_device中初始化
  uint32_t irq_id;    // 设备中断id create_virtio_device中初始化
  uint64_t base_addr; // the virtio device's base addr in non root zone's memory
                      // create_virtio_device中初始化
  uint64_t len;       // mmio region's length create_virtio_device中初始化
  VirtioDeviceType type; // 设备类型 create_virtio_device中初始化
  VirtMmioRegs regs;     // 设备的MMIO寄存器
                     // create_virtio_device中初始化，后继由驱动前端协商更改
  VirtQueue *vqs; // 设备的virtqueue数组 init_virtio_queue中初始化
  void *dev; // according to device type, blk is BlkDev, net is NetDev, console
             // is ConsoleDev // 指向特定设备的特殊config指针
  void (*virtio_close)(VirtIODevice *vdev); // 关闭virtio设备时所调用的函数
  bool activated;                           // 当前的virtio设备是否激活
};

// used event idx for driver telling device when to notify driver.
#define VQ_USED_EVENT(vq) ((vq)->avail_ring->ring[(vq)->num])
// avail event idx for device telling driver when to notify device.
#define VQ_AVAIL_EVENT(vq) (*(__uint16_t *)&(vq)->used_ring->ring[(vq)->num])

#define VIRT_MAGIC 0x74726976 /* 'virt' */

#define VIRT_VERSION 2

#define VIRT_VENDOR 0x48564953 /* 'HVIS' */

// 设置net和console非阻塞
int set_nonblocking(int fd);

//
int get_zone_ram_index(void *zonex_ipa, int zone_id);

/// check circular queue is full. size must be a power of 2
int is_queue_full(unsigned int front, unsigned int rear, unsigned int size);

int is_queue_empty(unsigned int front, unsigned int rear);

void write_barrier(void);

void read_barrier(void);

void rw_barrier(void);

VirtIODevice *create_virtio_device(VirtioDeviceType dev_type, uint32_t zone_id,
                                   uint64_t base_addr, uint64_t len,
                                   uint32_t irq_id, void *arg0, void *arg1);

void init_virtio_queue(VirtIODevice *vdev, VirtioDeviceType type);

void init_mmio_regs(VirtMmioRegs *regs, VirtioDeviceType type);

void virtio_dev_reset(VirtIODevice *vdev);

void virtqueue_reset(VirtQueue *vq, int idx);

bool virtqueue_is_empty(VirtQueue *vq);

// uint16_t virtqueue_pop_desc_chain_head(VirtQueue *vq);

void virtqueue_disable_notify(VirtQueue *vq);

void virtqueue_enable_notify(VirtQueue *vq);

bool desc_is_writable(volatile VirtqDesc *desc_table, uint16_t idx);

void *get_virt_addr(void *zonex_ipa, int zone_id);

void virtqueue_set_avail(VirtQueue *vq);

void virtqueue_set_used(VirtQueue *vq);

int descriptor2iov(int i, volatile VirtqDesc *vd, struct iovec *iov,
                   uint16_t *flags, int zone_id, bool copy_flags);

int process_descriptor_chain(VirtQueue *vq, uint16_t *desc_idx,
                             struct iovec **iov, uint16_t **flags,
                             int append_len, bool copy_flags);

void update_used_ring(VirtQueue *vq, uint16_t idx, uint32_t iolen);

uint64_t virtio_mmio_read(VirtIODevice *vdev, uint64_t offset, unsigned size);

void virtio_mmio_write(VirtIODevice *vdev, uint64_t offset, uint64_t value,
                       unsigned size);

bool in_range(uint64_t value, uint64_t lower, uint64_t len);

void virtio_inject_irq(VirtQueue *vq); // unused

void virtio_finish_cfg_req(uint32_t target_cpu, uint64_t value);

int virtio_handle_req(volatile struct device_req *req);

void virtio_close();

void handle_virtio_requests();

void initialize_log();

int virtio_init();

int create_virtio_device_from_json(cJSON *device_json, int zone_id);

int virtio_start_from_json(char *json_path);

int virtio_start(int argc, char *argv[]);

void *read_file(char *filename, u_int64_t *filesize);

#endif /* __HVISOR_VIRTIO_H */
