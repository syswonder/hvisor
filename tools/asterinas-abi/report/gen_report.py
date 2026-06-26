#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Generate Markdown, CSV, and HTML reports from diff_matrix.json.

The report is fully data-driven: every environment's identity, platform string,
and measurement status is read from the per-environment metadata that
``diff/syscall_diff.py`` embeds in ``diff_matrix.json`` (``metadata.env_a``,
``metadata.env_b``, ``metadata.env_c``, ``metadata.ref_linux``). Nothing about
the measurement environment is hardcoded here; an environment marked
``measured`` renders with its real platform string, and one marked
``not-measured`` (or absent) renders as unavailable.

The report has two parts, each emitted only when the underlying data is present:

  1. The Asterinas-vs-reference-Linux Linux-ABI comparison, present only when
     the diff carries ``ref_linux`` data.

  2. The Env A/B/C hvisor matrix, with each environment's status taken from the
     data rather than assumed.
"""

from __future__ import annotations

import argparse
import csv
import html
import json
import sys
from collections import defaultdict
from pathlib import Path


LAYER_ORDER = ["L1", "L2", "L3", "L4", "L5", "App"]

# Map the metadata key for each environment to the per-row status field and a
# stable display ID. Order is the order environments appear in the env table.
ENV_SPECS = [
    ("env_a", "env_a", "A"),
    ("ref_linux", "ref_linux", "Ref-Linux"),
    ("env_b", "env_b", "B"),
    ("env_c", "env_c", "C"),
]


def md_escape(value) -> str:
    text = str(value)
    return text.replace("|", "\\|").replace("\n", " ")


def per_layer_counts(rows, status_key):
    agg = defaultdict(lambda: defaultdict(int))
    for r in rows:
        agg[r["layer"]][r.get(status_key, "MISSING")] += 1
    return agg


def env_display_id(meta: dict, fallback: str) -> str:
    """Friendly environment ID, derived from metadata when available."""
    for key in ("env_id", "environment"):
        val = meta.get(key)
        if val and str(val).lower() != "unknown":
            return str(val)
    return fallback


def env_kernel(meta: dict) -> str:
    """Best available human description of the kernel/role under test.

    There is no dedicated 'kernel' field in the result metadata, so we use the
    operator-supplied ``label`` when present, otherwise fall back to the
    platform string, otherwise leave it unspecified."""
    label = meta.get("label")
    if label:
        return str(label)
    platform = meta.get("platform")
    if platform:
        return str(platform)
    return "(unspecified)"


def env_platform(meta: dict) -> str:
    """Real platform string from the data, with sensible fallbacks."""
    platform = meta.get("platform")
    if platform:
        return str(platform)
    label = meta.get("label")
    if label:
        return str(label)
    return "(unspecified)"


def _has_real_results(counts: dict | None) -> bool:
    """True iff the status tally contains any non-MISSING outcome."""
    if not counts:
        return False
    return any(k != "MISSING" and v for k, v in counts.items())


def is_measured(meta: dict, counts: dict | None) -> bool:
    """Authoritative 'was this environment measured?' decision, from the data.

    Honours an explicit ``measurement_status`` when present; otherwise falls
    back to whether the diff produced any non-MISSING results for it."""
    status = str(meta.get("measurement_status", "")).strip().lower()
    if status:
        return status == "measured"
    return _has_real_results(counts)


def env_status_text(meta: dict, counts: dict | None) -> str:
    """Render the measurement status from the data.

    ``measured`` environments additionally show their live PASS/FAIL/SKIP tally
    so the table reflects the actual measurement, never a hardcoded verdict."""
    status = str(meta.get("measurement_status", "")).strip()
    if not status:
        # No metadata at all: infer from whether any results were produced.
        status = "measured" if _has_real_results(counts) else "not-measured"
    if is_measured(meta, counts) and counts:
        passed = counts.get("PASS", 0)
        failed = counts.get("FAIL", 0)
        skipped = counts.get("SKIP", 0)
        return f"measured ({passed} PASS / {failed} FAIL / {skipped} SKIP)"
    return status


def present_envs(meta: dict, summary: dict) -> list[tuple[str, dict, dict | None, str]]:
    """Return (key, env_meta, status_counts, display_id) for environments that
    have either metadata or per-row status counts in the diff."""
    envs = []
    for meta_key, status_key, fallback_id in ENV_SPECS:
        env_meta = meta.get(meta_key)
        counts = summary.get(status_key)
        if env_meta is None and counts is None:
            continue
        env_meta = env_meta or {}
        envs.append((meta_key, env_meta, counts, env_display_id(env_meta, fallback_id)))
    return envs


def write_markdown(matrix: dict, path: Path) -> None:
    rows = matrix.get("rows", [])
    summary = matrix.get("summary", {})
    meta = matrix.get("metadata", {})
    has_ref = "ref_linux" in meta or any("ref_linux" in r for r in rows)
    av = matrix.get("aster_vs_linux_summary", {})
    envs = present_envs(meta, summary)

    with path.open("w", encoding="utf-8") as f:
        f.write("# Asterinas Linux-ABI Differential Compatibility Report\n\n")
        f.write("Generated by `report/gen_report.py` from collected JSON results.\n\n")

        # Environment legend, rendered entirely from per-env metadata.
        f.write("## Environments\n\n")
        f.write("| ID | Kernel / Role | Platform | Status |\n")
        f.write("|----|---------------|----------|--------|\n")
        for _key, env_meta, counts, disp_id in envs:
            f.write(
                "| {id} | {kernel} | {platform} | {status} |\n".format(
                    id=md_escape(disp_id),
                    kernel=md_escape(env_kernel(env_meta)),
                    platform=md_escape(env_platform(env_meta)),
                    status=md_escape(env_status_text(env_meta, counts)),
                )
            )
        f.write("\n")

        if has_ref:
            ref_meta = meta.get("ref_linux", {})
            ref_id = env_display_id(ref_meta, "Ref-Linux")
            f.write(f"## Measured result: Asterinas vs {md_escape(ref_id)}\n\n")
            total = len(rows)
            okc = av.get("OK", 0)
            kc = av.get("K", 0)
            unv = av.get("UNVERIFIED", 0)
            refc = av.get("REF", 0)
            runnable = okc + kc
            rate = (100.0 * okc / runnable) if runnable else 0.0
            f.write(f"- Total cases: {total}\n")
            f.write(f"- Comparable (reference passes, Asterinas runs): {runnable}\n")
            f.write(f"- **Asterinas matches the reference (OK): {okc}/{runnable} = {rate:.1f}%**\n")
            f.write(f"- Asterinas Linux-ABI gaps [K] (reference passes, Asterinas fails): {kc}\n")
            if refc:
                f.write(f"- Reference anomalies [REF] (reference itself failed): {refc}\n")
            f.write(f"- Not comparable (skipped on one side / environmental): {unv}\n\n")

            # Per-layer matched rate
            f.write("### Per-layer compatibility (Asterinas matched / comparable)\n\n")
            f.write("| Layer | Matched | Comparable | Rate |\n|---|---:|---:|---:|\n")
            ag = defaultdict(lambda: [0, 0])
            for r in rows:
                tag = r.get("aster_vs_linux")
                if tag == "OK":
                    ag[r["layer"]][0] += 1; ag[r["layer"]][1] += 1
                elif tag == "K":
                    ag[r["layer"]][1] += 1
            for layer in LAYER_ORDER:
                if layer in ag:
                    m, c = ag[layer]
                    pr = (100.0 * m / c) if c else 0.0
                    f.write(f"| {layer} | {m} | {c} | {pr:.0f}% |\n")
            f.write("\n")

            # Gap list
            gaps = [r for r in rows if r.get("aster_vs_linux") == "K"]
            f.write("### Asterinas Linux-ABI gaps (measured)\n\n")
            if gaps:
                f.write("| Test | Layer | Reference | Asterinas | Evidence |\n|---|---|---|---|---|\n")
                for r in gaps:
                    out = md_escape(r.get("outputs", {}).get("A", ""))[:120]
                    f.write(f"| {md_escape(r['name'])} | {r['layer']} | {r.get('ref_linux')} | {r['env_a']} | {out} |\n")
            else:
                f.write("None: Asterinas matched the reference Linux ABI on every comparable case.\n")
            f.write("\n")

            f.write("### Full Asterinas-vs-reference matrix\n\n")
            f.write("| Test Case | L | Reference | Asterinas | Delta | Attribution | Reason |\n")
            f.write("|---|---:|---|---|---|---|---|\n")
            for row in rows:
                f.write(
                    "| {name} | {layer} | {lin} | {a} | {d} | {attr} | {reason} |\n".format(
                        name=md_escape(row["name"]), layer=md_escape(row["layer"]),
                        lin=md_escape(row.get("ref_linux", "MISSING")),
                        a=md_escape(row["env_a"]),
                        d=md_escape(row.get("delta_a_linux", "")),
                        attr=md_escape(row.get("aster_vs_linux", "")),
                        reason=md_escape(row.get("aster_vs_linux_reason", "")),
                    )
                )
            f.write("\n")

        # A/B/C hvisor matrix, with the headline computed from the data.
        f.write("## Three-environment hvisor matrix (A/B/C)\n\n")
        f.write(env_matrix_summary_md(meta, summary, rows))
        f.write("\nAttribution legend: **[K]** Asterinas kernel - **[D]** hvisor device model - **[C]** zone config.\n\n")
        f.write("| Test Case | L | Env A | Env B | Env C | Delta C-A | Delta C-B | Attribution | Reason |\n")
        f.write("|---|---:|---|---|---|---|---|---|---|\n")
        for row in rows:
            f.write(
                "| {name} | {layer} | {a} | {b} | {c} | {dca} | {dcb} | {attr} | {reason} |\n".format(
                    name=md_escape(row["name"]), layer=md_escape(row["layer"]),
                    a=md_escape(row["env_a"]), b=md_escape(row["env_b"]), c=md_escape(row["env_c"]),
                    dca=md_escape(row["delta_c_a"]), dcb=md_escape(row["delta_c_b"]),
                    attr=md_escape(row["attribution"]), reason=md_escape(row["reason"]),
                )
            )
        f.write("\n## Failure Diagnosis Template\n\n")
        f.write("For every FAIL row: symptom -> Env comparison -> ")
        f.write("memory_regions/cmdline/e820/procfs/dmesg checks -> conclusion -> fix status.\n")


def _status_counts(summary: dict, key: str) -> dict:
    counts = summary.get(key)
    return counts if isinstance(counts, dict) else {}


def env_matrix_summary_md(meta: dict, summary: dict, rows: list) -> str:
    """Compose the A/B/C headline purely from the measured data."""
    lines = []
    a_meta = meta.get("env_a", {})
    b_meta = meta.get("env_b", {})
    c_meta = meta.get("env_c", {})
    a_counts = _status_counts(summary, "env_a")
    c_counts = _status_counts(summary, "env_c")

    # Per-env one-liners derived from metadata + live counts.
    for disp, m, counts in (
        (env_display_id(a_meta, "A"), a_meta, a_counts),
        (env_display_id(b_meta, "B"), b_meta, _status_counts(summary, "env_b")),
        (env_display_id(c_meta, "C"), c_meta, c_counts),
    ):
        lines.append(
            f"- **Env {disp}** ({env_platform(m)}): {env_status_text(m, counts)}."
        )

    # The C-vs-A delta, computed from the rows.
    diffs = [r for r in rows if r.get("env_c") != r.get("env_a")]
    c_measured = is_measured(c_meta, c_counts)
    a_measured = is_measured(a_meta, a_counts)
    if c_measured and a_measured:
        if not diffs:
            lines.append(
                "- **Delta C-A = 0**: every case has the same status under hvisor "
                "zone0 (Env C) as on bare QEMU (Env A); the hvisor path introduces "
                "no observed ABI regression."
            )
        else:
            names = ", ".join(md_escape(r["name"]) for r in diffs)
            lines.append(
                f"- **Delta C-A != 0**: {len(diffs)} case(s) differ between Env C "
                f"and Env A: {names}."
            )
    else:
        not_done = [d for d, done in (("A", a_measured), ("C", c_measured)) if not done]
        lines.append(
            "- Delta C-A not computable: Env "
            + " and Env ".join(not_done)
            + " "
            + ("have" if len(not_done) > 1 else "has")
            + " no collected data."
        )
    return "\n".join(lines) + "\n"


def write_csv(matrix: dict, path: Path) -> None:
    rows = matrix.get("rows", [])
    has_ref = any("ref_linux" in r for r in rows)
    fields = ["name", "layer", "description", "env_a", "env_b", "env_c",
              "delta_c_a", "delta_c_b", "attribution", "reason"]
    if has_ref:
        fields += ["ref_linux", "delta_a_linux", "aster_vs_linux", "aster_vs_linux_reason"]
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for row in rows:
            writer.writerow({field: row.get(field, "") for field in fields})


def write_html(matrix: dict, path: Path) -> None:
    rows = matrix.get("rows", [])
    summary = matrix.get("summary", {})
    meta = matrix.get("metadata", {})
    av = matrix.get("aster_vs_linux_summary", {})
    has_ref = any("ref_linux" in r for r in rows)
    envs = present_envs(meta, summary)
    style = """
body{font-family:system-ui,-apple-system,Segoe UI,sans-serif;margin:24px;color:#202124}
table{border-collapse:collapse;width:100%;font-size:13px;margin-bottom:24px}
th,td{border:1px solid #d0d7de;padding:6px 8px;text-align:left;vertical-align:top}
th{background:#f6f8fa;position:sticky;top:0}
.PASS,.OK{color:#116329;font-weight:600}
.FAIL,.C,.D,.K{color:#cf222e;font-weight:600}
.SKIP,.MISSING,.UNVERIFIED,.REF{color:#8250df;font-weight:600}
.kpi{display:inline-block;margin:4px 16px 12px 0;font-size:15px}
.kpi b{font-size:22px}
h2{border-bottom:2px solid #d0d7de;padding-bottom:4px;margin-top:32px}
"""
    with path.open("w", encoding="utf-8") as f:
        f.write("<!doctype html><html><head><meta charset='utf-8'>")
        f.write("<title>Asterinas Linux-ABI Differential Report</title>")
        f.write(f"<style>{style}</style></head><body>")
        f.write("<h1>Asterinas Linux-ABI Differential Compatibility Report</h1>")

        # Data-driven environment legend.
        f.write("<h2>Environments</h2>")
        f.write("<table><thead><tr>")
        for h in ["ID", "Kernel / Role", "Platform", "Status"]:
            f.write(f"<th>{html.escape(h)}</th>")
        f.write("</tr></thead><tbody>")
        for _key, env_meta, counts, disp_id in envs:
            status = env_status_text(env_meta, counts)
            status_cls = "measured" if is_measured(env_meta, counts) else "UNVERIFIED"
            f.write("<tr>")
            f.write(f"<td>{html.escape(str(disp_id))}</td>")
            f.write(f"<td>{html.escape(env_kernel(env_meta))}</td>")
            f.write(f"<td>{html.escape(env_platform(env_meta))}</td>")
            f.write(f"<td class='{html.escape(status_cls)}'>{html.escape(status)}</td>")
            f.write("</tr>")
        f.write("</tbody></table>")

        if has_ref:
            ref_id = env_display_id(meta.get("ref_linux", {}), "Ref-Linux")
            okc = av.get("OK", 0); kc = av.get("K", 0)
            runnable = okc + kc
            rate = (100.0 * okc / runnable) if runnable else 0.0
            f.write(f"<h2>Measured: Asterinas vs {html.escape(str(ref_id))}</h2>")
            f.write(f"<div class='kpi'>Compatibility<br><b class='OK'>{rate:.1f}%</b></div>")
            f.write(f"<div class='kpi'>Matched (OK)<br><b>{okc}/{runnable}</b></div>")
            f.write(f"<div class='kpi'>ABI gaps [K]<br><b class='K'>{kc}</b></div>")
            f.write("<table><thead><tr>")
            for h in ["Test Case", "L", "Reference", "Asterinas", "Delta", "Attribution", "Reason"]:
                f.write(f"<th>{html.escape(h)}</th>")
            f.write("</tr></thead><tbody>")
            for row in rows:
                f.write("<tr>")
                for value, cls in [
                    (row["name"], ""), (row["layer"], ""),
                    (row.get("ref_linux", "MISSING"), row.get("ref_linux", "MISSING")),
                    (row["env_a"], row["env_a"]),
                    (row.get("delta_a_linux", ""), ""),
                    (row.get("aster_vs_linux", ""), row.get("aster_vs_linux", "")),
                    (row.get("aster_vs_linux_reason", ""), ""),
                ]:
                    f.write(f"<td class='{html.escape(str(cls))}'>{html.escape(str(value))}</td>")
                f.write("</tr>")
            f.write("</tbody></table>")

        f.write("<h2>Three-environment hvisor matrix (A/B/C)</h2>")
        f.write("<ul>")
        for line in env_matrix_summary_md(meta, summary, rows).strip().splitlines():
            text = line.lstrip("- ").replace("**", "")
            f.write(f"<li>{html.escape(text)}</li>")
        f.write("</ul>")
        f.write("<table><thead><tr>")
        for h in ["Test Case", "L", "Env A", "Env B", "Env C", "Delta C-A", "Delta C-B", "Attribution", "Reason"]:
            f.write(f"<th>{html.escape(h)}</th>")
        f.write("</tr></thead><tbody>")
        for row in rows:
            f.write("<tr>")
            for value, cls in [
                (row["name"], ""), (row["layer"], ""),
                (row["env_a"], row["env_a"]), (row["env_b"], row["env_b"]), (row["env_c"], row["env_c"]),
                (row["delta_c_a"], ""), (row["delta_c_b"], ""),
                (row["attribution"], row["attribution"]), (row["reason"], ""),
            ]:
                f.write(f"<td class='{html.escape(str(cls))}'>{html.escape(str(value))}</td>")
            f.write("</tr>")
        f.write("</tbody></table></body></html>")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--diff", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    with args.diff.open("r", encoding="utf-8") as f:
        matrix = json.load(f)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    write_markdown(matrix, args.out_dir / "compatibility_report.md")
    write_csv(matrix, args.out_dir / "compatibility_matrix.csv")
    write_html(matrix, args.out_dir / "compatibility_report.html")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
