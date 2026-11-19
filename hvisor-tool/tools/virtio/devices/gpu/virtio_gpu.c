// SPDX-License-Identifier: GPL-2.0-only
/**
 * Copyright (c) 2025 Syswonder
 *
 * Syswonder Website:
 *      https://www.syswonder.org
 *
 * Authors:
 *      
 */
#include "virtio_gpu.h"
#include "linux/stddef.h"
#include "linux/types.h"
#include "linux/virtio_gpu.h"
#include "log.h"
#include "sys/queue.h"
#include "virtio.h"
#include <drm/drm.h>
#include <drm/drm_fourcc.h>
#include <drm/drm_mode.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <time.h>
#include <xf86drm.h>
#include <xf86drmMode.h>

void virtio_gpu_ctrl_response(VirtIODevice *vdev, GPUCommand *gcmd,
                              GPUControlHeader *resp, size_t resp_len) {
    log_debug("sending response");
    // Since the header of each response structure is GPUControlHeader, its
    // address is the address of the response structure

    // The first descriptor corresponding to iov[0] is always read-only, so
    // start from the second one
    size_t s = buf_to_iov(&gcmd->resp_iov[1], gcmd->resp_iov_cnt - 1, 0, resp,
                          resp_len);

    if (s != resp_len) {
        log_error("%s cannot copy buffer to iov with correct size", __func__);
        // Continue to return, let the front end handle it
    }

    update_used_ring(&vdev->vqs[gcmd->from_queue], gcmd->resp_idx, resp_len);

    gcmd->finished = true;
}

void virtio_gpu_ctrl_response_nodata(VirtIODevice *vdev, GPUCommand *gcmd,
                                     enum virtio_gpu_ctrl_type type) {
    log_debug("entering %s", __func__);

    GPUControlHeader resp;

    memset(&resp, 0, sizeof(resp));
    resp.type = type;
    virtio_gpu_ctrl_response(vdev, gcmd, &resp, sizeof(resp));
}

void virtio_gpu_get_display_info(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    struct virtio_gpu_resp_display_info display_info;
    GPUDev *gdev = vdev->dev;

    memset(&display_info, 0, sizeof(display_info));
    display_info.hdr.type = VIRTIO_GPU_RESP_OK_DISPLAY_INFO;

    // Fill the response structure with display information
    for (int i = 0; i < HVISOR_VIRTIO_GPU_MAX_SCANOUTS; ++i) {
        if (gdev->enabled_scanout_bitmask & (1 << i)) {
            display_info.pmodes[i].enabled = 1;
            display_info.pmodes[i].r.width = gdev->requested_states[i].width;
            display_info.pmodes[i].r.height = gdev->requested_states[i].height;
            log_debug(
                "return display info of scanout %d with width %d, height %d", i,
                display_info.pmodes[i].r.width,
                display_info.pmodes[i].r.height);
        }
    }

    virtio_gpu_ctrl_response(vdev, gcmd, &display_info.hdr,
                             sizeof(display_info));
}

void virtio_gpu_get_edid(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);
    // TODO: Implement this function
}

void virtio_gpu_resource_create_2d(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUSimpleResource *res = NULL;
    GPUDev *gdev = vdev->dev;
    struct virtio_gpu_resource_create_2d create_2d;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, create_2d);

    // Check if the resource_id to be created is 0 (cannot use 0)
    if (create_2d.resource_id == 0) {
        log_error("%s trying to create 2d resource with id 0", __func__);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID;
        return;
    }

    // Check if the resource has already been created
    res = virtio_gpu_find_resource(gdev, create_2d.resource_id);
    if (res) {
        log_error("%s trying to create an existing resource with id %d",
                  __func__, create_2d.resource_id);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID;
        return;
    }

    // Otherwise, create a new resource
    res = calloc(1, sizeof(GPUSimpleResource));
    memset(res, 0, sizeof(GPUSimpleResource));

    res->width = create_2d.width;
    res->height = create_2d.height;
    res->format = create_2d.format;
    res->resource_id = create_2d.resource_id;
    res->scanout_bitmask = 0;
    res->iov = NULL;
    res->iov_cnt = 0;

    // Calculate the memory size occupied by the resource
    // By default, only formats with 4 bytes per pixel are supported
    res->hostmem = calc_image_hostmem(32, create_2d.width, create_2d.height);
    if (res->hostmem + gdev->hostmem >= VIRTIO_GPU_MAX_HOSTMEM) {
        log_error("virtio gpu for zone %d out of hostmem when trying to create "
                  "resource %d",
                  vdev->zone_id, create_2d.resource_id);
        free(res);
        return;
    }

    // If memory is sufficient, add the resource to the virtio gpu management
    TAILQ_INSERT_HEAD(&gdev->resource_list, res, next);
    gdev->hostmem += res->hostmem;

    log_debug("add a resource %d to gpu dev of zone %d, width: %d height: %d "
              "format: %d mem: %d bytes host-hostmem: %d bytes",
              res->resource_id, vdev->zone_id, res->width, res->height,
              res->format, res->hostmem, gdev->hostmem);
}

GPUSimpleResource *virtio_gpu_find_resource(GPUDev *gdev,
                                            uint32_t resource_id) {
    GPUSimpleResource *temp_res = NULL;
    TAILQ_FOREACH(temp_res, &gdev->resource_list, next) {
        if (temp_res->resource_id == resource_id) {
            return temp_res;
        }
    }
    return NULL;
}

GPUSimpleResource *virtio_gpu_check_resource(VirtIODevice *vdev,
                                             uint32_t resource_id,
                                             const char *caller,
                                             uint32_t *error) {
    GPUSimpleResource *res = NULL;
    GPUDev *gdev = vdev->dev;

    res = virtio_gpu_find_resource(gdev, resource_id);
    if (!res) {
        log_error("%s cannot find resource by id %d", caller, resource_id);
        if (error) {
            *error = VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID;
        }
        return NULL;
    }

    if (!res->iov || res->hostmem <= 0) {
        log_error("%s found resource %d has no backing storage", caller,
                  resource_id);
        if (error) {
            *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        }
        return NULL;
    }

    return res;
}

uint32_t calc_image_hostmem(int bits_per_pixel, uint32_t width,
                            uint32_t height) {
    // 0x1f = 31, >> 5 is equivalent to dividing by 32, aligning the bits per
    // row to a multiple of 4 bytes
    // Finally, multiply by sizeof(uint32) to get the number of bytes
    int stride = ((width * bits_per_pixel + 0x1f) >> 5) * sizeof(uint32_t);
    // Return the total memory size
    return height * stride;
}

void virtio_gpu_resource_unref(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUDev *gdev = vdev->dev;

    GPUSimpleResource *res = NULL;
    struct virtio_gpu_resource_unref unref;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, unref);

    res = virtio_gpu_find_resource(gdev, unref.resource_id);
    if (!res) {
        log_error("%s cannot find resource %d", __func__, unref.resource_id);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID;
        return;
    }

    virtio_gpu_resource_destory(gdev, res);
}

void virtio_gpu_resource_destory(GPUDev *gdev, GPUSimpleResource *res) {
    if (res->scanout_bitmask) {
        for (int i = 0; i < HVISOR_VIRTIO_GPU_MAX_SCANOUTS; ++i) {
            if (res->scanout_bitmask & (1 << i)) {
                virtio_gpu_disable_scanout(gdev, i);
            }
        }
    }

    virtio_gpu_cleanup_mapping(gdev, res);
    TAILQ_REMOVE(&gdev->resource_list, res, next);
    gdev->hostmem -= res->hostmem;
    free(res);
}

void virtio_gpu_disable_scanout(GPUDev *gdev, int scanout_id) {
    GPUScanout *scanout = &gdev->scanouts[scanout_id];
    GPUSimpleResource *res = NULL;

    if (scanout->resource_id == 0) {
        return;
    }

    res = virtio_gpu_find_resource(gdev, scanout->resource_id);
    if (res) {
        res->scanout_bitmask &= ~(1 << scanout_id);
    }

    scanout->resource_id = 0;
    scanout->width = 0;
    scanout->height = 0;
}

void virtio_gpu_cleanup_mapping(GPUDev *gdev, GPUSimpleResource *res) {
    if (res->iov) {
        free(res->iov);
        // The memory block corresponding to iov is handled by the guest
    }

    res->iov = NULL;
    res->iov_cnt = 0;
}

void virtio_gpu_resource_flush(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUDev *gdev = vdev->dev;

    GPUSimpleResource *res = NULL;
    GPUScanout *scanout = NULL;
    struct virtio_gpu_resource_flush resource_flush;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, resource_flush);

    res = virtio_gpu_check_resource(vdev, resource_flush.resource_id, __func__,
                                    &gcmd->error);
    if (!res) {
        return;
    }

    if (resource_flush.r.x > res->width || resource_flush.r.y > res->height ||
        resource_flush.r.width > res->width ||
        resource_flush.r.height > res->height ||
        resource_flush.r.x + resource_flush.r.width > res->width ||
        resource_flush.r.y + resource_flush.r.height > res->height) {
        log_error(
            "%s flush bounds outside resource %d bounds, (%d, %d) + %d, %d "
            "vs %d, %d",
            __func__, resource_flush.resource_id, resource_flush.r.x,
            resource_flush.r.y, resource_flush.r.width, resource_flush.r.height,
            res->width, res->height);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER;
        return;
    }

    for (int i = 0; i < HVISOR_VIRTIO_GPU_MAX_SCANOUTS; ++i) {
        // Traverse the scanouts corresponding to the resource
        if (!(res->scanout_bitmask & (1 << i))) {
            continue;
        }
        scanout = &gdev->scanouts[i];

        if (!scanout->frame_buffer.enabled) {
            virtio_gpu_create_drm_framebuffer(scanout, &gcmd->error);
            if (gcmd->error) {
                return;
            }

            virtio_gpu_copy_and_flush(scanout, res, &gcmd->error);
        } else {
            // TODO: If the framebuffer has already been allocated once and the
            // size is not suitable, it needs to be reallocated
            // TODO: This includes calling munmap and the drm destroy function
            // TODO: Encapsulate the case where it has not been allocated

            virtio_gpu_copy_and_flush(scanout, res, &gcmd->error);
        }
    }
}

void virtio_gpu_create_drm_framebuffer(GPUScanout *scanout, uint32_t *error) {
    if (scanout->frame_buffer.enabled) {
        return;
    }

    // If the scanout's frame_buffer has not been initialized yet
    GPUFrameBuffer *fb = &scanout->frame_buffer;

    struct drm_mode_create_dumb dumb = {0};
    struct drm_mode_map_dumb map = {0};
    dumb.width = fb->width;
    dumb.height = fb->height;
    dumb.bpp = fb->bytes_pp * 8;
    uint32_t offsets[4] = {0};
    uint32_t fb_id = 0;

    if (drmIoctl(scanout->card0_fd, DRM_IOCTL_MODE_CREATE_DUMB, &dumb) < 0) {
        log_error("%s failed to create a drm dumb", __func__);
        *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    } // Create a dumb object

    map.handle = dumb.handle;
    // Bind the video memory to the framebuffer, get the offset based on the
    // handle
    if (drmIoctl(scanout->card0_fd, DRM_IOCTL_MODE_MAP_DUMB, &map) < 0) {
        log_error("%s failed to map a drm dumb", __func__);
        virtio_gpu_remove_drm_framebuffer(scanout);
        *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }

    // ! legacy
    // uint32_t drm_format = VIRTIO_GPU_FORMAT_TO_DRM_FORMAT(fb->format);
    // if (!drm_format) {
    //   log_error("%s found drm_framebuffer has an unknown format", __func__);
    //   *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
    //   return;
    // }

    // ! legacy
    // if (drmModeAddFB2(scanout->card0_fd, dumb.width, dumb.height, drm_format,
    //                   &dumb.handle, &dumb.pitch, offsets, &fb_id, 0) < 0) {
    //   log_error("%s failed to add a drm_framebuffer to card0", __func__);
    //   *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
    //   return;
    // }

    if (drmModeAddFB(scanout->card0_fd, dumb.width, dumb.height, 24, 32,
                     dumb.pitch, dumb.handle, &fb_id) < 0) {
        log_error("%s failed to add a drm_framebuffer to card0", __func__);
        *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }

    ///
    log_debug("%s create a drm_framebuffer with width: %d, height: %d, "
              "format: %d, handle: %d, fb_id: %d",
              __func__, dumb.width, dumb.height, fb->format, dumb.handle,
              fb_id);
    ///

    void *vaddr = mmap(0, dumb.size, PROT_READ | PROT_WRITE, MAP_SHARED,
                       scanout->card0_fd, map.offset);

    if (!vaddr) {
        log_error("%s cannot map drm_framebuffer of scanout", __func__);
        virtio_gpu_remove_drm_framebuffer(scanout);
        *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }

    log_debug("%s map drm_framebuffer to %x with size %d", __func__, vaddr,
              dumb.size);

    fb->fb_id = fb_id;
    fb->drm_dumb_handle = dumb.handle;
    fb->drm_dumb_size = dumb.size;
    fb->fb_addr = vaddr;
    fb->enabled = true;
}

void virtio_gpu_copy_and_flush(GPUScanout *scanout, GPUSimpleResource *res,
                               uint32_t *error) {

    uint32_t format = 0;
    uint32_t stride = 0;
    uint32_t src_offset = 0;
    uint32_t dst_offset = 0;
    int bpp = 0;

    GPUFrameBuffer *fb = &scanout->frame_buffer;

    if (!res || !res->iov || res->hostmem <= 0) {
        log_error("%s found res is not create yet");
        *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }

    if (!fb || !fb->enabled || !fb->fb_addr) {
        log_error("%s found drm_framebuffer is not enabled yet");
        *error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }

    format = res->format;
    bpp = 32;
    stride = res->hostmem / res->height;

    // Copy data from resource to framebuffer
    if (res->transfer_rect.x || res->transfer_rect.width != res->width) {
        for (int h = 0; h < res->transfer_rect.height; h++) {
            src_offset = res->transfer_offset + stride * h;
            dst_offset = (res->transfer_rect.y + h) * stride +
                         (res->transfer_rect.x * bpp);

            size_t s = iov_to_buf(res->iov, res->iov_cnt, src_offset,
                                  fb->fb_addr + dst_offset,
                                  res->transfer_rect.width * bpp);

            // debug
            log_debug("%s copy %d bytes from resource %d to drm_framebuffer, "
                      "src_offset: %d, dst_offset: %d",
                      __func__, s, res->resource_id, src_offset, dst_offset);
        }
    } else {
        src_offset = res->transfer_offset;
        dst_offset = res->transfer_rect.y * stride + res->transfer_rect.x * bpp;

        size_t s = iov_to_buf(res->iov, res->iov_cnt, src_offset,
                              fb->fb_addr + dst_offset,
                              stride * res->transfer_rect.height);

        // debug
        log_debug("%s copy %d bytes from resource %d to drm_framebuffer, "
                  "src_offset: %d, dst_offset: %d",
                  __func__, s, res->resource_id, src_offset, dst_offset);
    }

    drmModeModeInfo mode = scanout->connector->modes[0];
    drmModeSetCrtc(scanout->card0_fd, scanout->crtc->crtc_id, fb->fb_id, 0, 0,
                   &scanout->connector->connector_id, 1, &mode);

    log_debug("%s flush with card0_fd: %d, crtc_id: %d, fb_id: %d, "
              "connector_id: %d",
              __func__, scanout->card0_fd, scanout->crtc->crtc_id, fb->fb_id,
              scanout->connector->connector_id);
}

void virtio_gpu_remove_drm_framebuffer(GPUScanout *scanout) {
    GPUFrameBuffer *fb = &scanout->frame_buffer;

    if (!fb || !fb->enabled || !fb->fb_addr) {
        log_error("%s found drm_framebuffer is not enabled yet");
        return;
    }

    struct drm_mode_destroy_dumb destory = {0};
    destory.handle = fb->drm_dumb_handle;

    drmModeRmFB(scanout->card0_fd, fb->fb_id);
    if (fb->fb_addr != NULL) {
        munmap(fb->fb_addr, fb->drm_dumb_size);
    }
    drmIoctl(scanout->card0_fd, DRM_IOCTL_MODE_DESTROY_DUMB, &destory);

    log_debug("%s destoryed drm_framebuffer with id: %d, handle: %d, size: %d",
              __func__, fb->fb_id, fb->drm_dumb_handle, fb->drm_dumb_size);

    // Reserve others
    fb->fb_id = 0;
    fb->drm_dumb_handle = 0;
    fb->drm_dumb_size = 0;
    fb->fb_addr = NULL;
    fb->enabled = false;
}

void virtio_gpu_set_scanout(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUDev *gdev = vdev->dev;

    GPUSimpleResource *res = NULL;
    GPUFrameBuffer fb = {0};
    struct virtio_gpu_set_scanout set_scanout;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, set_scanout);

    // Check scanout id
    if (set_scanout.scanout_id >= gdev->scanouts_num) {
        log_error("%s setting invalid scanout with scanout_id %d", __func__,
                  set_scanout.scanout_id);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_SCANOUT_ID;
        return;
    }

    res = virtio_gpu_check_resource(vdev, set_scanout.resource_id, __func__,
                                    &gcmd->error);
    if (!res) {
        return;
    }

    log_debug("%s setting scanout %d with resource %d", __func__,
              set_scanout.scanout_id, set_scanout.resource_id);

    fb.format = res->format;
    fb.bytes_pp = 4; // All formats are 4 bytes pp (32 bytes per pixel)
    fb.width = res->width;
    fb.height = res->height;
    fb.stride = res->hostmem / res->height; // hostmem = height * stride
    fb.offset = set_scanout.r.x * fb.bytes_pp + set_scanout.r.y * fb.stride;
    fb.fb_addr = NULL;
    fb.enabled = false;

    virtio_gpu_do_set_scanout(vdev, set_scanout.scanout_id, &fb, res,
                              &set_scanout.r, &gcmd->error);
}

bool virtio_gpu_do_set_scanout(VirtIODevice *vdev, uint32_t scanout_id,
                               GPUFrameBuffer *fb, GPUSimpleResource *res,
                               struct virtio_gpu_rect *r, uint32_t *error) {
    GPUDev *gdev = vdev->dev;
    GPUScanout *scanout = NULL;

    scanout = &gdev->scanouts[scanout_id];

    if (r->x > fb->width || r->y > fb->width || r->width < 16 ||
        r->height < 16 || r->width > fb->width || r->height > fb->height ||
        r->x + r->width > fb->width || r->y + r->height > fb->height) {
        log_error(
            "%s found illegal scanout %d bounds for resource %d, rect (%d, "
            "%d) + %d, %d, fb %d, %d",
            __func__, scanout_id, res->resource_id, r->x, r->y, r->width,
            r->height, fb->width, fb->height);
        *error = VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER;
        return false;
    }

    log_debug(
        "%s set scanout display region (%d, %d) + %d, %d, with framebuffer "
        "%d, %d",
        __func__, r->x, r->y, r->width, r->height, fb->width, fb->height);

    // TODO: Blob related logic (if VIRTIO_GPU_F_RESOURCE_BLOB feature is
    // supported)

    // Update scanout
    virtio_gpu_update_scanout(vdev, scanout_id, fb, res, r);
    return true;
}

void virtio_gpu_update_scanout(VirtIODevice *vdev, uint32_t scanout_id,
                               GPUFrameBuffer *fb, GPUSimpleResource *res,
                               struct virtio_gpu_rect *r) {
    GPUDev *gdev = vdev->dev;
    GPUSimpleResource *origin_res = NULL;
    GPUScanout *scanout = NULL;

    scanout = &gdev->scanouts[scanout_id];
    origin_res = virtio_gpu_find_resource(gdev, scanout->resource_id);
    if (origin_res) {
        // Unbind the original resource
        origin_res->scanout_bitmask &= ~(1 << scanout_id);
    }

    // Bind the new resource
    res->scanout_bitmask |= (1 << scanout_id);
    log_debug("%s updated scanout %d to resource %d", __func__, scanout_id,
              res->resource_id);

    // Update scanout parameters and framebuffer
    scanout->resource_id = res->resource_id;
    scanout->x = r->x;
    scanout->y = r->y;
    scanout->width = r->width;
    scanout->height = r->height;
    scanout->frame_buffer = *fb;
}

void virtio_gpu_transfer_to_host_2d(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUDev *gdev = vdev->dev;

    GPUSimpleResource *res = NULL;
    uint32_t src_offset = 0;
    uint32_t dst_offset = 0;
    struct virtio_gpu_transfer_to_host_2d transfer_2d;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, transfer_2d);

    res = virtio_gpu_check_resource(vdev, transfer_2d.resource_id, __func__,
                                    &gcmd->error);
    if (!res) {
        return;
    }

    if (transfer_2d.r.x > res->width || transfer_2d.r.y > res->height ||
        transfer_2d.r.width > res->width ||
        transfer_2d.r.height > res->height ||
        transfer_2d.r.x + transfer_2d.r.width > res->width ||
        transfer_2d.r.y + transfer_2d.r.height > res->height) {
        log_error(
            "%s trying to transfer bounds outside resource %d bounds, (%d, "
            "%d) + %d, %d vs %d, %d",
            __func__, res->resource_id, transfer_2d.r.x, transfer_2d.r.y,
            transfer_2d.r.width, transfer_2d.r.height, res->width, res->height);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER;
        return;
    }

    log_debug(
        "%s transfering a region (%d, %d) + %d, %d from resource %d %d, %d",
        __func__, transfer_2d.r.x, transfer_2d.r.y, transfer_2d.r.width,
        transfer_2d.r.height, res->resource_id, res->width, res->height);

    // Retain transfer information, and perform the actual copy during flush
    res->transfer_rect.x = transfer_2d.r.x;
    res->transfer_rect.y = transfer_2d.r.y;
    res->transfer_rect.width = transfer_2d.r.width;
    res->transfer_rect.height = transfer_2d.r.height;
    res->transfer_offset = transfer_2d.offset;
}

void virtio_gpu_resource_attach_backing(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUSimpleResource *res = NULL;
    struct virtio_gpu_resource_attach_backing attach_backing;
    GPUDev *gdev = vdev->dev;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, attach_backing);

    // Check if the resource to be attached is already registered
    res = virtio_gpu_find_resource(gdev, attach_backing.resource_id);
    if (!res) {
        log_error("%s cannot find resource with id %d", __func__,
                  attach_backing.resource_id);
        gcmd->error = VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID;
        return;
    }

    if (res->iov) {
        log_error("%s found resource %d already has iov", __func__,
                  attach_backing.resource_id);
        gcmd->error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }

    log_debug("attaching guest mem to resource %d of gpu dev from zone %d",
              res->resource_id, vdev->zone_id);

    int err = virtio_gpu_create_mapping_iov(vdev, attach_backing.nr_entries,
                                            sizeof(attach_backing), gcmd,
                                            &res->iov, &res->iov_cnt);
    if (err != 0) {
        log_error("%s failed to map guest memory to iov", __func__);
        gcmd->error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        return;
    }
}

int virtio_gpu_create_mapping_iov(VirtIODevice *vdev, uint32_t nr_entries,
                                  uint32_t offset,
                                  GPUCommand *gcmd, /*uint64_t **addr,*/
                                  struct iovec **iov, uint32_t *niov) {
    log_debug("entering %s", __func__);
    GPUDev *gdev = vdev->dev;

    struct virtio_gpu_mem_entry *entries =
        NULL; // Array to store all guest memory entries
    size_t entries_size = 0;
    int e = 0;
    int v = 0;

    if (nr_entries > 16384) {
        log_error("%s found number of entries %d is too big (need to be less "
                  "than 16384)",
                  __func__, nr_entries);
        return -1;
    }

    // Allocate memory and manage guest memory with iov
    // Guest memory to host memory requires a layer of conversion
    entries_size = sizeof(*entries) * nr_entries;
    entries = malloc(entries_size);
    log_debug("%s got %d entries with total size %d", __func__, nr_entries,
              entries_size);
    // First copy all memory entries attached to the request to entries
    size_t s = iov_to_buf(gcmd->resp_iov, gcmd->resp_iov_cnt, offset, entries,
                          entries_size);
    if (s != entries_size) {
        log_error("%s failed to copy memory entries to buffer", __func__);
        free(entries);
        return -1;
    }

    *iov = NULL;
    // if (addr) {
    //   *addr = NULL;
    // }

    for (e = 0, v = 0; e < nr_entries; ++e, ++v) {
        uint64_t e_addr =
            entries[e].addr; // Starting position of the guest memory block
        uint32_t e_length =
            entries[e].length; // Length of the guest memory block

        // Since all memory of zonex will be mapped to zone0
        // And when zone starts, all memory of zonex will be mapped to the
        // virtual memory space of hvisor-tool
        // Therefore, here we only need to manage the virtual address of the
        // memory block used by zonex to store resource data with iov

        // Allocate iov in groups of 16, if not enough, reallocate memory
        if (!(v % 16)) {
            struct iovec *temp = realloc(*iov, (v + 16) * sizeof(struct iovec));
            if (temp == NULL) {
                // Unable to allocate
                log_error("%s cannot allocate enough memory for iov", __func__);
                free(*iov); // Directly free the iov array
                free(entries);
                *iov = NULL;
                return -1;
            }
            *iov = temp;
            // if (addr) {
            //   *addr = realloc(*addr, (v * 16) * sizeof(uint64_t));
            // }
        }

        (*iov)[v].iov_base = get_virt_addr((void *)e_addr, vdev->zone_id);
        (*iov)[v].iov_len = e_length;
        log_debug("guest addr %x map to %x with size %d", e_addr,
                  (*iov)[v].iov_base, (*iov)[v].iov_len);
        // if (addr) {
        //   (*addr)[v] = e_addr;
        // }

        // Considering that in future changes, the mapping from zonex to zone0
        // may not be direct, but through dma or other methods
        // Therefore, keep e and v to deal with the situation where entries and
        // iov do not correspond one-to-one
    }
    *niov = v;
    log_debug("%d memory blocks mapped", *niov);

    // Free entries
    free(entries);

    return 0;
}

// ! reserved
// void virtio_gpu_cleanup_mapping_iov(GPUDev *gdev, struct iovec *iov,
//                                     uint32_t iov_cnt) {}

void virtio_gpu_resource_detach_backing(VirtIODevice *vdev, GPUCommand *gcmd) {
    log_debug("entering %s", __func__);

    GPUDev *gdev = vdev->dev;

    GPUSimpleResource *res = NULL;
    struct virtio_gpu_resource_detach_backing detach;

    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt, detach);

    res = virtio_gpu_check_resource(vdev, detach.resource_id, __func__,
                                    &gcmd->error);
    if (!res) {
        return;
    }

    virtio_gpu_cleanup_mapping(gdev, res);
}

void virtio_gpu_simple_process_cmd(GPUCommand *gcmd, VirtIODevice *vdev) {
    log_debug("------ entering %s ------", __func__);

    gcmd->error = 0;
    gcmd->finished = false;

    // First fill in the cmd_hdr that each request has
    VIRTIO_GPU_FILL_CMD(gcmd->resp_iov, gcmd->resp_iov_cnt,
                        gcmd->control_header);

    // Jump to the corresponding processing function according to the type of
    // cmd_hdr
    /**********************************
     * The general 2D rendering call chain is
     * get_display_info->resource_create_2d->resource_attach_backing->set_scanout->get_display_info
     * (to determine if the setting is successful)
     * ->transfer_to_host_2d->resource_flush->*repeat transfer and flush*->end
     */
    switch (gcmd->control_header.type) {
    case VIRTIO_GPU_CMD_GET_DISPLAY_INFO:
        virtio_gpu_get_display_info(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_GET_EDID:
        virtio_gpu_get_edid(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_RESOURCE_CREATE_2D:
        virtio_gpu_resource_create_2d(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_RESOURCE_UNREF:
        virtio_gpu_resource_unref(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_RESOURCE_FLUSH:
        virtio_gpu_resource_flush(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D:
        virtio_gpu_transfer_to_host_2d(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_SET_SCANOUT:
        virtio_gpu_set_scanout(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING:
        virtio_gpu_resource_attach_backing(vdev, gcmd);
        break;
    case VIRTIO_GPU_CMD_RESOURCE_DETACH_BACKING:
        virtio_gpu_resource_detach_backing(vdev, gcmd);
        break;
    default:
        log_error("unknown request type");
        gcmd->error = VIRTIO_GPU_RESP_ERR_UNSPEC;
        break;
    }

    if (!gcmd->finished) {
        // If no response with data is returned directly, check for errors and
        // return a response without data
        if (gcmd->error) {
            log_error(
                "failed to handle virtio gpu request from zone %d, and request "
                "type is %d, error type is %d",
                vdev->zone_id, gcmd->control_header.type, gcmd->error);
        }
        virtio_gpu_ctrl_response_nodata(
            vdev, gcmd, gcmd->error ? gcmd->error : VIRTIO_GPU_RESP_OK_NODATA);
    }

    // Processing is complete, no need for iov
    free(gcmd->resp_iov);

    log_debug("------ leaving %s ------", __func__);
}