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
#ifndef _HVISOR_VIRTIO_GPU_H
#define _HVISOR_VIRTIO_GPU_H
#ifdef ENABLE_VIRTIO_GPU

#include "bits/pthreadtypes.h"
#include "linux/types.h"
#include "sys/queue.h"
#include "virtio.h"
#include <linux/virtio_gpu.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <xf86drm.h>
#include <xf86drmMode.h>

/*********************************************************************
    Macro Definitions
 */
// controlq index
#define GPU_CONTROL_QUEUE 0

// cursorq index
#define GPU_CURSOR_QUEUE 1

// Number of virtqueues
#define GPU_MAX_QUEUES 2

// Maximum number of elements in a virtqueue
#define VIRTQUEUE_GPU_MAX_SIZE 256

// Kick the frontend immediately after processing more than this number of
// requests
#define VIRTIO_GPU_MAX_REQUEST_BEFORE_KICK 16

// Maximum number of scanouts supported by hvisor's virtio_gpu implementation
#define HVISOR_VIRTIO_GPU_MAX_SCANOUTS 1

// Maximum memory used for storing resources by a virtio gpu device
#define VIRTIO_GPU_MAX_HOSTMEM 536870912 // 512MB

// Supported virtio features
// Optional VIRTIO_RING_F_INDIRECT_DESC and VIRTIO_RING_F_EVENT_IDX
// Pending support for VIRTIO_GPU_F_EDID, VIRTIO_GPU_F_RESOURCE_UUID,
// VIRTIO_GPU_F_RESOURCE_BLOB, VIRTIO_GPU_F_VIRGL, VIRTIO_GPU_F_CONTEXT_INIT
#define GPU_SUPPORTED_FEATURES                                                 \
    ((1ULL << VIRTIO_F_VERSION_1) | VIRTIO_RING_F_INDIRECT_DESC)

// Default configuration for scanout[0]
#define SCANOUT_DEFAULT_WIDTH 1280

#define SCANOUT_DEFAULT_HEIGHT 800

// Macro to find the minimum value
#define MIN(a, b) ((a) < (b) ? (a) : (b))

// Conversion between virtio_gpu_formats and drm formats
#define VIRTIO_GPU_FORMAT_TO_DRM_FORMAT(format)                                \
    ((format == VIRTIO_GPU_FORMAT_X8R8G8B8_UNORM)   ? DRM_FORMAT_XRGB8888      \
     : (format == VIRTIO_GPU_FORMAT_X8B8G8R8_UNORM) ? DRM_FORMAT_XBGR8888      \
     : (format == VIRTIO_GPU_FORMAT_R8G8B8X8_UNORM) ? DRM_FORMAT_RGBX8888      \
     : (format == VIRTIO_GPU_FORMAT_B8G8R8X8_UNORM) ? DRM_FORMAT_BGRX8888      \
     : (format == VIRTIO_GPU_FORMAT_A8R8G8B8_UNORM) ? DRM_FORMAT_ARGB8888      \
     : (format == VIRTIO_GPU_FORMAT_A8B8G8R8_UNORM) ? DRM_FORMAT_ABGR8888      \
     : (format == VIRTIO_GPU_FORMAT_R8G8B8A8_UNORM) ? DRM_FORMAT_RGBA8888      \
     : (format == VIRTIO_GPU_FORMAT_B8G8R8A8_UNORM) ? DRM_FORMAT_BGRA8888      \
                                                    : 0 /* Unknown format */)

/*********************************************************************
    Structures
 */
// virtio_gpu_config as follows
// #define VIRTIO_GPU_EVENT_DISPLAY (1 << 0)
// struct virtio_gpu_config {
// 	__u32 events_read;  // Events waiting to be read by the driver
// (VIRTIO_GPU_EVENT_DISPLAY)
// 	__u32 events_clear; // Clear waiting events in the driver
// 	__u32 num_scanouts; // Maximum number of supported scanouts, see related
// macro definition VIRTIO_GPU_MAX_SCANOUTS
// 	__u32 num_capsets;  // Device's capability sets, see related macro
// definition VIRTIO_GPU_CAPSET_*
// };

// Clear events
// events_read &= ~events_clear

typedef struct virtio_gpu_config GPUConfig;
typedef struct virtio_gpu_ctrl_hdr GPUControlHeader;
typedef struct virtio_gpu_update_cursor GPUUpdateCursor;

// Resource object stored in memory during rendering (e.g., images)
// When in use, it needs to be converted from iov to a drm_mode_create_dumb
// object for output
typedef struct virtio_gpu_simple_resource {
    uint32_t resource_id;   // Resource ID
    uint32_t width, height; // Resource width and height
    uint32_t format; // Resource format, combined with width and height to
                     // calculate the memory size used by the resource
    // addrs is an array of physical addresses of the resource in the guest, as
    // zonex will directly map to the virtual memory space of hvisor-tool, this
    // field is temporarily not needed uint64_t *addrs;
    struct iovec *iov; // Use iov to store the resource
    unsigned int iov_cnt;
    uint64_t hostmem;         // Size of the resource in the host
    uint32_t scanout_bitmask; // Marks which scanout the resource is used by
    struct virtio_gpu_rect
        transfer_rect; // In transfer_to_2d, determines which part of the image
                       // in guest memory to flush
    uint64_t transfer_offset;
    TAILQ_ENTRY(virtio_gpu_simple_resource) next;
} GPUSimpleResource;

typedef struct virtio_gpu_framebuffer {
    // TODO: Double buffering
    uint32_t fb_id; // Framebuffer ID
    // TODO: Format
    // The format mainly determines how many bytes each pixel occupies
    // The formats provided by virtio_gpu_formats are all 4 bytes per pixel
    uint32_t format;        // virtio_gpu_formats
    uint32_t bytes_pp;      // Bytes per pixel
    uint32_t width, height; // Framebuffer width and height
    uint32_t stride; // Stride refers to the number of bytes each row of the
                     // image occupies in memory, stride * height equals the
                     // total number of bytes (hostmem)
    uint32_t offset;
    // drm related
    uint32_t drm_dumb_size;   // Buffer size
    uint32_t drm_dumb_handle; // Handle pointing to the drm framebuffer
    void *fb_addr;            // Virtual address of the buffer in this process
    bool enabled;             // Whether the buffer is enabled
} GPUFrameBuffer;

// 32-bit RGBA
typedef struct hvisor_cursor {
    uint16_t width, height;
    int hot_x, hot_y;
    int refcount;
    uint32_t data[];
} HvCursor;

// Settings related to the actual display device
typedef struct virtio_gpu_scanout {
    uint32_t width, height;
    uint32_t x, y;
    uint32_t resource_id;
    GPUUpdateCursor cursor;
    HvCursor *current_cursor;
    GPUFrameBuffer frame_buffer;
    // Output card used
    int card0_fd;
    // drm related
    drmModeCrtc *crtc;
    drmModeEncoder *encoder;
    drmModeConnector *connector;
} GPUScanout;

// Settings of the display device specified by json
typedef struct virtio_gpu_requested_state {
    uint32_t width, height;
    int x, y;
} GPURequestedState;

// GPU device structure
typedef struct virtio_gpu_dev {
    GPUConfig config;
    // TODO: Initialize scanouts
    // Pre-allocated scanouts owned by virtio gpu
    GPUScanout scanouts[HVISOR_VIRTIO_GPU_MAX_SCANOUTS];
    // TODO: Initialize requested_state
    // Requested states negotiated by these scanouts
    GPURequestedState requested_states[HVISOR_VIRTIO_GPU_MAX_SCANOUTS];
    // Resources owned by virtio gpu
    TAILQ_HEAD(, virtio_gpu_simple_resource) resource_list;
    // Command queue for async processing by virtio gpu
    TAILQ_HEAD(, virtio_gpu_control_cmd) command_queue;
    // Number of scanouts
    int scanouts_num;
    // Total memory occupied by the virtio device
    uint64_t hostmem;
    // Enabled scanout
    int enabled_scanout_bitmask;
    // async
    pthread_t gpu_thread;
    pthread_cond_t gpu_cond;
    pthread_mutex_t queue_mutex;
    bool close;
} GPUDev;

typedef struct virtio_gpu_control_cmd {
    GPUControlHeader control_header;
    struct iovec *resp_iov;    // iov to write the response
    unsigned int resp_iov_cnt; // Number of iovs
    uint16_t
        resp_idx;   // Used index corresponding to the request after completion
    bool finished;  // Indicates whether the current cmd is completed after
                    // processing, if not, use no_data response uniformly
    uint32_t error; // Error type
    uint32_t from_queue; // Queue from which the request came
    TAILQ_ENTRY(virtio_gpu_control_cmd) next; // Next cmd in the command queue
} GPUCommand;

/*********************************************************************
  virtio_gpu_base.c
 */
// Initialize GPUDev structure
GPUDev *init_gpu_dev(GPURequestedState *requested_states);

// Initialize virtio-gpu device
int virtio_gpu_init(VirtIODevice *vdev);

// Close virtio-gpu device
void virtio_gpu_close(VirtIODevice *vdev);

// Reset virtio-gpu device
void virtio_gpu_reset();

// Handler function when controlq has requests to process
int virtio_gpu_ctrl_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

// Handler function when cursorq has requests to process
int virtio_gpu_cursor_notify_handler(VirtIODevice *vdev, VirtQueue *vq);

// Process a single request
int virtio_gpu_handle_single_request(VirtIODevice *vdev, VirtQueue *vq,
                                     uint32_t from);

// Copy request control structure from iov
#define VIRTIO_GPU_FILL_CMD(iov, iov_cnt, out)                                 \
    do {                                                                       \
        size_t virtiogpufillcmd_s_ =                                           \
            iov_to_buf(iov, iov_cnt, 0, &(out), sizeof(out));                  \
        if (virtiogpufillcmd_s_ != sizeof(out)) {                              \
            log_error("cannot fill virtio gpu command with input!");           \
            return;                                                            \
        }                                                                      \
    } while (0)

// Copy arbitrary size from iov to buffer
size_t iov_to_buf_full(const struct iovec *iov, unsigned int iov_cnt,
                       size_t offset, void *buf, size_t bytes_need_copy);

// Copy arbitrary size from buffer to iov
size_t buf_to_iov_full(const struct iovec *iov, unsigned int iov_cnt,
                       size_t offset, const void *buf, size_t bytes_need_copy);

// Fill buffer from iov
// Often used when only a field in the command structure is needed, or when the
// vq length is only 2 Therefore, optimization is needed when copying can be
// completed in only one buffer of the iov
static inline size_t iov_to_buf(const struct iovec *iov, // NOLINT
                                const unsigned int iov_cnt, size_t offset,
                                void *buf, size_t bytes_need_copy) {
    if (__builtin_constant_p(bytes_need_copy) && iov_cnt &&
        offset <= iov[0].iov_len &&
        bytes_need_copy <= iov[0].iov_len - offset) {
        // If the copy operation can be completed in the first buffer of the iov
        // Copy and return immediately
        memcpy(buf, (char *)iov[0].iov_base + offset, bytes_need_copy);
        return bytes_need_copy;
    }
    // Need to copy from multiple buffers of the iov
    return iov_to_buf_full(iov, iov_cnt, offset, buf, bytes_need_copy);
}

// Fill iov from buffer, single optimization
static inline size_t buf_to_iov(const struct iovec *iov, // NOLINT
                                unsigned int iov_cnt, size_t offset,
                                const void *buf, size_t bytes_need_copy) {
    if (__builtin_constant_p(bytes_need_copy) && iov_cnt &&
        offset <= iov[0].iov_len &&
        bytes_need_copy <= iov[0].iov_len - offset) {
        memcpy((char *)iov[0].iov_base + offset, buf, bytes_need_copy);
        return bytes_need_copy;
    }
    return buf_to_iov_full(iov, iov_cnt, offset, buf, bytes_need_copy);
}

/*********************************************************************
  virtio_gpu.c
 */
// Return response with data
void virtio_gpu_ctrl_response(VirtIODevice *vdev, GPUCommand *gcmd,
                              GPUControlHeader *resp, size_t resp_len);

// Return response without data
void virtio_gpu_ctrl_response_nodata(VirtIODevice *vdev, GPUCommand *gcmd,
                                     enum virtio_gpu_ctrl_type type);

// Corresponding to VIRTIO_GPU_CMD_GET_DISPLAY_INFO
// Return current output configuration
void virtio_gpu_get_display_info(VirtIODevice *vdev, GPUCommand *gcmd);

// Corresponding to VIRTIO_GPU_CMD_GET_EDID
// Return current EDID information
void virtio_gpu_get_edid(VirtIODevice *vdev, GPUCommand *gcmd);

// Corresponding to VIRTIO_GPU_CMD_RESOURCE_CREATE_2D
// Create a 2D resource on the host with the given id, width, height, and format
// from the guest
void virtio_gpu_resource_create_2d(VirtIODevice *vdev, GPUCommand *gcmd);

// Check if the given virtio gpu device has a resource with the id resource_id
// If not, return NULL
GPUSimpleResource *virtio_gpu_find_resource(GPUDev *gdev, uint32_t resource_id);

// Check if the resource with the specified id is already bound, if so, return
// its pointer Otherwise, if there is no resource corresponding to the id, or
// the resource is not bound, return NULL
GPUSimpleResource *virtio_gpu_check_resource(VirtIODevice *vdev,
                                             uint32_t resource_id,
                                             const char *caller,
                                             uint32_t *error);

// Calculate the memory size occupied by the resource in the host
uint32_t calc_image_hostmem(int bits_per_pixel, uint32_t width,
                            uint32_t height);

// Corresponding to VIRTIO_GPU_CMD_RESOURCE_UNREF
// Destroy a resource
void virtio_gpu_resource_unref(VirtIODevice *vdev, GPUCommand *gcmd);

// Destroy the specified resource
void virtio_gpu_resource_destory(GPUDev *gdev, GPUSimpleResource *res);

// Unregister the resource of the given scanout, making the scanout invalid
void virtio_gpu_disable_scanout(GPUDev *gdev, int scanout_id);

// Clear the mapping of the resource
void virtio_gpu_cleanup_mapping(GPUDev *gdev, GPUSimpleResource *res);

// Corresponding to VIRTIO_GPU_CMD_RESOURCE_FLUSH
// Flush a resource that is linked to a scanout
void virtio_gpu_resource_flush(VirtIODevice *vdev, GPUCommand *gcmd);

// Create a drm_framebuffer for the scanout
void virtio_gpu_create_drm_framebuffer(GPUScanout *scanout, uint32_t *error);

// Copy the resource to the scanout's drm_framebuffer and flush
void virtio_gpu_copy_and_flush(GPUScanout *scanout, GPUSimpleResource *res,
                               uint32_t *error);

// Remove the drm_framebuffer of the scanout
void virtio_gpu_remove_drm_framebuffer(GPUScanout *scanout);

// Corresponding to VIRTIO_GPU_CMD_SET_SCANOUT
// Set the display parameters of the scanout and bind the resource to the
// scanout
void virtio_gpu_set_scanout(VirtIODevice *vdev, GPUCommand *gcmd);

// Specifically set the scanout according to the parameters
bool virtio_gpu_do_set_scanout(VirtIODevice *vdev, uint32_t scanout_id,
                               GPUFrameBuffer *fb, GPUSimpleResource *res,
                               struct virtio_gpu_rect *r, uint32_t *error);

// Update the scanout
void virtio_gpu_update_scanout(VirtIODevice *vdev, uint32_t scanout_id,
                               GPUFrameBuffer *fb, GPUSimpleResource *res,
                               struct virtio_gpu_rect *r);

// Corresponding to VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D
// Transfer the content in guest memory to the resource in the host
void virtio_gpu_transfer_to_host_2d(VirtIODevice *vdev, GPUCommand *gcmd);

// Corresponding to VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING
// Bind multiple memory regions in the guest to the resource (as backing
// storage)
void virtio_gpu_resource_attach_backing(VirtIODevice *vdev, GPUCommand *gcmd);

// Map guest memory to host's iov
int virtio_gpu_create_mapping_iov(VirtIODevice *vdev, uint32_t nr_entries,
                                  uint32_t offset,
                                  GPUCommand *gcmd, /*uint64_t **addr,*/
                                  struct iovec **iov, uint32_t *niov);

// ! reserved
// Clear mapping
// void virtio_gpu_cleanup_mapping_iov(GPUDev *gdev, struct iovec *iov,
//                                     uint32_t iov_cnt);

// Corresponding to VIRTIO_GPU_CMD_RESOURCE_DETACH_BACKING
// Unbind the memory regions in the guest from the resource
void virtio_gpu_resource_detach_backing(VirtIODevice *vdev, GPUCommand *gcmd);

// Process the request according to the control header
void virtio_gpu_simple_process_cmd(GPUCommand *gcmd, VirtIODevice *vdev);

/*********************************************************************
  virtio_gpu_async.c
 */
// Processing thread
void *virtio_gpu_handler(void *vdev);

#endif
#endif /* _HVISOR_VIRTIO_GPU_H */