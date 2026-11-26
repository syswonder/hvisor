// SPDX-License-Identifier: GPL-2.0-only
#include <errno.h>
#include <limits.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/un.h>
#include <unistd.h>

#include "log.h"
#include "virtio_internal.h"

#define VIRTIO_CTRL_SOCKET_PATH "/run/hvisor-virtio.sock"
#define VIRTIO_CTRL_OP_ADD "add"
#define VIRTIO_CTRL_OP_CLEANUP_ZONE "cleanup-zone"
#define VIRTIO_CTRL_MSG_LEN 256

typedef struct virtio_control_request {
    char op[16];
    char json_path[PATH_MAX];
} VirtioControlRequest;

typedef struct virtio_control_response {
    int status;
    char message[VIRTIO_CTRL_MSG_LEN];
} VirtioControlResponse;

static int ctrl_fd = -1;
static pthread_t ctrl_tid;
static atomic_bool ctrl_running = false;
static bool ctrl_thread_started;
static pthread_mutex_t ctrl_client_lock = PTHREAD_MUTEX_INITIALIZER;
static int ctrl_client_fd = -1;

static ssize_t read_full(int fd, void *buf, size_t len) {
    size_t off = 0;
    while (off < len) {
        ssize_t ret = read(fd, (char *)buf + off, len - off);
        if (ret == 0)
            return (ssize_t)off;
        if (ret < 0) {
            if (errno == EINTR)
                continue;
            return -1;
        }
        off += (size_t)ret;
    }
    return (ssize_t)off;
}

static ssize_t write_full(int fd, const void *buf, size_t len) {
    size_t off = 0;
    while (off < len) {
        ssize_t ret = write(fd, (const char *)buf + off, len - off);
        if (ret < 0) {
            if (errno == EINTR)
                continue;
            return -1;
        }
        if (ret == 0)
            return (ssize_t)off;
        off += (size_t)ret;
    }
    return (ssize_t)off;
}

void virtio_publish_mmio_addrs(void) {
    pthread_mutex_lock(&VDEV_MUTEX);
    for (int i = 0; i < vdevs_num; i++)
        virtio_bridge->mmio_addrs[i] = vdevs[i]->base_addr;
    for (int i = vdevs_num; i < MAX_DEVS; i++)
        virtio_bridge->mmio_addrs[i] = 0;
    pthread_mutex_unlock(&VDEV_MUTEX);
    write_barrier();
    virtio_bridge->mmio_avail = 1;
    write_barrier();
}

int virtio_add_from_json(const char *json_path) {
    int err = virtio_start_from_json((char *)json_path);
    if (err)
        return err;
    virtio_publish_mmio_addrs();
    return 0;
}

static void fill_control_response(VirtioControlResponse *resp, int status,
                                  const char *message) {
    memset(resp, 0, sizeof(*resp));
    resp->status = status;
    snprintf(resp->message, sizeof(resp->message), "%s", message);
}

static void handle_control_client(int client_fd) {
    VirtioControlRequest req;
    VirtioControlResponse resp;
    memset(&req, 0, sizeof(req));
    if (read_full(client_fd, &req, sizeof(req)) != sizeof(req))
        return;
    req.op[sizeof(req.op) - 1] = '\0';
    req.json_path[sizeof(req.json_path) - 1] = '\0';
    if (strcmp(req.op, VIRTIO_CTRL_OP_ADD) == 0) {
        if (virtio_add_from_json(req.json_path) != 0)
            fill_control_response(&resp, -1, "virtio add failed");
        else
            fill_control_response(&resp, 0, "virtio add succeeded");
    } else if (strcmp(req.op, VIRTIO_CTRL_OP_CLEANUP_ZONE) == 0) {
        char *end = NULL;
        errno = 0;
        unsigned long zone = strtoul(req.json_path, &end, 10);
        int ret = (errno || end == req.json_path || *end != '\0' || zone == 0 ||
                   zone >= MAX_ZONES)
                      ? -EINVAL
                      : virtio_cleanup_zone((uint32_t)zone);
        if (ret < 0)
            fill_control_response(&resp, ret, "virtio cleanup zone failed");
        else
            fill_control_response(&resp, ret, "virtio cleanup zone succeeded");
    } else {
        fill_control_response(&resp, -1,
                              "unsupported virtio control operation");
    }
    if (atomic_load(&ctrl_running))
        write_full(client_fd, &resp, sizeof(resp));
}

static void *virtio_control_loop(void *arg) {
    (void)arg;
    while (atomic_load(&ctrl_running)) {
        int client_fd = accept(ctrl_fd, NULL, NULL);
        if (client_fd < 0) {
            if (errno == EINTR)
                continue;
            if (!atomic_load(&ctrl_running) &&
                (errno == EINVAL || errno == EBADF || errno == ENOTSOCK))
                break;
            if (!atomic_load(&ctrl_running))
                break;
            log_error("accept virtio control client failed, errno is %d",
                      errno);
            continue;
        }
        pthread_mutex_lock(&ctrl_client_lock);
        if (!atomic_load(&ctrl_running)) {
            pthread_mutex_unlock(&ctrl_client_lock);
            shutdown(client_fd, SHUT_RDWR);
            close(client_fd);
            break;
        }
        ctrl_client_fd = client_fd;
        pthread_mutex_unlock(&ctrl_client_lock);
        handle_control_client(client_fd);
        pthread_mutex_lock(&ctrl_client_lock);
        if (ctrl_client_fd == client_fd)
            ctrl_client_fd = -1;
        pthread_mutex_unlock(&ctrl_client_lock);
        close(client_fd);
    }
    return NULL;
}

static int start_virtio_control_server(void) {
    struct sockaddr_un addr;
    unlink(VIRTIO_CTRL_SOCKET_PATH);
    ctrl_fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (ctrl_fd < 0)
        return -1;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s",
             VIRTIO_CTRL_SOCKET_PATH);
    mode_t old_umask = umask(0077);
    if (bind(ctrl_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        umask(old_umask);
        close(ctrl_fd);
        ctrl_fd = -1;
        unlink(VIRTIO_CTRL_SOCKET_PATH);
        return -1;
    }
    umask(old_umask);
    if (chmod(VIRTIO_CTRL_SOCKET_PATH, 0600) != 0) {
        log_error("could not restrict virtio control socket: %s",
                  strerror(errno));
        close(ctrl_fd);
        ctrl_fd = -1;
        unlink(VIRTIO_CTRL_SOCKET_PATH);
        return -1;
    }
    if (listen(ctrl_fd, 8) != 0) {
        close(ctrl_fd);
        ctrl_fd = -1;
        unlink(VIRTIO_CTRL_SOCKET_PATH);
        return -1;
    }
    pthread_mutex_lock(&ctrl_client_lock);
    ctrl_client_fd = -1;
    pthread_mutex_unlock(&ctrl_client_lock);
    atomic_store(&ctrl_running, true);
    if (pthread_create(&ctrl_tid, NULL, virtio_control_loop, NULL) != 0) {
        atomic_store(&ctrl_running, false);
        close(ctrl_fd);
        ctrl_fd = -1;
        unlink(VIRTIO_CTRL_SOCKET_PATH);
        return -1;
    }
    ctrl_thread_started = true;
    return 0;
}

void stop_virtio_control_server(void) {
    if (!ctrl_thread_started && ctrl_fd < 0)
        return;
    atomic_store(&ctrl_running, false);
    if (ctrl_fd >= 0)
        shutdown(ctrl_fd, SHUT_RDWR);
    pthread_mutex_lock(&ctrl_client_lock);
    int client_fd = ctrl_client_fd;
    pthread_mutex_unlock(&ctrl_client_lock);
    if (client_fd >= 0)
        shutdown(client_fd, SHUT_RDWR);
    if (ctrl_thread_started) {
        pthread_join(ctrl_tid, NULL);
        ctrl_thread_started = false;
    }
    pthread_mutex_lock(&ctrl_client_lock);
    ctrl_client_fd = -1;
    pthread_mutex_unlock(&ctrl_client_lock);
    if (ctrl_fd >= 0) {
        close(ctrl_fd);
        ctrl_fd = -1;
    }
    unlink(VIRTIO_CTRL_SOCKET_PATH);
}

int virtio_start(int argc, char *argv[]) {
    int err;
    if (argc < 4)
        return -1;
    err = virtio_init();
    if (err)
        return err;
    err = virtio_start_from_json(argv[3]);
    if (err)
        goto err_out;
    if (start_virtio_control_server() != 0) {
        err = -1;
        goto err_out;
    }
    virtio_publish_mmio_addrs();
    handle_virtio_requests();
    return 0;
err_out:
    virtio_close();
    return err;
}

int virtio_add(int argc, char *argv[]) {
    VirtioControlRequest req;
    VirtioControlResponse resp;
    struct sockaddr_un addr;
    char path[PATH_MAX];
    if (argc < 4 || !realpath(argv[3], path))
        return -1;
    int fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0)
        return -1;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s",
             VIRTIO_CTRL_SOCKET_PATH);
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        return -1;
    }
    memset(&req, 0, sizeof(req));
    snprintf(req.op, sizeof(req.op), "%s", VIRTIO_CTRL_OP_ADD);
    snprintf(req.json_path, sizeof(req.json_path), "%s", path);
    if (write_full(fd, &req, sizeof(req)) != sizeof(req) ||
        read_full(fd, &resp, sizeof(resp)) != sizeof(resp)) {
        close(fd);
        return -1;
    }
    close(fd);
    printf("%s\n", resp.message);
    return resp.status;
}

/*
 * Ask the daemon to quiesce/close a zone's virtio devices and unmap its guest
 * memory before the caller issues HVISOR_ZONE_SHUTDOWN.
 *
 * Returns VIRTIO_CTRL_NO_DAEMON (2) when no daemon is listening or when talking
 * to an older daemon, VIRTIO_CTRL_CLEANUP_RELEASED (1) / NONE (0) on success,
 * or a negative status on daemon/transport failure.
 */
int virtio_control_cleanup_zone(uint32_t zone_id) {
    VirtioControlRequest req;
    VirtioControlResponse resp;
    struct sockaddr_un addr;
    int fd;
    int rc;

    fd = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0)
        return -errno;
    struct timeval tv = {.tv_sec = 30, .tv_usec = 0};
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s",
             VIRTIO_CTRL_SOCKET_PATH);
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        int err = errno;
        close(fd);
        if (err == ENOENT || err == ECONNREFUSED) {
            log_info("virtio control cleanup zone %u: no daemon", zone_id);
            return VIRTIO_CTRL_NO_DAEMON;
        }
        log_error("virtio control connect failed for zone %u: %s", zone_id,
                  strerror(err));
        return -err;
    }
    memset(&req, 0, sizeof(req));
    snprintf(req.op, sizeof(req.op), "%s", VIRTIO_CTRL_OP_CLEANUP_ZONE);
    snprintf(req.json_path, sizeof(req.json_path), "%u", zone_id);
    if (write_full(fd, &req, sizeof(req)) != sizeof(req) ||
        read_full(fd, &resp, sizeof(resp)) != sizeof(resp)) {
        int err = errno ? errno : EIO;
        close(fd);
        return -err;
    }
    close(fd);
    if (resp.status == -1 && strstr(resp.message, "unsupported") != NULL) {
        log_warn("virtio daemon does not support cleanup-zone, continuing");
        return VIRTIO_CTRL_NO_DAEMON;
    }
    rc = resp.status;
    log_info("virtio control cleanup zone %u: status %d", zone_id, rc);
    return rc;
}
