#include "log.h"
#include "sys/queue.h"
#include "unistd.h"
#include "virtio.h"
#include "virtio_gpu.h"
#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <xf86drm.h>
#include <xf86drmMode.h>

GPUDev *init_gpu_dev(GPURequestedState *requested_state) {
  log_info("initializing GPUDev");

  if (requested_state == NULL) {
    log_error("null requested state");
    return NULL;
  }

  // 分配内存
  GPUDev *gdev = malloc(sizeof(GPUDev));

  if (gdev == NULL) {
    log_error("failed to initialize GPUDev");
    // 让调用函数来释放内存
    // free(requested_state);
    return NULL;
  }

  memset(gdev, 0, sizeof(GPUDev));

  // 初始化config
  gdev->config.events_read = 0;
  gdev->config.events_clear = 0;
  gdev->config.num_scanouts = HVISOR_VIRTIO_GPU_MAX_SCANOUTS;
  gdev->config.num_capsets = 0;

  // 初始化scanouts数量
  gdev->scanouts_num = 1;

  // 初始化scanout 0
  // TODO(root): 使用json初始化，或者读取设备
  gdev->scanouts[0].width = SCANOUT_DEFAULT_WIDTH;
  gdev->scanouts[0].height = SCANOUT_DEFAULT_HEIGHT;

  log_debug("set scanouts[0] width: %d height: %d", gdev->scanouts[0].width,
            gdev->scanouts[0].height);

  // dev->scanouts[0].x = 0;
  // dev->scanouts[0].y = 0;
  // dev->scanouts[0].resource_id = 0;
  gdev->scanouts[0].current_cursor = NULL;
  gdev->scanouts[0].card0_fd = -1;
  gdev->enabled_scanout_bitmask |= (1 << 0); // 启用scanout 0

  // scanout的framebuffer由驱动前端设置，见virtio_gpu_set_scanout

  // TODO(root): 多组requested_states(需要更改json解析)
  gdev->requested_states[0].width = requested_state->width;
  gdev->requested_states[0].height = requested_state->height;

  log_debug("requested state from json, width: %d height: %d",
            gdev->requested_states[0].width, gdev->requested_states[0].height);

  // 初始化资源列表和命令队列
  TAILQ_INIT(&gdev->resource_list);
  TAILQ_INIT(&gdev->command_queue);

  // 初始化内存计数
  gdev->hostmem = 0;

  // 初始化async部分
  gdev->close = false;

  return gdev;
}

int virtio_gpu_init(VirtIODevice *vdev) {
  log_info("entering %s", __func__);

  // TODO(root): 显示设备初始化
  GPUDev *gdev = vdev->dev;

  // 设置virtio gpu的关闭函数
  vdev->virtio_close = virtio_gpu_close;

  int drm_fd = 0;

  // 打开card0
  drm_fd = open("/dev/dri/card0", O_RDWR | O_CLOEXEC);
  if (drm_fd < 0) {
    log_error("%s failed to open /dev/dri/card0", __func__);
    return -1;
  }

  // 获取drm资源
  // 需要获得设备的连接器(connector)、显示控制器(CRTC)、encoder、framebuffer
  drmModeRes *res = drmModeGetResources(drm_fd);
  if (!res) {
    log_error("%s cannot get card0 resource", __func__);
    close(drm_fd);
    return -1;
  }

  // 获取connector
  drmModeConnector *connector = NULL;
  for (int i = 0; i < res->count_connectors; ++i) {
    connector = drmModeGetConnector(drm_fd, res->connectors[i]);
    if (connector->connection == DRM_MODE_CONNECTED) {
      break;
    }
    drmModeFreeConnector(connector);
  }

  if (!connector || connector->connection != DRM_MODE_CONNECTED) {
    log_error("%s cannot find a connector", __func__);
    drmModeFreeResources(res);
    close(drm_fd);
    return -1;
  }

  // 获取encoder
  drmModeEncoder *encoder = drmModeGetEncoder(drm_fd, connector->encoder_id);
  if (!encoder) {
    log_error("%s cannot get encoder", __func__);
    drmModeFreeConnector(connector);
    drmModeFreeResources(res);
    close(drm_fd);
    return -1;
  }

  // 获取CRTC
  drmModeCrtc *crtc = drmModeGetCrtc(drm_fd, encoder->crtc_id);
  if (!crtc) {
    log_error("%s cannot get CRTC", __func__);
    drmModeFreeEncoder(encoder);
    drmModeFreeConnector(connector);
    drmModeFreeResources(res);
    close(drm_fd);
    return -1;
  }
  // drmModeModeInfo mode = connector->modes[0];

  gdev->scanouts[0].card0_fd = drm_fd;
  gdev->scanouts[0].crtc = crtc;
  gdev->scanouts[0].connector = connector;
  gdev->scanouts[0].encoder = encoder;

  ///
  log_debug("%s set scanout[0] card0_fd %d", __func__, drm_fd);
  log_debug("%s set scanout[0] crtc %x with id %d", __func__, crtc,
            crtc->crtc_id);
  log_debug("%s set scanout[0] connector %x with id %d", __func__, connector,
            connector->connector_id);
  log_debug("%s get scanout[0] connector mode hdisplay: %d, vdisplay: %d",
            __func__, connector->modes[0].hdisplay,
            connector->modes[0].vdisplay);
  log_debug("%s set scanout[0] encoder %x", __func__, encoder);
  ///

  gdev->scanouts[0].width = connector->modes[0].hdisplay;
  gdev->scanouts[0].height = connector->modes[0].vdisplay;

  // async
  pthread_create(&gdev->gpu_thread, NULL, virtio_gpu_handler, vdev);
  pthread_cond_init(&gdev->gpu_cond, NULL);
  pthread_mutex_init(&gdev->queue_mutex, NULL);

  return 0;
}

void virtio_gpu_close(VirtIODevice *vdev) {
  log_info("virtio_gpu close");

  // 回收scanouts相关内存
  GPUDev *gdev = (GPUDev *)vdev->dev;
  for (int i = 0; i < gdev->scanouts_num; ++i) {
    free(gdev->scanouts[i].current_cursor);

    virtio_gpu_remove_drm_framebuffer(&gdev->scanouts[i]);

    drmModeFreeCrtc(gdev->scanouts[i].crtc);
    drmModeFreeEncoder(gdev->scanouts[i].encoder);
    drmModeFreeConnector(gdev->scanouts[i].connector);

    // 释放card0_fd
    if (gdev->scanouts[i].card0_fd != -1) {
      close(gdev->scanouts[i].card0_fd);
    }
  }

  // 回收resource相关内存
  while (!TAILQ_EMPTY(&gdev->resource_list)) {
    GPUSimpleResource *temp = TAILQ_FIRST(&gdev->resource_list);
    TAILQ_REMOVE(&gdev->resource_list, temp, next);
    free(temp);
  }

  // 回收命令队列内存
  while (!TAILQ_EMPTY(&gdev->command_queue)) {
    GPUCommand *temp = TAILQ_FIRST(&gdev->command_queue);
    TAILQ_REMOVE(&gdev->command_queue, temp, next);
    free(temp);
  }

  // 回收async部分
  gdev->close = true;
  pthread_cond_signal(&gdev->gpu_cond);
  pthread_join(gdev->gpu_thread, NULL);
  pthread_cond_destroy(&gdev->gpu_cond);
  pthread_mutex_destroy(&gdev->queue_mutex);

  free(gdev);
  gdev = NULL;

  // vq由驱动前端管理，这里直接释放
  free(vdev->vqs);
  free(vdev);
}

void virtio_gpu_reset(GPUDev *gdev) {
  // TODO(root):
  for (int i = 0; i < HVISOR_VIRTIO_GPU_MAX_SCANOUTS; ++i) {
    gdev->scanouts[i].resource_id = 0;
    gdev->scanouts[i].width = 0;
    gdev->scanouts[i].height = 0;
    gdev->scanouts[i].x = 0;
    gdev->scanouts[i].y = 0;
  }
}

int virtio_gpu_ctrl_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
  log_debug("entering %s", __func__);

  GPUDev *gdev = vdev->dev;
  uint32_t cnt = 0;

  virtqueue_disable_notify(vq);
  while (!virtqueue_is_empty(vq)) {
    int err = virtio_gpu_handle_single_request(vdev, vq, GPU_CONTROL_QUEUE);
    if (err < 0) {
      log_error("notify handle failed at zone %d, device %s", vdev->zone_id,
                virtio_device_type_to_string(vdev->type));
      // return -1;
    }
    cnt++;
  }
  log_debug("%s add %d request to command queue", __func__, cnt);

  // kick处理线程
  pthread_mutex_lock(&gdev->queue_mutex);
  pthread_cond_signal(&gdev->gpu_cond);
  pthread_mutex_unlock(&gdev->queue_mutex);

  virtqueue_enable_notify(vq);

  return 0;
}

int virtio_gpu_cursor_notify_handler(VirtIODevice *vdev, VirtQueue *vq) {
  log_debug("entering %s", __func__);

  virtqueue_disable_notify(vq);
  while (!virtqueue_is_empty(vq)) {
    int err = virtio_gpu_handle_single_request(vdev, vq, GPU_CURSOR_QUEUE);
    if (err < 0) {
      log_error("notify handle failed at zone %d, device %s", vdev->zone_id,
                virtio_device_type_to_string(vdev->type));
      // return -1;
    }
  }
  virtqueue_enable_notify(vq);

  virtio_inject_irq(vq);

  return 0;
}

int virtio_gpu_handle_single_request(VirtIODevice *vdev, VirtQueue *vq,
                                     uint32_t from) {
  // virtio-gpu dev
  GPUDev *gdev = vdev->dev;
  // 描述符链的起始idx
  uint16_t first_idx_on_chain = 0;
  // 通信使用的iovec
  struct iovec *iov = NULL;
  // 描述符链上的所有描述符的flags
  uint16_t *flags = NULL;
  // 处理的描述符数
  // 描述符数等于缓冲区数，也就等于iov数组的长度
  int desc_processed_num = 0;
  // 生成的命令
  GPUCommand *gcmd = NULL;

  // 根据描述符链，将通信的所有buffer集中到iov进行管理
  desc_processed_num =
      process_descriptor_chain(vq, &first_idx_on_chain, &iov, &flags, 0, true);
  if (desc_processed_num < 1) {
    log_debug("no more desc at %s", __func__);
    return 0;
  }

  gcmd = malloc(sizeof(GPUCommand));
  memset(gcmd, 0, sizeof(GPUCommand));
  gcmd->resp_iov = iov;
  gcmd->resp_iov_cnt = desc_processed_num;
  gcmd->resp_idx = first_idx_on_chain;
  gcmd->from_queue = from;

  // 加入命令队列
  pthread_mutex_lock(&gdev->queue_mutex);
  TAILQ_INSERT_TAIL(&gdev->command_queue, gcmd, next);
  pthread_mutex_unlock(&gdev->queue_mutex);

  free(flags);
  return 0;
}

size_t iov_to_buf_full(const struct iovec *iov, unsigned int iov_cnt,
                       size_t offset, void *buf, size_t bytes_need_copy) {
  size_t done = 0;
  unsigned int i = 0;
  for (; (offset || done < bytes_need_copy) && i < iov_cnt; i++) {
    if (offset < iov[i].iov_len) {
      size_t len = MIN(iov[i].iov_len - offset, bytes_need_copy - done);
      memcpy(buf + done, iov[i].iov_base + offset, len);
      done += len;
      offset = 0;
    } else {
      offset -= iov[i].iov_len;
    }
  }
  if (offset != 0) {
    log_error("failed to copy iov to buf");
    return 0;
  }

  return done;
}

size_t buf_to_iov_full(const struct iovec *iov, unsigned int iov_cnt,
                       size_t offset, const void *buf, size_t bytes_need_copy) {
  size_t done = 0;
  unsigned int i = 0;
  for (; (offset || done < bytes_need_copy) && i < iov_cnt; i++) {
    if (offset < iov[i].iov_len) {
      size_t len = MIN(iov[i].iov_len - offset, bytes_need_copy - done);
      memcpy(iov[i].iov_base + offset, buf + done, len);
      done += len;
      offset = 0;
    } else {
      offset -= iov[i].iov_len;
    }
  }
  if (offset != 0) {
    log_error("failed to copy buf to iov");
    return 0;
  }

  return done;
}