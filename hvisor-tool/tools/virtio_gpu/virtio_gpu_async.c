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
        // Check if the device is closed
        if (gdev->close) {
            pthread_mutex_unlock(&gdev->queue_mutex);
            pthread_exit(NULL);
            return NULL;
        }

        // Try to get a command
        // Holding the lock at this point
        while (!TAILQ_EMPTY(&gdev->command_queue)) {
            gcmd = TAILQ_FIRST(&gdev->command_queue);
            TAILQ_REMOVE(&gdev->command_queue, gcmd, next);

            pthread_mutex_unlock(&gdev->queue_mutex);

            // Release the lock and start processing
            virtio_gpu_simple_process_cmd(gcmd, vdev);
            // Notify the frontend and free memory after the command is
            // completed, iov is freed by virtio_gpu_simple_process_cmd

            request_cnt++;

            if (request_cnt >= VIRTIO_GPU_MAX_REQUEST_BEFORE_KICK) {
                // Processed a certain number of requests, kick the frontend
                virtio_inject_irq(&vdev->vqs[gcmd->from_queue]);
                request_cnt = 0;
                // log_info("%s: processed request >= 16, kick frontend",
                // __func__);
            }

            free(gcmd);

            // Since we are still in the processing loop, no need to worry about
            // losing awake
            // Re-acquire the lock on the next check
            pthread_mutex_lock(&gdev->queue_mutex);
        }

        if (request_cnt != 0) {
            // Processed requests but the task queue is empty, immediately kick
            // the frontend
            virtio_inject_irq(&vdev->vqs[gcmd->from_queue]);
            request_cnt = 0;
            // log_info("%s: request queue empty, kick frontend", __func__);
        }

        pthread_cond_wait(&gdev->gpu_cond, &gdev->queue_mutex);
    }
}