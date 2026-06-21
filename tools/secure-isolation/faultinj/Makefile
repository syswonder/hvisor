CC ?= cc
CFLAGS ?= -O2 -Wall -Wextra -std=c11
LDFLAGS ?=

BUILD_DIR := build
TESTS := test_unmapped_ipa test_cross_zone_mem test_unauthorized_mmio test_noisy_neighbor

.PHONY: all clean

all: $(addprefix $(BUILD_DIR)/,$(TESTS))

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/%: tests/%.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)

clean:
	rm -rf $(BUILD_DIR)
