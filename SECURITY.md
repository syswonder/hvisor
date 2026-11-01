# Security Policy

## Supported Versions

The hvisor project currently provides security updates for the latest release and the active development branch.

| Version | Supported |
| ------- | --------- |
| `dev`   | :white_check_mark: |
| `v0.6`  | :white_check_mark: |
| `v0.5` and earlier | :x: |

Security fixes are primarily developed against the `dev` branch and incorporated into supported releases when appropriate. Older releases are not guaranteed to receive security updates.

## Reporting a Vulnerability

If you discover a security vulnerability in hvisor, please report it responsibly.

**Please do not disclose security vulnerabilities through public GitHub Issues, Discussions, or Pull Requests.**

The preferred reporting method is GitHub's private vulnerability reporting feature:

- Navigate to the [hvisor Security Advisories](https://github.com/syswonder/hvisor/security/advisories) page.
- Click **Report a vulnerability** to submit a private report to the maintainers.

If private vulnerability reporting is unavailable, please open a GitHub Issue requesting a private communication channel. Do not include vulnerability details or exploit code in the public issue.

When reporting a vulnerability, please provide as much of the following information as possible:

- A description of the vulnerability and its potential security impact.
- The affected version, commit, and target architecture.
- The hardware platform or emulation environment (e.g., QEMU).
- Steps to reproduce the issue or a proof of concept (PoC).
- Relevant logs, configuration files, and other supporting evidence.

## Scope

The following types of security issues are particularly relevant to hvisor:

- **Virtualization isolation:** Unauthorized access between zones, VM escape, or privilege escalation to the hypervisor or management zone.
- **Memory isolation:** Invalid memory mappings, stage-2 translation vulnerabilities, or unauthorized access to host physical memory.
- **CPU virtualization:** Vulnerabilities in privilege transitions, trap handling, hypercalls, or virtual CPU state management.
- **I/O and DMA isolation:** Improper device passthrough, IOMMU protection bypasses, PCIe vulnerabilities, or unauthorized DMA access.
- **Interrupt and device virtualization:** Vulnerabilities in interrupt routing, interrupt injection, MMIO emulation, or Virtio device handling.
- **Memory safety and concurrency:** Memory corruption, race conditions, or other defects that can compromise the hypervisor or isolation boundaries.
- **Build and release security:** Vulnerabilities in project-controlled build workflows or release processes that could compromise hvisor binaries.

Issues that affect only a guest operating system without compromising hvisor or the isolation between zones are generally outside the scope of this policy.

## Response and Disclosure

The maintainers aim to:

- Acknowledge vulnerability reports within 5 business days.
- Provide an initial assessment within 10 business days.
- Keep reporters informed of the investigation and remediation progress.

If a vulnerability is confirmed, the maintainers will work with the reporter to develop a fix and coordinate its disclosure.

If a report is declined, the maintainers will provide an explanation where possible.

Please allow reasonable time for investigation and remediation before publicly disclosing a vulnerability. As a general guideline, coordinated disclosure is expected within 90 days of the initial report, subject to mutual agreement.

## Security Testing Guidelines

Security researchers are expected to:

- Test only on systems they own or have explicit authorization to test.
- Avoid testing that could disrupt production systems or affect other users.
- Avoid unauthorized access, data modification, or data exfiltration.
- Share vulnerability details privately with the maintainers before public disclosure.

The hvisor project does not currently offer a vulnerability bounty program.

We appreciate the contributions of security researchers who help improve the security and reliability of hvisor.
