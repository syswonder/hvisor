# SPDX-License-Identifier: MulanPSL-2.0

ROOT := $(abspath .)
BUILD := $(ROOT)/_build
RESULTS := $(ROOT)/results
REPORT_OUT := $(ROOT)/report/out
PYTHON ?= python3
PYRUN := PYTHONDONTWRITEBYTECODE=1 $(PYTHON)
CC ?= $(shell command -v musl-gcc 2>/dev/null || command -v gcc 2>/dev/null)
CFLAGS ?= -O2 -Wall -Wextra -D_GNU_SOURCE
LDLIBS ?=

TEST_SRCS := $(shell find tests -type f -name 't_*.c' | sort)
TEST_BINS := $(patsubst tests/%.c,$(BUILD)/tests/%.test,$(TEST_SRCS))
TEST_SCRIPTS := $(shell find tests/l5_observability tests/real_apps -type f -name '*.sh' | sort)
STAGED_SCRIPTS := $(patsubst tests/%.sh,$(BUILD)/tests/%.sh,$(TEST_SCRIPTS))

.PHONY: all tests clean-tests smoke report fixture verify clean initramfs lint-tools runner

all: tests runner

tests: clean-tests $(TEST_BINS) $(STAGED_SCRIPTS)

clean-tests:
	@rm -rf "$(BUILD)/tests"

runner: $(BUILD)/abi_runner.test

$(BUILD)/abi_runner.test: harness/abi_runner.c
	@mkdir -p $(BUILD)
	@if [ -z "$(CC)" ]; then echo "No C compiler found"; exit 1; fi
	@echo "CC harness/abi_runner.c"
	@$(CC) $(CFLAGS) -Wno-format-truncation harness/abi_runner.c -o "$@" -static $(LDLIBS) || { \
		echo "WARNING: static link failed for abi_runner; retrying dynamic link"; \
		$(CC) $(CFLAGS) -Wno-format-truncation harness/abi_runner.c -o "$@" $(LDLIBS); \
	}

$(BUILD)/tests/%.test: tests/%.c tests/test_common.h
	@mkdir -p $(dir $@)
	@if [ -z "$(CC)" ]; then echo "No C compiler found"; exit 1; fi
	@echo "CC $<"
	@$(CC) $(CFLAGS) -Itests "$<" -o "$@" -static $(LDLIBS) || { \
		echo "WARNING: static link failed for $<; retrying dynamic link"; \
		$(CC) $(CFLAGS) -Itests "$<" -o "$@" $(LDLIBS); \
	}

$(BUILD)/tests/%.sh: tests/%.sh
	@mkdir -p $(dir $@)
	@cp "$<" "$@"
	@chmod +x "$@"

smoke: tests runner
	@mkdir -p "$(BUILD)"
	@ABI_TEST_ROOT="$(BUILD)/tests" ABI_RESULT_JSON="$(BUILD)/smoke_results.json" \
		sh tests/run_all.sh
	@$(PYRUN) -m json.tool "$(BUILD)/smoke_results.json" >/dev/null
	@echo "smoke results: $(BUILD)/smoke_results.json"

fixture:
	@mkdir -p "$(RESULTS)/fixtures"
	@$(PYRUN) scripts/make_fixture_results.py --out "$(RESULTS)/fixtures"

report:
	@mkdir -p "$(REPORT_OUT)"
	@$(PYRUN) diff/syscall_diff.py \
		--catalog tests/catalog.json \
		--env-a "$(RESULTS)/env_a/results.json" \
		--env-b "$(RESULTS)/env_b/results.json" \
		--env-c "$(RESULTS)/env_c/results.json" \
		--ref-linux "$(RESULTS)/ref_linux_qemu/results.json" \
		--allow-missing \
		--out "$(REPORT_OUT)/diff_matrix.json"
	@$(PYRUN) report/gen_report.py \
		--diff "$(REPORT_OUT)/diff_matrix.json" \
		--out-dir "$(REPORT_OUT)"
	@echo "report: $(REPORT_OUT)/compatibility_report.md"

lint-tools:
	@$(PYRUN) scripts/check_python_syntax.py harness diff report scripts
	@$(PYRUN) scripts/check_catalog.py
	@$(PYRUN) scripts/validate_configs.py

initramfs: tests runner
	@sh initramfs/build_initramfs.sh

verify: lint-tools smoke fixture
	@$(PYRUN) diff/syscall_diff.py \
		--catalog tests/catalog.json \
		--env-a "$(RESULTS)/fixtures/env_a.json" \
		--env-b "$(RESULTS)/fixtures/env_b.json" \
		--env-c "$(RESULTS)/fixtures/env_c.json" \
		--out "$(REPORT_OUT)/fixture_diff_matrix.json"
	@$(PYRUN) report/gen_report.py \
		--diff "$(REPORT_OUT)/fixture_diff_matrix.json" \
		--out-dir "$(REPORT_OUT)/fixture"
	@sh scripts/check_spdx.sh
	@echo "verification complete"

clean:
	rm -rf "$(BUILD)" "$(REPORT_OUT)/fixture" "$(REPORT_OUT)/fixture_diff_matrix.json"
