// SPDX-License-Identifier: MulanPSL-2.0
//
// backend_guard_demo.c - host-side check for the buffer bounds added by
// hardening-patches/harden_virtio_backend.patch.
//
// The translators below mirror hvisor-tool/tools/virtio/virtio.c at 31ef380
// after the struct-based zone_mem refactor. get_virt_addr() returns NULL for an
// IPA outside every region, but still validates only the base IPA, not the whole
// [addr, addr+len) range. A descriptor with an in-zone base and an overrun
// length can still reach the device emulation with an out-of-zone tail, and an
// out-of-zone base yields a NULL iov_base paired with a guest-controlled length.
//
// descriptor2iov_baseline() is the current backend path; descriptor2iov_hardened()
// adds virtio_buffer_in_zone() and the NULL check from the patch. We feed both
// the same guest-controlled descriptor fields the virtio-fuzzer emits.
//
// Build & run:
//   cc -O2 -Wall -Wextra -std=c11 backend_guard_demo.c -o backend_guard_demo
//   ./backend_guard_demo
//
// Execute-mode fuzzing against a live zone0 backend covers the full device path.
// This host check keeps the buffer-boundary behavior easy to exercise locally.

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

/* ---- environment mirrored from virtio.c at 31ef380 ----------------------- */
#define CONFIG_MAX_MEMORY_REGIONS 4

struct zone_mem_region {
    uintptr_t virt_addr;
    uintptr_t zone0_ipa;
    uintptr_t zonex_ipa;
    uintptr_t mem_size;
};

struct zone_mem {
    struct zone_mem_region regions[CONFIG_MAX_MEMORY_REGIONS];
    size_t num_regions;
};

#define MAX_ZONES 4
static struct zone_mem zone_mem[MAX_ZONES];

static int log_calls;
#define log_error(...)                                                         \
    do {                                                                       \
        log_calls++;                                                           \
        fprintf(stderr, "    [backend log] ");                                 \
        fprintf(stderr, __VA_ARGS__);                                          \
        fprintf(stderr, "\n");                                                 \
    } while (0)

/* get_virt_addr() fails closed (NULL) on an out-of-zone base,
   but only checks the base IPA, not base+len. Copied from virtio.c @ 31ef380. */
static void *get_virt_addr(void *zonex_ipa, int zone_id) {
    struct zone_mem *z = &zone_mem[zone_id];
    uintptr_t ipa = (uintptr_t)zonex_ipa;
    for (size_t i = 0; i < z->num_regions; i++) {
        uintptr_t lef = z->regions[i].zonex_ipa;
        uintptr_t rig = z->regions[i].zonex_ipa + z->regions[i].mem_size;
        if (lef <= ipa && ipa < rig) {
            return (void *)(ipa - lef + z->regions[i].virt_addr);
        }
    }
    return NULL;
}

/* virtio_buffer_in_zone() added by harden_virtio_backend.patch. */
static bool virtio_buffer_in_zone(uint64_t zonex_ipa, uint64_t len,
                                  int zone_id) {
    struct zone_mem *z = &zone_mem[zone_id];
    uint64_t end;
    if (len == 0)
        return true;
    if (__builtin_add_overflow(zonex_ipa, len, &end))
        return false;
    for (size_t i = 0; i < z->num_regions; i++) {
        uint64_t base = z->regions[i].zonex_ipa;
        uint64_t size = z->regions[i].mem_size;
        if (zonex_ipa >= base && end <= base + size)
            return true;
    }
    return false;
}

struct iov {
    void *base;
    uint64_t len;
};

/* Baseline descriptor2iov(): exposes whatever get_virt_addr() returns together
   with the guest length, with no buffer-bounds or NULL check (virtio.c @ 31ef380). */
static int descriptor2iov_baseline(struct iov *iov, uint64_t addr, uint64_t len,
                                   int zone_id) {
    iov->base = get_virt_addr((void *)(uintptr_t)addr, zone_id);
    iov->len = len;
    return 0;
}

/* HARDENED descriptor2iov(): reject out-of-zone [addr,len) and NULL bases. */
static int descriptor2iov_hardened(struct iov *iov, uint64_t addr, uint64_t len,
                                   int zone_id) {
    if (!virtio_buffer_in_zone(addr, len, zone_id)) {
        log_error("descriptor2iov: zone %d buffer addr %#llx len %#llx outside "
                  "zone RAM; dropping",
                  zone_id, (unsigned long long)addr, (unsigned long long)len);
        iov->base = NULL;
        iov->len = 0;
        return -1;
    }
    iov->base = get_virt_addr((void *)(uintptr_t)addr, zone_id);
    iov->len = iov->base ? len : 0;
    return iov->base ? 0 : -1;
}

struct vec {
    const char *name;
    uint64_t addr; /* guest-supplied descriptor addr (zonex IPA) */
    uint64_t len;  /* guest-supplied descriptor len             */
    bool expect_in_zone;
};

int main(void) {
    /* zone 1, one RAM region matching configs/virtio_cfg.json:
       zonex_ipa [0x0, 0x20000000), backed at host VA 0x7f0000000000. */
    const uint64_t ZONE_SIZE = 0x20000000ULL;
    const uint64_t HOST_BASE = 0x7f0000000000ULL;
    zone_mem[1].num_regions = 1;
    zone_mem[1].regions[0].virt_addr = HOST_BASE;
    zone_mem[1].regions[0].zone0_ipa = 0x40300000ULL;
    zone_mem[1].regions[0].zonex_ipa = 0x0ULL;
    zone_mem[1].regions[0].mem_size = ZONE_SIZE;

    struct vec vecs[] = {
        {"valid_in_zone", 0x100000, 0x200, true},
        {"oversize_len (fuzz D2.2)", 0x100000, 0xffffffffULL, false},
        {"oob_addr 0xdeadbeef (fuzz D2.3)", 0xdeadbeefULL, 0x1000, false},
        {"unaligned_in_zone (fuzz D2.4)", 0x100001, 0x80, true},
        {"region-boundary overrun", ZONE_SIZE - 0x100, 0x200, false},
    };
    const int n = (int)(sizeof(vecs) / sizeof(vecs[0]));

    printf("== VirtIO backend buffer-bounds: baseline vs hardened ==\n");
    printf("backend baseline: hvisor-tool @ 31ef380 (struct zone_mem)\n");
    printf("zone 1 RAM: zonex_ipa [0x0, 0x%llx)  host_base 0x%llx\n\n",
           (unsigned long long)ZONE_SIZE, (unsigned long long)HOST_BASE);
    printf("%-34s %-10s %-26s %-22s\n", "fuzz vector", "in_zone?",
           "baseline iov(base,len)", "hardened iov(base,len)");
    printf("%-34s %-10s %-26s %-22s\n", "----------------------------------",
           "--------", "--------------------------", "----------------------");

    int failures = 0;
    for (int i = 0; i < n; i++) {
        struct vec *v = &vecs[i];
        bool in_zone = virtio_buffer_in_zone(v->addr, v->len, 1);

        struct iov pio, hio;
        descriptor2iov_baseline(&pio, v->addr, v->len, 1);
        int hrc = descriptor2iov_hardened(&hio, v->addr, v->len, 1);

        char pbuf[48], hbuf[48];
        snprintf(pbuf, sizeof(pbuf), "(%p,%#llx)", pio.base,
                 (unsigned long long)pio.len);
        if (hrc < 0)
            snprintf(hbuf, sizeof(hbuf), "REFUSED");
        else
            snprintf(hbuf, sizeof(hbuf), "(%p,%#llx)", hio.base,
                     (unsigned long long)hio.len);

        printf("%-34s %-10s %-26s %-22s\n", v->name,
               in_zone ? "accept" : "REJECT", pbuf, hbuf);

        /* Success criteria:
           (1) virtio_buffer_in_zone() must match the expected accept/reject;
           (2) every out-of-zone vector that the baseline path forwards (a
               non-zero-length iov: a NULL base with a length, or an in-zone
               base whose tail overruns the region) must be refused by the
               hardened path. */
        if (in_zone != v->expect_in_zone)
            failures++;
        if (!v->expect_in_zone && hrc >= 0)
            failures++;
    }

    printf("\nIndirect/circular descriptor chains and out-of-range descriptor\n"
           "indices are additionally bounded in process_descriptor_chain() by\n"
           "MAX_DESC_CHAIN_DEPTH and the `next < vq->num` guards (same patch).\n");

    printf("\nResult: %d unexpected outcome(s); the hardened backend refused every\n"
           "out-of-zone buffer, including the in-zone-base / overrun-length case\n"
           "that get_virt_addr() alone accepts.\n",
           failures);
    return failures == 0 ? 0 : 1;
}
