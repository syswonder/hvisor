#include "log.h"
#include "sys/queue.h"
#include "virtio.h"
#include "virtio_gpu.h"
#include <pthread.h>
#include <stdlib.h>

void *virtio_gpu_handler(void *dev) {
  VirtIODevice *vdev = (VirtIODevice *)dev;
  GPUDev *gdev = vdev->dev;
  GPUCommand *gcmd = NULL;

  uint32_t request_cnt = 0;

  pthread_mutex_lock(&gdev->queue_mutex);
  for (;;) {
    // 检查设备是否关闭
    if (gdev->close) {
      pthread_mutex_unlock(&gdev->queue_mutex);
      pthread_exit(NULL);
      return NULL;
    }

    // 试图获取一个命令
    // 此时持有锁
    while (!TAILQ_EMPTY(&gdev->command_queue)) {
      gcmd = TAILQ_FIRST(&gdev->command_queue);
      TAILQ_REMOVE(&gdev->command_queue, gcmd, next);

      pthread_mutex_unlock(&gdev->queue_mutex);

      // 释放锁并开始处理
      virtio_gpu_simple_process_cmd(gcmd, vdev);
      // 命令完成后通知前端并释放内存
      // iov由virtio_gpu_simple_process_cmd释放

      request_cnt++;

      if (request_cnt >= VIRTIO_GPU_MAX_REQUEST_BEFORE_KICK) {
        // 已经处理了一定数量的请求，kick前端
        virtio_inject_irq(&vdev->vqs[gcmd->from_queue]);
        request_cnt = 0;
        // log_info("%s: processed request >= 16, kick frontend", __func__);
      }

      free(gcmd);

      // 由于我们仍在处理循环中，此时不用担心lose awake
      // 下一次检查时重新获得锁
      pthread_mutex_lock(&gdev->queue_mutex);
    }

    if (request_cnt != 0) {
      // 已经处理了请求但是任务队列为空，立刻kick前端
      virtio_inject_irq(&vdev->vqs[gcmd->from_queue]);
      request_cnt = 0;
      // log_info("%s: request queue empty, kick frontend", __func__);
    }

    pthread_cond_wait(&gdev->gpu_cond, &gdev->queue_mutex);
  }
}