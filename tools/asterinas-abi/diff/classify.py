# SPDX-License-Identifier: MulanPSL-2.0
"""Attribution rules for A/B/C Linux ABI differential results."""

PASS = "PASS"
FAIL = "FAIL"
SKIP = "SKIP"
MISSING = "MISSING"


def normalize_status(value):
    if value is None:
        return MISSING
    value = str(value).upper()
    if value in {PASS, FAIL, SKIP, MISSING}:
        return value
    if value in {"OK", "SUCCESS"}:
        return PASS
    if value in {"ERROR", "TIMEOUT"}:
        return FAIL
    return value


def is_pass(value):
    return normalize_status(value) == PASS


def is_fail(value):
    return normalize_status(value) == FAIL


def classify(env_a, env_b, env_c):
    """Return attribution tag and explanation for the A/B/C differential.

    Env A: Asterinas on native QEMU/KVM   (kernel/Asterinas baseline).
    Env B: Linux on hvisor                (hvisor device/model baseline).
    Env C: Asterinas on hvisor            (the TARGET we are attributing).

    Code letters (unchanged meanings):
      [K] Asterinas/kernel Linux-ABI gap.
      [D] hvisor device/model limitation.
      [C] target-only failure (zone config / K+D interaction).

    Attribution is target-relative: the target is C. If C passes, the
    configuration under test works and there is no regression to attribute,
    regardless of how the baselines behaved -- so the verdict is OK. Only a
    target FAIL is attributed, using the two baselines to localize the cause:

      C    A     B     -> tag   rationale
      PASS *     *     -> OK    target works; nothing to attribute
      FAIL FAIL  *     -> K     Asterinas itself fails the case (kernel gap)
      FAIL PASS  FAIL  -> D     only the hvisor path regresses Linux too
      FAIL PASS  PASS  -> C     both baselines pass; target-only (config)

    (MISSING / SKIP on the target or a baseline yield UNVERIFIED first.)
    """
    a = normalize_status(env_a)
    b = normalize_status(env_b)
    c = normalize_status(env_c)

    if MISSING in {a, b, c}:
        return "UNVERIFIED", "missing one or more environment results"

    if c == SKIP:
        return "UNVERIFIED", "target environment skipped the test"
    if a == SKIP or b == SKIP:
        return "UNVERIFIED", "baseline skipped the test"

    # The target passed: the configuration under test works, so there is no
    # regression to attribute even if a baseline happened to fail.
    if is_pass(c):
        return "OK", "no compatibility regression observed"

    if is_fail(c):
        if is_fail(a):
            return "K", "Asterinas baseline also fails; likely kernel ABI gap"
        # a == PASS from here (a is PASS or FAIL only).
        if is_fail(b):
            return "D", "Linux-on-hvisor also fails; likely hvisor device/model limit"
        # a == PASS and b == PASS: both baselines pass, only the target fails.
        return "C", "target-only failure; inspect zone config first, then K+D interaction"

    return "UNVERIFIED", "status combination needs manual diagnosis"


def delta(target, baseline):
    target = normalize_status(target)
    baseline = normalize_status(baseline)
    if target == MISSING or baseline == MISSING:
        return "unknown"
    if target == baseline:
        return "same"
    return f"{baseline}->{target}"


def classify_vs_linux(asterinas, linux):
    """Attribution for an Asterinas-vs-Linux comparison on the same QEMU/KVM
    host. Linux/QEMU is the reference; any test where Linux passes but Asterinas
    does not is an Asterinas Linux-ABI gap ([K]).

    Returns (tag, explanation).
    """
    a = normalize_status(asterinas)
    l = normalize_status(linux)

    if l == MISSING or a == MISSING:
        return "UNVERIFIED", "missing Asterinas or Linux result"
    if l == SKIP:
        return "UNVERIFIED", "reference Linux skipped this case (environmental)"
    if l == FAIL:
        # The reference should pass; if it does not, the case itself or the
        # reference environment is suspect rather than Asterinas.
        return "REF", "Linux reference failed; case/reference needs inspection"
    # From here Linux PASSed.
    if a == PASS:
        return "OK", "Asterinas matches the Linux ABI on this case"
    if a == SKIP:
        return "UNVERIFIED", "Asterinas skipped a case Linux can run"
    if a == FAIL:
        return "K", "Asterinas Linux-ABI gap: Linux passes, Asterinas fails"
    return "UNVERIFIED", "status combination needs manual diagnosis"
