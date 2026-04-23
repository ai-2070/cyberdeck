#!/usr/bin/env python3
"""Parse criterion benchmark output and produce structured markdown.

Usage:
    parse_criterion.py <input.md> [<output.md>]

If <output.md> is omitted, prints to stdout.

Strategy for extracting the clean benchmark name:
  - Criterion progress lines have the form
      'Benchmarking NAME: <status>Benchmarking NAME: <status>...NAME'
    so we match `Benchmarking (\\S+?):` — the portion before the first colon
    is always the full, clean benchmark name.
  - Some benchmarks have a clean name on its own line like
    `cortex_query/tasks_find_many/100` — we match those directly.
  - Criterion's condensed format puts name and time on the same line:
      `bench_name/sub       time:   [low mid high]`
  - We ignore `change:` blocks (criterion's perf-comparison output).
  - Suite headers come from `Running benches/<name>.rs` (Unix) or
    `Running benches\\<name>.rs` (Windows).
  - When the same benchmark name appears more than once (e.g., two runs in
    the same file), the later result overwrites the earlier one.
"""

import re
import sys
from collections import OrderedDict
from pathlib import Path

TIME_RE = re.compile(r"time:\s*\[([^\]]+)\]")
THRPT_RE = re.compile(r"thrpt:\s*\[([^\]]+)\]")
RUNNING_RE = re.compile(r"Running benches[/\\]([A-Za-z0-9_]+)\.rs")
BENCHMARKING_RE = re.compile(r"Benchmarking\s+([^\s:]+):")
PLAIN_NAME_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*(?:/[A-Za-z0-9_]+)+)\s*$")
NAME_TIME_RE = re.compile(
    r"^\s*([A-Za-z_][A-Za-z0-9_]*(?:/[A-Za-z0-9_]+)*)\s+time:\s*\[([^\]]+)\]"
)


def median_of_triplet(s: str) -> str:
    """Return the middle value+unit from 'low unit mid unit high unit'."""
    s = s.strip()
    toks = s.split()
    if len(toks) == 6:
        return f"{toks[2]} {toks[3]}"
    m = re.findall(r"(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\s*([A-Za-zµ][A-Za-z/]*)", s)
    if len(m) >= 3:
        return f"{m[1][0]} {m[1][1]}"
    return s


def parse_file(path: Path):
    lines = path.read_text(errors="replace").splitlines()

    suites: "OrderedDict[str, OrderedDict[str, tuple[str, str]]]" = OrderedDict()
    current_suite = None
    last_name = None
    pending_time = None
    in_change = False

    def commit():
        nonlocal pending_time
        if pending_time is not None and last_name is not None and current_suite:
            suites.setdefault(current_suite, OrderedDict())
            suites[current_suite][last_name] = (pending_time, "")
        pending_time = None

    for raw in lines:
        line = raw.rstrip("\n")

        m_run = RUNNING_RE.search(line)
        if m_run:
            commit()
            current_suite = m_run.group(1)
            last_name = None
            in_change = False
            continue

        if "change:" in line:
            in_change = True
            continue
        if re.search(
            r"(Change within noise threshold\.|Performance has regressed\.|"
            r"No change in performance detected\.|Performance has improved\.)",
            line,
        ):
            in_change = False
            continue
        if line.startswith("Found ") and "outliers" in line:
            commit()
            in_change = False
            continue
        if not line.strip():
            commit()
            in_change = False
            continue

        m_bench = BENCHMARKING_RE.search(line)
        if m_bench:
            commit()
            last_name = m_bench.group(1)
            in_change = False
            continue

        m_name_time = NAME_TIME_RE.match(line)
        if m_name_time and not in_change:
            commit()
            last_name = m_name_time.group(1)
            pending_time = median_of_triplet(m_name_time.group(2))
            continue

        m_time = TIME_RE.search(line)
        if m_time and not in_change:
            commit()
            pending_time = median_of_triplet(m_time.group(1))
            continue

        m_thrpt = THRPT_RE.search(line)
        if m_thrpt and not in_change:
            if pending_time is not None and last_name is not None and current_suite:
                suites.setdefault(current_suite, OrderedDict())
                suites[current_suite][last_name] = (
                    pending_time,
                    median_of_triplet(m_thrpt.group(1)),
                )
                pending_time = None
            continue

        m_plain = PLAIN_NAME_RE.match(line)
        if m_plain:
            commit()
            last_name = m_plain.group(1)
            in_change = False
            continue

    commit()
    return suites


def format_md(suites, src_name: str):
    out = [f"# Benchmark Results (parsed from {src_name})\n"]
    for suite, benches in suites.items():
        out.append(f"## {suite}\n")
        out.append("| Benchmark | Time (median) | Throughput (median) |")
        out.append("|---|---|---|")
        for name, (t, thr) in benches.items():
            out.append(f"| {name} | {t} | {thr} |")
        out.append("")
    return "\n".join(out) + "\n"


def main(argv):
    if len(argv) < 2 or argv[1] in ("-h", "--help"):
        print(__doc__)
        return 1 if len(argv) < 2 else 0

    src = Path(argv[1])
    if not src.exists():
        print(f"error: input file not found: {src}", file=sys.stderr)
        return 2

    suites = parse_file(src)
    md = format_md(suites, src.name)

    if len(argv) >= 3:
        Path(argv[2]).write_text(md)
        total = sum(len(v) for v in suites.values())
        print(f"Parsed {len(suites)} suites, {total} benchmarks total", file=sys.stderr)
        for s, b in suites.items():
            print(f"  {s}: {len(b)} benchmarks", file=sys.stderr)
    else:
        sys.stdout.write(md)

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
