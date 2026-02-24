#!/usr/bin/env python3
"""
Parse MDCS benchmark log output into a Markdown table matching
the dmonad/crdt-benchmarks format.

Usage:
    python benchmarks/parse_log.py logs/benchmark_results.txt
    python benchmarks/parse_log.py logs/benchmark_results.txt -o benchmarks/RESULTS.md
"""

import re
import sys
import argparse
from collections import OrderedDict

# Pattern: [Test Name] (metric)    value
LINE_RE = re.compile(
    r'^\[(.+?)\]\s+\((\w[\w:]*)\)\s+(.+)$'
)

def parse_log(path: str) -> OrderedDict:
    """Parse log file into {test_name: {metric: value}}."""
    tests = OrderedDict()
    with open(path) as f:
        for line in f:
            line = line.strip()
            m = LINE_RE.match(line)
            if not m:
                continue
            name, metric, value = m.group(1), m.group(2), m.group(3).strip()
            if name not in tests:
                tests[name] = OrderedDict()
            tests[name][metric] = value
    return tests


def group_tests(tests: OrderedDict) -> list:
    """Group tests by B1/B2/B3 prefix."""
    groups = OrderedDict()
    for name in tests:
        # Extract group prefix like "B1", "B2", "B3"
        prefix_match = re.match(r'(B\d+)', name)
        prefix = prefix_match.group(1) if prefix_match else "Other"
        if prefix not in groups:
            groups[prefix] = []
        groups[prefix].append(name)
    return groups


def format_table(tests: OrderedDict, group_name: str, test_names: list) -> str:
    """Format a group of tests into a markdown table."""
    # Collect all metrics across these tests
    all_metrics = []
    for name in test_names:
        for metric in tests[name]:
            if metric not in all_metrics:
                all_metrics.append(metric)

    lines = []
    lines.append(f"### {group_name}\n")

    # Header row: | Benchmark | metric1 | metric2 | ... |
    header = "| Benchmark |"
    sep = "| :--- |"
    for metric in all_metrics:
        header += f" {metric} |"
        sep += " ---: |"

    lines.append(header)
    lines.append(sep)

    # Data rows
    for name in test_names:
        # Shorten name: "B1.1 Append N characters" -> just the name
        row = f"| {name} |"
        for metric in all_metrics:
            val = tests[name].get(metric, "—")
            row += f" {val} |"
        lines.append(row)

    lines.append("")
    return "\n".join(lines)


def build_markdown(tests: OrderedDict) -> str:
    """Build full markdown output."""
    parts = []
    parts.append("# MDCS Benchmark Results\n")
    parts.append("> N = 6000 (default). Generated from raw benchmark log.\n")
    parts.append("Comparable to [dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks) output.\n")

    groups = group_tests(tests)
    for group_name, test_names in groups.items():
        parts.append(format_table(tests, group_name, test_names))

    return "\n".join(parts)


def main():
    parser = argparse.ArgumentParser(description="Parse MDCS benchmark logs to Markdown table")
    parser.add_argument("logfile", help="Path to the benchmark log file")
    parser.add_argument("-o", "--output", help="Output markdown file (default: stdout)")
    args = parser.parse_args()

    tests = parse_log(args.logfile)

    if not tests:
        print(f"No benchmark results found in {args.logfile}", file=sys.stderr)
        sys.exit(1)

    md = build_markdown(tests)

    if args.output:
        with open(args.output, "w") as f:
            f.write(md)
        print(f"Written to {args.output}")
    else:
        print(md)


if __name__ == "__main__":
    main()
