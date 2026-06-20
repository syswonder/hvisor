# SPDX-License-Identifier: MulanPSL-2.0
# Copyright (c) 2026 hvisor contributors

CC ?= gcc
CFLAGS ?= -O2 -g -Wall -Wextra -Werror -std=gnu11
LDFLAGS ?=
BUILD_DIR ?= build
BIN_DIR ?= $(BUILD_DIR)/bin
GUEST_BIN_DIR ?= $(BUILD_DIR)/guest-bin

# Guest binaries are statically linked so they run inside the Asterinas guest
# (which provides a Linux ABI but not necessarily the host dynamic loader/libc).
GUEST_CFLAGS ?= -O2 -Wall -Wextra -Werror -std=gnu11 -static
GUEST_LDFLAGS ?= -static -s

GUEST_BINS := \
	$(GUEST_BIN_DIR)/smp_probe \
	$(GUEST_BIN_DIR)/ipi_pingpong \
	$(GUEST_BIN_DIR)/timer_jitter \
	$(GUEST_BIN_DIR)/ctxsw \
	$(GUEST_BIN_DIR)/mem_burn \
	$(GUEST_BIN_DIR)/probe_selftest \
	$(GUEST_BIN_DIR)/noisy_init

.PHONY: all probes noisy guest clean test

all: probes noisy

probes: $(BIN_DIR)/smp_probe $(BIN_DIR)/ipi_pingpong $(BIN_DIR)/timer_jitter $(BIN_DIR)/ctxsw

noisy: $(BIN_DIR)/mem_burn

# Statically-linked guest payload (probes + self-test launcher) for Asterinas.
guest: $(GUEST_BINS)

$(BIN_DIR):
	mkdir -p $@

$(GUEST_BIN_DIR):
	mkdir -p $@

$(BIN_DIR)/smp_probe: probes/smp_probe/smp_probe.c | $(BIN_DIR)
	$(CC) $(CFLAGS) $< -o $@ -pthread $(LDFLAGS)

$(BIN_DIR)/ipi_pingpong: probes/ipi_pingpong/ipi_pingpong.c | $(BIN_DIR)
	$(CC) $(CFLAGS) $< -o $@ -pthread $(LDFLAGS)

$(BIN_DIR)/timer_jitter: probes/timer_jitter/timer_jitter.c | $(BIN_DIR)
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)

$(BIN_DIR)/ctxsw: probes/ctxsw/ctxsw.c | $(BIN_DIR)
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)

$(BIN_DIR)/mem_burn: noisy/mem_burn.c | $(BIN_DIR)
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)

$(GUEST_BIN_DIR)/smp_probe: probes/smp_probe/smp_probe.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ -pthread $(GUEST_LDFLAGS)

$(GUEST_BIN_DIR)/ipi_pingpong: probes/ipi_pingpong/ipi_pingpong.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ -pthread $(GUEST_LDFLAGS)

$(GUEST_BIN_DIR)/timer_jitter: probes/timer_jitter/timer_jitter.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ $(GUEST_LDFLAGS)

$(GUEST_BIN_DIR)/ctxsw: probes/ctxsw/ctxsw.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ $(GUEST_LDFLAGS)

$(GUEST_BIN_DIR)/mem_burn: noisy/mem_burn.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ $(GUEST_LDFLAGS)

$(GUEST_BIN_DIR)/probe_selftest: rootfs/probe_selftest_init.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ $(GUEST_LDFLAGS)

$(GUEST_BIN_DIR)/noisy_init: rootfs/noisy_init.c | $(GUEST_BIN_DIR)
	$(CC) $(GUEST_CFLAGS) $< -o $@ $(GUEST_LDFLAGS)

# Local smoke test: build host probes, validate zone configs, and exercise the
# stats pipeline on a tiny known stream.
test: probes
	python3 scripts/check_configs.py
	printf '1\n2\n3\n4\n5\n' | python3 pipeline/stats.py

clean:
	rm -rf $(BUILD_DIR)
