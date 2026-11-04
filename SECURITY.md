# hvisor Security Policy

hvisor is a Type-1 hypervisor whose main purpose is strong isolation between
zones. Security reports are taken seriously and handled confidentially.

README: [中文](./README-zh.md) | [English](./README.md)

## Reporting a Vulnerability

**Do not report suspected security vulnerabilities through public GitHub
Issues, discussions, or pull requests.**

Report them privately to:

> **security-alert@syswonder.org**

You will receive an acknowledgment within **5 business days**. If you do not
receive a reply within that time, follow up on the same thread or contact a
maintainer through another private channel.

Please include:

- hvisor version or commit hash (and the branch if built from source).
- Architecture (`aarch64`, `riscv64`, `loongarch64`, `x86_64`) and the
  platform/board (e.g., QEMU, RK3588, ZCU102).
- The zone configuration files used, or a minimal variant that reproduces the
  problem.
- A description of the vulnerability, its impact, and which security boundary
  it breaks (see [Scope and Trust Model](#scope-and-trust-model)).
- Reproduction steps, a proof of concept, or a crash log. Plain text is
  preferred; keep attachments small.
- Your name/handle if you wish to be credited in the fix.

Alternatively, GitHub's *privately report a vulnerability* feature
(**Security** tab → **Report a vulnerability**) may be used.

## Supported Versions

hvisor is a fast-moving research hypervisor; security fixes are applied to the
current development branch and the latest release only.

| Version | Security fixes |
| --- | --- |
| `dev` branch / latest release (v0.6.x) | ✅ Supported |
| v0.5 and older tags | ❌ Not supported — please upgrade |

Backports to older versions are considered only in exceptional cases; open a
private report and explain why you cannot upgrade.

## Scope and Trust Model

hvisor follows a separation-kernel design with static partitioning. The
security goal is that **a non-root zone can only access the resources
explicitly assigned to it in its configuration**.

### Security boundaries

A vulnerability is a defect that allows a non-root zone (zoneU, zoneR, or any
guest started by zone0) to:

1. Read or write hvisor memory, code, or internal state (escape from the
   guest to the hypervisor).
2. Read or write the memory, devices, or state of another zone (breakout from
   one guest into a sibling zone).
3. Access resources not assigned to it: pCPUs, physical memory regions,
   MMIO/PCIe ranges, interrupt controllers (GIC/PLIC/AIA/APIC), or DMA
   capable devices bypassing the intended IOMMU/virtio path.
4. Corrupt hvisor's integrity or availability through guest-triggerable
   input, e.g. hypercalls, virtio device models, interrupt/SGI injection, or
   trapped register access.
5. Obtain information that crosses the boundaries above (information leak).

### What is trusted by design

- **zone0 (root Linux) is trusted.** It manages zones through
  [hvisor-tool](https://github.com/syswonder/hvisor-tool), runs the virtio
  backends, and owns the platform configuration. A malicious or compromised
  zone0 can already destroy or reconfigure all zones, so attacks originating
  from zone0 are not considered security vulnerabilities. Defects reachable
  only by zone0 (e.g., a malformed zone config crashing hvisor) are fixed as
  robustness bugs, not security issues.
- **Guest operating systems and applications inside a zone are untrusted**
  toward every boundary listed above.

### Out of scope

- User misconfiguration of zone configurations that grants a zone more
  resources than intended (hvisor tries to detect overlaps, but a
  deliberately permissive configuration is the operator's choice).
- Bugs in guest kernels, root Linux, hvisor-tool, or device firmware that do
  not break an hvisor security boundary.
- Exhaustion of resources statically assigned to a zone (a zone
  overloading its own CPUs/memory/virtio devices).
- Attacks requiring physical access (JTAG, debug probes, cold boot, DMA from
  physically accessible ports).
- Hardware, firmware, or platform errata.
- Speculative-execution and microarchitectural side channels. Reports are
  accepted and evaluated case by case, but a full mitigation may be out of
  reach for a research hypervisor.

Bugs that are in scope for hvisor-tool (the `hvisor` command, `hvisor.ko`)
or hvisor-book should be reported to the respective
[syswonder](https://github.com/syswonder) repository, unless the defect is in
the hypervisor side of the hypercall ABI.

## AI-Assisted Reports

AI tools may be used to search, analyze, or draft a report, mirroring the
[contribution guidelines](./CONTRIBUTING.md): **AI is an assistant, not a
substitute for the reporter.** Verify the finding yourself before reporting,
state that and how AI was used, and remain responsible for the content.
Unverified AI-generated reports create significant work for volunteer
maintainers and may be closed without response.

## Non-Security Bugs

For ordinary bugs, feature requests, and documentation issues, please use
[GitHub Issues](https://github.com/syswonder/hvisor/issues) and follow the
[contribution guide](./CONTRIBUTING.md).
