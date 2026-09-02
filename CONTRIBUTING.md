# hvisor Contribution Guide

Thank you for contributing to hvisor. hvisor is a Type-1 hypervisor that runs on bare metal, so changes can affect boot, memory isolation, interrupts, device access, and multiple CPU architectures. This guide defines the minimum requirements for code quality, testing, and pull requests (PRs); maintainers may provide additional requirements in an Issue or PR.

## Contents

- [Before You Start](#before-you-start)
- [Collaboration](#collaboration)
  - [Repository Responsibilities](#repository-responsibilities)
  - [Assessing Change Impact](#assessing-change-impact)
  - [Integration Testing](#integration-testing)
  - [Cross-Repository PRs](#cross-repository-prs)
- [Quality and Testing Requirements](#quality-and-testing-requirements)
- [AI Agent Guidelines](#ai-agent-guidelines)
- [Submitting a PR](#submitting-a-pr)
- [License](#license)

## Before You Start

1. Read the [README](./README.md) and the documentation for the target platform. Check whether the problem is already covered by an Issue or PR.
2. For large features, interface changes, or architecture changes, open an Issue describing the motivation, scope, and proposed approach first. Small fixes may go directly into a PR.
3. hvisor pins its Rust toolchain in `rust-toolchain.toml`. Prepare `rust-src`, `rustfmt`, `LLVM/cargo-binutils`, `QEMU`, the cross compiler required by the target platform, and the required `Python` dependencies. Hardware testing also requires the relevant board, serial connection, and network environment.
4. Before and after making a change, identify its impact on architectures (`aarch64`, `riscv64`, `loongarch64`, and `x86_64`), boards, zones, devices, configuration files, and user-visible behavior. Keep each PR focused on one issue where possible. Describe the impact on shared cross-architecture code separately from platform-specific code.

## Collaboration

hvisor's runtime, management tools, and developer documentation are maintained in separate repositories. Before making a change, trace its impact through the runtime implementation, management entry point, and user documentation. A successful hvisor build alone does not demonstrate that a cross-repository feature works.

### Repository Responsibilities

| Repository | Responsibilities | Relationship |
| --- | --- | --- |
| [hvisor](https://github.com/syswonder/hvisor) | Type-1 hypervisor kernel, hypercalls, device virtualization, zone lifecycle, and platform support | Defines runtime behavior and cross-boundary ABI |
| [hvisor-tool](https://github.com/syswonder/hvisor-tool) | The `hvisor` command, `hvisor.ko`, and management interfaces in zone0 | Calls hvisor through hypercalls, shared memory, ioctls, and configuration files |
| [hvisor-book](https://github.com/syswonder/hvisor-book) | User and developer manuals, platform documentation, and feature descriptions | Documents the actual build, configuration, and usage of hvisor and hvisor-tool |

The responsibilities of these repositories are complementary: hvisor defines what the system can do, hvisor-tool defines how users invoke it, and hvisor-book explains how users should use it.

### Assessing Change Impact

Before implementing a change, use the following boundaries to decide whether other repositories must be updated:

| Change | hvisor-tool | hvisor-book | Required handling |
| --- | --- | --- | --- |
| Internal implementation only, with no change to ABI, configuration, commands, or user-visible behavior | Run a regression with a matching version | Usually no change | A hvisor-only PR is acceptable, but record the tool version and end-to-end result |
| Changes to hypercalls, error codes, cross-boundary structures, shared-memory/VirtIO versions, ioctls, or configuration fields | Must be evaluated; usually requires a companion PR | Update affected interfaces and examples | Describe the compatibility window, merge order, and rollback method |
| Changes to commands, device behavior, platform support, boot steps, or limitations | Synchronize as required by the interface change | Must be updated | Verify that commands and manual examples match the actual versions |

Cross-boundary interfaces must follow these rules:

- Never reuse a published hypercall number. Structure field order, size, alignment, the length of data referenced by pointers, and object lifetime are all part of the ABI.
- Prefer backward-compatible protocol extensions. When compatibility is impossible, check versions explicitly and return a diagnosable error instead of silently degrading.
- When build parameters, the boot chain, device trees, zone resources, JSON configuration, VirtIO/PCI/IVC behavior, or test steps change, check the corresponding hvisor-book chapters. Pure internal refactoring that does not change user behavior does not require new documentation.

### Integration Testing

For cross-repository integration, pin and record the branch or commit of each repository. Do not rely on an undocumented local worktree state. Build a matching hvisor-tool for the target architecture and Linux kernel version, deploy the `hvisor` userspace command, `hvisor.ko`, and configuration into zone0, and verify the minimum end-to-end loop on the target QEMU platform or development board:

```text
Load hvisor.ko -> hvisor zone list -> zone start -> confirm the guest is running
                         -> zone shutdown -> confirm the zone state again
```

For VirtIO, device passthrough, IVC, or exception-path changes, also cover successful operations, invalid inputs, and failure paths. Record build parameters, configuration files, tool and hvisor versions, key commands, zone states, and guest serial output or logs in the PR. Hardware tests must also record the board, firmware, and Linux versions. For manual changes, run `mdbook build` in hvisor-book and walk through the updated examples.

### Cross-Repository PRs

For a cross-repository feature, describe the interface and migration plan in an Issue first. Then open separate hvisor, hvisor-tool, and hvisor-book PRs as needed. Link the related PRs or Issues from each PR and include:

- The branch or commit of each repository, along with the specific interface, configuration, and command changes;
- The proposed merge order, compatibility window, upgrade/rollback method, and platforms that are not yet supported;
- End-to-end test commands, environments, results, and reasons for any tests not run.

By default, merge the hvisor compatibility foundation first, then hvisor-tool, and update hvisor-book last. If the actual dependency requires a different order, explain why in every related PR. The final release branch must not document commands, configuration, or platform support that does not exist or does not match the implementation.

## Quality and Testing Requirements

An acceptable implementation must meet the following requirements:

- The requirement, design, implementation, and tests must correspond to one another, with boundary conditions and failure paths handled.
- Zone isolation, memory permissions, device passthrough boundaries, I/O, and interrupt safety must not be weakened. For changes in these areas, the PR must state whether the relevant invariants are unchanged or explain the reason for changing them.
- Performance optimizations must have a measurement method and baseline; claims that something is "faster" are not sufficient. Behavior changes must update documentation, configuration examples, or the CHANGELOG where applicable.
- New logic should have reproducible unit, integration, or system tests. When hardware validation cannot be automated, record the board, firmware, kernel, commands, and key logs.

List each test command in the PR description and mark it as passed, failed with a reason, or not run with a reason. The minimum requirements are:

| Change type | Required verification |
| --- | --- |
| Documentation, comments, or formatting | `cargo fmt --all -- --check` when Rust files change, plus link and example checks |
| hvisor-book documentation or user-visible behavior | Run `mdbook build` in hvisor-book; for command, configuration, or platform changes, perform the minimum startup/regression steps from the manual and link the documentation change |
| Public Rust code or architecture-independent logic | Formatting, license checks, a target configuration build, `make ... clippy`, and `make ... test` when available |
| Architecture, page tables, memory, exceptions, interrupts, hypercalls, VirtIO, or PCI | The checks above, plus a zone0/zone1 startup and regression on QEMU or a development board with a matching hvisor-tool; cover successful, invalid-input, and failure paths |
| hvisor-tool commands, configuration, or ABI interaction | Build both repositories; verify module loading, `zone list/start/shutdown`, and affected VirtIO or device operations; link cross-repository PRs |
| Board, device tree, boot chain, or driver | Test on the relevant hardware. When hardware is unavailable, state the limitation and provide QEMU or static-check results |
| Performance or concurrency changes | Functional regression plus a reproducible benchmark or stress test with configuration, samples, and results; `ptest` data does not replace functional testing |

## AI Agent Guidelines

AI agents may assist with code search, documentation explanation, design proposals, first drafts, and tests. AI is an assistant, not a substitute for the author or tester. Contributors remain responsible for the final code, licenses, dependencies, test results, and security consequences.

- Manually inspect every line of AI-generated or AI-modified code, especially code involving `unsafe`, assembly, concurrency, memory mappings, permissions, or architecture-specific behavior. Do not skip validation because a model claims that something was tested.
- Do not submit code that you cannot explain, fabricated APIs or test results, unrelated bulk refactors, or generated content that is incompatible with the project license. Dependencies and code sources must be traceable.

## Submitting a PR

The repository provides a [PR template](./.github/pull_request_template.md) as a reference. GitHub may prefill it when a PR is created, but using the template is not mandatory. Adjust or remove sections that do not apply, and make sure the PR description contains the key details about the problem, approach, impact, verification, and risks.

## License

By contributing code to hvisor, you confirm that you have the right to submit it and agree that it will be released under the project's [Mulan PSL v2](./LICENSE). Confirm that third-party code, documentation, images, and dependencies are compatible with this license and preserve attribution where required.
