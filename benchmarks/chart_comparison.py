#!/usr/bin/env python3
"""
Generate comparison charts from MDCS benchmark raw logs vs competitor CRDTs.

Reads the raw benchmark log (logs/benchmark_results.txt by default),
pairs MDCS results against competitor data from dmonad/crdt-benchmarks,
and produces grouped bar charts saved to benchmarks/charts/.

Usage:
    uv pip install matplotlib numpy
    uv run python chart_comparison.py
    uv run python chart_comparison.py --logfile logs/benchmark_results_raw.txt
    uv run python chart_comparison.py --outdir my_charts/

Dependencies: matplotlib, numpy
"""

import re
import os
import argparse
from collections import OrderedDict
from pathlib import Path
import matplotlib
matplotlib.use('Agg') 
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np

# ─── Raw log parser ───────────────────────────────────────────────────────────

LINE_RE = re.compile(r"^\[(.+?)\]\s+\((\w[\w:]*)\)\s+(.+)$")


def parse_log(path: str) -> OrderedDict:
    """Parse MDCS benchmark log into {test_name: {metric: raw_string}}."""
    tests: OrderedDict[str, OrderedDict[str, str]] = OrderedDict()
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


def parse_value_to_bytes(raw: str) -> float | None:
    """Convert byte-unit strings like '6002 bytes', '2.8 MB', '17.4 kB' to bytes."""
    raw = raw.strip().replace(",", "")
    if raw in ("—", "n/a", "N/A") or "n/a" in raw:
        return None
    raw = raw.rstrip("†").strip()
    m = re.match(r"^([\d.]+)\s*(bytes?|B|kB|KB|MB|GB)$", raw, re.IGNORECASE)
    if not m:
        return None
    val = float(m.group(1))
    unit = m.group(2).upper()
    multipliers = {"BYTES": 1, "BYTE": 1, "B": 1, "KB": 1024, "MB": 1024**2, "GB": 1024**3}
    return val * multipliers.get(unit, 1)


def parse_value_to_ms(raw: str) -> float | None:
    """Convert time strings like '11531 ms', '97 µs', '< 1 ms' to milliseconds."""
    raw = raw.strip().replace(",", "")
    if raw in ("—", "n/a", "N/A"):
        return None
    raw = raw.lstrip("*").rstrip("*").strip()
    if raw.startswith("< "):
        # "< 1 ms" → treat as 0.5 ms
        rest = raw[2:]
        m = re.match(r"^([\d.]+)\s*(ms|µs|us|s)$", rest)
        if m:
            val = float(m.group(1)) * 0.5
            unit = m.group(2)
        else:
            return None
    else:
        m = re.match(r"^([\d.]+)\s*(ms|µs|us|s)$", raw)
        if not m:
            return None
        val = float(m.group(1))
        unit = m.group(2)
    if unit == "s":
        return val * 1000.0
    if unit == "ms":
        return val
    if unit in ("µs", "us"):
        return val / 1000.0
    return val


def parse_competitor_value(raw: str) -> float | None:
    """Parse a value from the RESULTS.md competitor table cell."""
    raw = raw.strip().replace(",", "")
    raw = raw.lstrip("*").rstrip("*").strip()  # strip markdown bold
    raw = raw.rstrip("†").strip()
    if raw in ("—", "0 B", "N/A (native)"):
        return None
    return raw  # return string for caller to dispatch


# ─── Competitor data (from RESULTS.md comparison table, N = 6000) ─────────────

COMPETITORS = ["yjs", "ywasm", "loro", "automerge", "mdcs-sdk"]

# Structured data: {benchmark_label: {metric: {lib: value}}}
# Time values in ms, size values in bytes

COMPARISON_DATA: dict[str, dict[str, dict[str, float | None]]] = {
    # ── B1 ──
    "B1.1 Append N chars": {
        "time": {"yjs": 188, "ywasm": 154, "loro": 120, "automerge": 365},
        "avgUpdateSize": {"yjs": 27, "ywasm": 27, "loro": 109, "automerge": 121},
        "docSize": {"yjs": 6031, "ywasm": 6031, "loro": 6162, "automerge": 3992},
        "parseTime": {"yjs": 32, "ywasm": 23, "loro": 26, "automerge": 80},
    },
    "B1.2 Insert string of N": {
        "time": {"yjs": 0.5, "ywasm": 0.5, "loro": 0.5, "automerge": 9},
        "avgUpdateSize": {"yjs": 6031, "ywasm": 6031, "loro": 6107, "automerge": 6201},
        "docSize": {"yjs": 6031, "ywasm": 6031, "loro": 6117, "automerge": 3974},
        "parseTime": {"yjs": 27, "ywasm": 34, "loro": 29, "automerge": 47},
    },
    "B1.3 Prepend N chars": {
        "time": {"yjs": 119, "ywasm": 23, "loro": 81, "automerge": 307},
        "avgUpdateSize": {"yjs": 27, "ywasm": 27, "loro": 108, "automerge": 116},
        "docSize": {"yjs": 6041, "ywasm": 6041, "loro": 12125, "automerge": 3988},
        "parseTime": {"yjs": 93, "ywasm": 31, "loro": 26, "automerge": 63},
    },
    "B1.4 Insert N chars random": {
        "time": {"yjs": 131, "ywasm": 128, "loro": 79, "automerge": 310},
        "avgUpdateSize": {"yjs": 29, "ywasm": 29, "loro": 109, "automerge": 121},
        "docSize": {"yjs": 29554, "ywasm": 29554, "loro": 35401, "automerge": 24743},
        "parseTime": {"yjs": 76, "ywasm": 29, "loro": 31, "automerge": 79},
    },
    "B1.5 Insert N words random": {
        "time": {"yjs": 154, "ywasm": 449, "loro": 82, "automerge": 449},
        "avgUpdateSize": {"yjs": 36, "ywasm": 36, "loro": 117, "automerge": 131},
        "docSize": {"yjs": 87924, "ywasm": 87924, "loro": 94524, "automerge": 96203},
        "parseTime": {"yjs": 92, "ywasm": 34, "loro": 31, "automerge": 143},
    },
    "B1.6 Insert then delete": {
        "time": {"yjs": 1, "ywasm": 1, "loro": 2, "automerge": 22},
        "avgUpdateSize": {"yjs": 6053, "ywasm": 6053, "loro": 6217, "automerge": 6338},
        "docSize": {"yjs": 38, "ywasm": 38, "loro": 6120, "automerge": 3993},
        "parseTime": {"yjs": 44, "ywasm": 28, "loro": 27, "automerge": 37},
    },
    "B1.7 Insert/Delete random": {
        "time": {"yjs": 158, "ywasm": 141, "loro": 98, "automerge": 389},
        "avgUpdateSize": {"yjs": 31, "ywasm": 31, "loro": 113, "automerge": 135},
        "docSize": {"yjs": 28377, "ywasm": 28377, "loro": 50836, "automerge": 59281},
        "parseTime": {"yjs": 117, "ywasm": 31, "loro": 25, "automerge": 111},
    },
    "B1.8 Append N numbers": {
        "time": {"yjs": 148, "ywasm": 29, "loro": 81, "automerge": 480},
        "avgUpdateSize": {"yjs": 32, "ywasm": 32, "loro": 114, "automerge": 125},
        "docSize": {"yjs": 35634, "ywasm": 35634, "loro": 35719, "automerge": 26985},
        "parseTime": {"yjs": 36, "ywasm": 31, "loro": 27, "automerge": 80},
    },
    "B1.9 Insert Array of N nums": {
        "time": {"yjs": 1, "ywasm": 2, "loro": 9, "automerge": 38},
        "docSize": {"yjs": 35657, "ywasm": 35657, "loro": 35742, "automerge": 26953},
        "parseTime": {"yjs": 33, "ywasm": 26, "loro": 22, "automerge": 53},
    },
    "B1.10 Prepend N numbers": {
        "time": {"yjs": 122, "ywasm": 28, "loro": 78, "automerge": 461},
        "avgUpdateSize": {"yjs": 32, "ywasm": 36, "loro": 113, "automerge": 120},
        "docSize": {"yjs": 35665, "ywasm": 65658, "loro": 41748, "automerge": 26987},
        "parseTime": {"yjs": 96, "ywasm": 31, "loro": 32, "automerge": 77},
    },
    "B1.11 Insert N nums random": {
        "time": {"yjs": 134, "ywasm": 144, "loro": 78, "automerge": 433},
        "avgUpdateSize": {"yjs": 33, "ywasm": 34, "loro": 114, "automerge": 125},
        "docSize": {"yjs": 59136, "ywasm": 59152, "loro": 65016, "automerge": 47746},
        "parseTime": {"yjs": 80, "ywasm": 34, "loro": 36, "automerge": 93},
    },
    # ── B2 ──
    "B2.1 Concurrent insert string": {
        "time": {"yjs": 1, "ywasm": 0.5, "loro": 2, "automerge": 62},
        "updateSize": {"yjs": 6094, "ywasm": 6094, "loro": 9276, "automerge": 9499},
        "docSize": {"yjs": 12152, "ywasm": 12151, "loro": 12248, "automerge": 8011},
        "parseTime": {"yjs": 43, "ywasm": 27, "loro": 25, "automerge": 47},
    },
    "B2.2 Concurrent insert N chars": {
        "time": {"yjs": 65, "ywasm": 365, "loro": 83, "automerge": 287},
        "updateSize": {"yjs": 33444, "ywasm": 177007, "loro": 35554, "automerge": 27476},
        "docSize": {"yjs": 66852, "ywasm": 66860, "loro": 71858, "automerge": 50683},
        "parseTime": {"yjs": 101, "ywasm": 34, "loro": 30, "automerge": 53},
    },
    "B2.3 Concurrent insert N words": {
        "time": {"yjs": 85, "ywasm": 1014, "loro": 112, "automerge": 663},
        "updateSize": {"yjs": 88994, "ywasm": 215213, "loro": 93132, "automerge": 122485},
        "docSize": {"yjs": 178137, "ywasm": 178130, "loro": 188458, "automerge": 185019},
        "parseTime": {"yjs": 85, "ywasm": 71, "loro": 52, "automerge": 168},
    },
    "B2.4 Concurrent insert & delete": {
        "time": {"yjs": 178, "ywasm": 2786, "loro": 208, "automerge": 1066},
        "updateSize": {"yjs": 139517, "ywasm": 398881, "loro": 163564, "automerge": 298810},
        "docSize": {"yjs": 279172, "ywasm": 279166, "loro": 289590, "automerge": 293828},
        "parseTime": {"yjs": 121, "ywasm": 78, "loro": 50, "automerge": 255},
    },
    # ── B3 ──
    "B3.1 Concurrent set number Map": {
        "time": {"yjs": 75, "ywasm": 290, "loro": 56, "automerge": 1632},
        "docSize": {"yjs": 32225, "ywasm": 32209, "loro": 21506, "automerge": 86167},
        "parseTime": {"yjs": 104, "ywasm": 70, "loro": 40, "automerge": 37},
    },
    "B3.2 Concurrent set Object Map": {
        "time": {"yjs": 84, "ywasm": 278, "loro": 67, "automerge": 1726},
        "docSize": {"yjs": 32235, "ywasm": 32249, "loro": 40494, "automerge": 112570},
        "parseTime": {"yjs": 102, "ywasm": 70, "loro": 45, "automerge": 86},
    },
    "B3.3 Concurrent set String Map": {
        "time": {"yjs": 86, "ywasm": 299, "loro": 116, "automerge": 2335},
        "docSize": {"yjs": 38357, "ywasm": 38376, "loro": 7798572, "automerge": 98047},
        "parseTime": {"yjs": 97, "ywasm": 52, "loro": 55, "automerge": 118},
    },
    "B3.4 Concurrent insert text Array": {
        "time": {"yjs": 72, "ywasm": 283, "loro": 227, "automerge": 2780},
        "docSize": {"yjs": 26583, "ywasm": 26596, "loro": 31119, "automerge": 96463},
        "parseTime": {"yjs": 84, "ywasm": 60, "loro": 29, "automerge": 42},
    },
}

# Map from COMPARISON_DATA short labels → raw log test names
LABEL_TO_LOG_NAME: dict[str, str] = {
    "B1.1 Append N chars": "B1.1 Append N characters",
    "B1.2 Insert string of N": "B1.2 Insert string of length N",
    "B1.3 Prepend N chars": "B1.3 Prepend N characters",
    "B1.4 Insert N chars random": "B1.4 Insert N characters at random positions",
    "B1.5 Insert N words random": "B1.5 Insert N words at random positions",
    "B1.6 Insert then delete": "B1.6 Insert string, then delete it",
    "B1.7 Insert/Delete random": "B1.7 Insert/Delete strings at random positions",
    "B1.8 Append N numbers": "B1.8 Append N numbers",
    "B1.9 Insert Array of N nums": "B1.9 Insert Array of N numbers",
    "B1.10 Prepend N numbers": "B1.10 Prepend N numbers",
    "B1.11 Insert N nums random": "B1.11 Insert N numbers at random positions",
    "B2.1 Concurrent insert string": "B2.1 Concurrently insert string of length N at index 0",
    "B2.2 Concurrent insert N chars": "B2.2 Concurrently insert N characters at random positions",
    "B2.3 Concurrent insert N words": "B2.3 Concurrently insert N words at random positions",
    "B2.4 Concurrent insert & delete": "B2.4 Concurrently insert & delete",
    "B3.1 Concurrent set number Map": "B3.1 20*sqrt(N) clients concurrently set number in Map",
    "B3.2 Concurrent set Object Map": "B3.2 20*sqrt(N) clients concurrently set Object in Map",
    "B3.3 Concurrent set String Map": "B3.3 20*sqrt(N) clients concurrently set String in Map",
    "B3.4 Concurrent insert text Array": "B3.4 20*sqrt(N) clients concurrently insert text in Array",
}

# Metric key mapping: comparison name → possible raw-log metric names
METRIC_LOG_KEYS: dict[str, list[str]] = {
    "time": ["time"],
    "avgUpdateSize": ["avgUpdateSize"],
    "updateSize": ["avgUpdateSize"],
    "docSize": ["docSize:json", "docSize:bincode"],  # prefer json, fallback bincode
    "parseTime": ["parseTime"],
    "encodeTime": ["encodeTime"],
    "memUsed": ["memUsed"],
}


def extract_mdcs_value(tests: OrderedDict, log_name: str, metric_key: str) -> float | None:
    """Pull the MDCS value from parsed logs for a given benchmark + metric."""
    if log_name not in tests:
        return None
    test_data = tests[log_name]
    candidates = METRIC_LOG_KEYS.get(metric_key, [metric_key])
    for cand in candidates:
        if cand in test_data:
            raw = test_data[cand]
            if "bytes" in raw.lower() or "mb" in raw.lower() or "kb" in raw.lower():
                return parse_value_to_bytes(raw)
            else:
                return parse_value_to_ms(raw)
    return None


# ─── Metric direction (lower vs higher is better) ─────────────────────────────

METRIC_DIRECTION: dict[str, str] = {
    "time": "lower is better",
    "avgUpdateSize": "lower is better",
    "updateSize": "lower is better",
    "docSize": "lower is better",
    "parseTime": "lower is better",
    "encodeTime": "lower is better",
    "memUsed": "lower is better",
}

# ─── Benchmark group descriptions (from RESULTS.md) ──────────────────────────

GROUP_DESCRIPTIONS: dict[str, str] = {
    "B1": (
        "B1: No conflicts — Simulate two clients. One client modifies a text "
        "object and sends update messages to the other client."
    ),
    "B2": (
        "B2: Two users producing conflicts — Simulate two clients. Both start "
        "with a synced text object containing 100 characters. Both clients modify "
        "the text object in a single transaction and then send their changes to "
        "the other client."
    ),
    "B3": (
        "B3: Many conflicts — Simulate \u221AN concurrent actions. \u221AN concurrent "
        "actions may result in up to \u221AN\u00B2\u22121 conflicts."
    ),
}

# Per-metric human-readable descriptions
METRIC_DESCRIPTIONS: dict[str, str] = {
    "time": "Wall-clock time to perform the task",
    "avgUpdateSize": "Average size of data exchanged per update",
    "updateSize": "Total size of update messages exchanged",
    "docSize": "Size of the encoded document after the task",
    "parseTime": "Time to parse/decode the encoded document",
    "encodeTime": "Time to encode/serialize the document",
    "memUsed": "Memory used to hold the decoded document",
}

# ─── Chart palette ────────────────────────────────────────────────────────────

COLORS = {
    "yjs": "#4CAF50",
    "ywasm": "#2196F3",
    "loro": "#FF9800",
    "automerge": "#9C27B0",
    "mdcs-sdk": "#F44336",
}


def _unit_label(metric: str) -> str:
    if metric in ("time", "parseTime", "encodeTime"):
        return "ms"
    if metric in ("avgUpdateSize", "updateSize", "docSize", "memUsed"):
        return "bytes"
    return ""


def _format_bar_label(val: float, metric: str) -> str:
    """Pretty-print a value for bar annotations."""
    if val >= 1_000_000:
        return f"{val / 1_000_000:.1f}M"
    if val >= 1_000:
        return f"{val / 1_000:.1f}K"
    if val < 1:
        return f"{val:.2f}"
    return f"{val:.0f}"


# ─── Chart generation ─────────────────────────────────────────────────────────


def make_grouped_bar(
    benchmarks: list[str],
    metric: str,
    values_per_lib: dict[str, list[float | None]],
    title: str,
    outpath: str,
    use_log_scale: bool = False,
    subtitle: str = "",
):
    """Create a single grouped-bar chart comparing libraries across benchmarks."""
    libs = [lib for lib in COMPETITORS if lib in values_per_lib]
    n_benches = len(benchmarks)
    n_libs = len(libs)
    bar_width = 0.8 / n_libs
    x = np.arange(n_benches)

    fig, ax = plt.subplots(figsize=(max(14, n_benches * 1.5), 8.2))

    for i, lib in enumerate(libs):
        vals = values_per_lib[lib]
        plot_vals = [v if v is not None else 0 for v in vals]
        offset = (i - n_libs / 2 + 0.5) * bar_width
        bars = ax.bar(
            x + offset,
            plot_vals,
            bar_width,
            label=lib,
            color=COLORS.get(lib, "#888"),
            edgecolor="white",
            linewidth=0.5,
            zorder=3,
        )
        # Annotate bars
        for bar, v in zip(bars, vals):
            if v is not None and v > 0:
                label = _format_bar_label(v, metric)
                y_pos = bar.get_height()
                ax.annotate(
                    label,
                    xy=(bar.get_x() + bar.get_width() / 2, y_pos),
                    xytext=(0, 4),
                    textcoords="offset points",
                    ha="center",
                    va="bottom",
                    fontsize=6,
                    rotation=45,
                )

    ax.set_xlabel("Benchmark", fontsize=11, fontweight="bold")
    direction = METRIC_DIRECTION.get(metric, "lower is better")
    direction_arrow = "\u25bc" if "lower" in direction else "\u25b2"
    ax.set_ylabel(
        f"{metric} ({_unit_label(metric)})  [{direction_arrow} {direction}]",
        fontsize=11, fontweight="bold",
    )
    ax.set_title(title, fontsize=13, fontweight="bold", pad=30)
    # Add subtitle with metric description and group context
    if subtitle:
        fig.text(
            0.5, 0.97, subtitle,
            ha="center", va="top", fontsize=9,
            fontstyle="italic", color="#555",
            wrap=True,
        )
    ax.set_xticks(x)
    ax.set_xticklabels(benchmarks, rotation=35, ha="right", fontsize=8)
    ax.legend(loc="upper left", fontsize=9, framealpha=0.9)
    ax.grid(axis="y", alpha=0.3, zorder=0)

    if use_log_scale:
        ax.set_yscale("log")
        ax.yaxis.set_major_formatter(ticker.FuncFormatter(lambda v, _: _format_bar_label(v, metric) if v > 0 else "0"))

    fig.tight_layout(rect=[0, 0, 1, 0.95])
    fig.savefig(outpath, dpi=180, bbox_inches="tight")
    plt.close(fig)
    print(f"  saved → {outpath}")


def generate_metric_chart(
    tests: OrderedDict,
    bench_labels: list[str],
    metric: str,
    title: str,
    outpath: str,
    use_log: bool = False,
    group: str = "",
):
    """Build data for one metric across given benchmarks and plot."""
    values_per_lib: dict[str, list[float | None]] = {lib: [] for lib in COMPETITORS}

    for label in bench_labels:
        log_name = LABEL_TO_LOG_NAME[label]
        comp = COMPARISON_DATA.get(label, {}).get(metric, {})
        for lib in COMPETITORS:
            if lib == "mdcs-sdk":
                val = extract_mdcs_value(tests, log_name, metric)
            else:
                val = comp.get(lib)
            values_per_lib[lib].append(val)

    # Only plot if at least one library has data
    has_data = any(any(v is not None for v in vals) for vals in values_per_lib.values())
    if not has_data:
        return

    # Build subtitle: metric description + group description
    parts = []
    metric_desc = METRIC_DESCRIPTIONS.get(metric, "")
    if metric_desc:
        direction = METRIC_DIRECTION.get(metric, "lower is better")
        parts.append(f"{metric_desc} ({direction})")
    group_desc = GROUP_DESCRIPTIONS.get(group, "")
    if group_desc:
        parts.append(group_desc)
    subtitle = ". ".join(parts)

    make_grouped_bar(
        bench_labels, metric, values_per_lib, title, outpath,
        use_log_scale=use_log, subtitle=subtitle,
    )


def generate_overview_chart(tests: OrderedDict, outdir: str):
    """Generate a summary heatmap-style chart showing wins per metric per benchmark."""
    metrics = ["time", "docSize", "avgUpdateSize", "parseTime"]
    bench_labels = list(COMPARISON_DATA.keys())
    libs = COMPETITORS

    # Build win matrix: for each (benchmark, metric), which lib is best?
    win_counts = {lib: 0 for lib in libs}
    cells = []  # (bench_idx, metric_idx, winner_lib)

    for bi, label in enumerate(bench_labels):
        for mi, metric in enumerate(metrics):
            comp = COMPARISON_DATA.get(label, {}).get(metric, {})
            log_name = LABEL_TO_LOG_NAME[label]
            best_lib = None
            best_val = float("inf")
            for lib in libs:
                if lib == "mdcs-sdk":
                    v = extract_mdcs_value(tests, log_name, metric)
                else:
                    v = comp.get(lib)
                if v is not None and v < best_val:
                    best_val = v
                    best_lib = lib
            if best_lib:
                win_counts[best_lib] += 1
                cells.append((bi, mi, best_lib))

    # Plot win counts as a bar chart
    fig, ax = plt.subplots(figsize=(10, 5))
    lib_names = [lib for lib in libs if win_counts[lib] > 0]
    counts = [win_counts[lib] for lib in lib_names]
    colors = [COLORS.get(lib, "#888") for lib in lib_names]
    bars = ax.bar(lib_names, counts, color=colors, edgecolor="white", linewidth=1, zorder=3)
    for bar, c in zip(bars, counts):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.3,
                str(c), ha="center", va="bottom", fontweight="bold", fontsize=11)

    ax.set_ylabel("Wins (Higher  = best)", fontsize=11, fontweight="bold")
    ax.set_title(
        "Best-in-class wins across all benchmarks\n(time, docSize, avgUpdateSize, parseTime — all)",
        fontsize=12, fontweight="bold",
    )
    ax.grid(axis="y", alpha=0.3, zorder=0)
    fig.tight_layout()
    path = os.path.join(outdir, "overview_wins.png")
    fig.savefig(path, dpi=180, bbox_inches="tight")
    plt.close(fig)
    print(f"  saved → {path}")


def generate_radar_chart(tests: OrderedDict, outdir: str):
    """Generate a radar/spider chart comparing MDCS strengths across metric categories."""
    # Aggregate: for each metric, compute the average rank of each library
    metrics = ["time", "docSize", "avgUpdateSize", "parseTime"]
    metric_labels = [
        "Time Efficiency\n(Speed)", 
        "Storage Efficiency\n(Doc Size)", 
        "Network Efficiency\n(Update Size)", 
        "Parse Speed"
    ]
    bench_labels = list(COMPARISON_DATA.keys())

    avg_rank: dict[str, list[float]] = {lib: [] for lib in COMPETITORS}

    for metric in metrics:
        ranks_sum = {lib: 0.0 for lib in COMPETITORS}
        rank_count = {lib: 0 for lib in COMPETITORS}

        for label in bench_labels:
            comp = COMPARISON_DATA.get(label, {}).get(metric, {})
            log_name = LABEL_TO_LOG_NAME[label]
            lib_vals = []
            for lib in COMPETITORS:
                if lib == "mdcs-sdk":
                    v = extract_mdcs_value(tests, log_name, metric)
                else:
                    v = comp.get(lib)
                lib_vals.append((lib, v))

            # Rank (lower value = better = rank 1)
            valid = [(lib, v) for lib, v in lib_vals if v is not None]
            valid.sort(key=lambda x: x[1])
            for rank_idx, (lib, _) in enumerate(valid):
                ranks_sum[lib] += rank_idx + 1
                rank_count[lib] += 1

        for lib in COMPETITORS:
            if rank_count[lib] > 0:
                avg_rank[lib].append(ranks_sum[lib] / rank_count[lib])
            else:
                avg_rank[lib].append(float(len(COMPETITORS)))

    # Invert: lower rank = better → higher on radar means better
    max_rank = len(COMPETITORS)
    angles = np.linspace(0, 2 * np.pi, len(metrics), endpoint=False).tolist()
    angles += angles[:1]  # close polygon

    fig, ax = plt.subplots(figsize=(8, 8), subplot_kw={"polar": True})
    for lib in COMPETITORS:
        scores = [max_rank + 1 - r for r in avg_rank[lib]]
        scores += scores[:1]
        ax.plot(angles, scores, "o-", label=lib, color=COLORS.get(lib, "#888"), linewidth=2, markersize=5)
        ax.fill(angles, scores, alpha=0.08, color=COLORS.get(lib, "#888"))

    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(metric_labels, fontsize=10)
    ax.set_title(
        "Average Ranking (higher = better)\nacross all benchmarks",
        fontsize=12, fontweight="bold", pad=20,
    )
    ax.legend(loc="upper right", bbox_to_anchor=(1.3, 1.1), fontsize=9)
    fig.tight_layout()
    path = os.path.join(outdir, "radar_rankings.png")
    fig.savefig(path, dpi=180, bbox_inches="tight")
    plt.close(fig)
    print(f"  saved → {path}")


# ─── Main driver ──────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(
        description="Generate MDCS vs competitors comparison charts"
    )
    parser.add_argument(
        "--logfile",
        default=os.path.join(os.path.dirname(__file__), "logs", "benchmark_results.txt"),
        help="Path to MDCS raw benchmark log (default: logs/benchmark_results.txt)",
    )
    parser.add_argument(
        "--outdir",
        default=os.path.join(os.path.dirname(__file__), "charts"),
        help="Output directory for chart images (default: benchmarks/charts/)",
    )
    args = parser.parse_args()

    tests = parse_log(args.logfile)
    if not tests:
        print(f"ERROR: No benchmark data found in {args.logfile}")
        raise SystemExit(1)

    os.makedirs(args.outdir, exist_ok=True)
    print(f"Parsed {len(tests)} benchmarks from {args.logfile}")
    print(f"Output directory: {args.outdir}\n")

    # ── B1 benchmarks ──
    b1_labels = [l for l in COMPARISON_DATA if l.startswith("B1")]
    b2_labels = [l for l in COMPARISON_DATA if l.startswith("B2")]
    b3_labels = [l for l in COMPARISON_DATA if l.startswith("B3")]

    # === Execution time charts (log scale — values span orders of magnitude) ===
    print("── Execution Time ──")
    generate_metric_chart(
        tests, b1_labels, "time",
        "B1: Execution Time — No Conflicts (N=6000)",
        os.path.join(args.outdir, "b1_time.png"), use_log=True, group="B1",
    )
    generate_metric_chart(
        tests, b2_labels, "time",
        "B2: Execution Time — Two Users Conflicts (N=6000)",
        os.path.join(args.outdir, "b2_time.png"), use_log=True, group="B2",
    )
    generate_metric_chart(
        tests, b3_labels, "time",
        "B3: Execution Time — Many Conflicts (N=6000)",
        os.path.join(args.outdir, "b3_time.png"), use_log=True, group="B3",
    )

    # === Document size charts (log scale) ===
    print("\n── Document Size ──")
    generate_metric_chart(
        tests, b1_labels, "docSize",
        "B1: Encoded Document Size — No Conflicts (N=6000)",
        os.path.join(args.outdir, "b1_docsize.png"), use_log=True, group="B1",
    )
    generate_metric_chart(
        tests, b2_labels, "docSize",
        "B2: Encoded Document Size — Two Users Conflicts (N=6000)",
        os.path.join(args.outdir, "b2_docsize.png"), use_log=True, group="B2",
    )
    generate_metric_chart(
        tests, b3_labels, "docSize",
        "B3: Encoded Document Size — Many Conflicts (N=6000)",
        os.path.join(args.outdir, "b3_docsize.png"), use_log=True, group="B3",
    )

    # === Average update size charts ===
    print("\n── Average Update Size ──")
    b1_with_update = [l for l in b1_labels if "avgUpdateSize" in COMPARISON_DATA.get(l, {})]
    if b1_with_update:
        generate_metric_chart(
            tests, b1_with_update, "avgUpdateSize",
            "B1: Average Update Size — No Conflicts (N=6000)",
            os.path.join(args.outdir, "b1_updatesize.png"), use_log=True, group="B1",
        )
    b2_with_update = [l for l in b2_labels if "updateSize" in COMPARISON_DATA.get(l, {})]
    if b2_with_update:
        generate_metric_chart(
            tests, b2_with_update, "updateSize",
            "B2: Update Size — Two Users Conflicts (N=6000)",
            os.path.join(args.outdir, "b2_updatesize.png"), use_log=True, group="B2",
        )

    # === Parse time charts ===
    print("\n── Parse Time ──")
    generate_metric_chart(
        tests, b1_labels, "parseTime",
        "B1: Parse Time — No Conflicts (N=6000)",
        os.path.join(args.outdir, "b1_parsetime.png"), use_log=True, group="B1",
    )
    generate_metric_chart(
        tests, b2_labels, "parseTime",
        "B2: Parse Time — Two Users Conflicts (N=6000)",
        os.path.join(args.outdir, "b2_parsetime.png"), use_log=True, group="B2",
    )
    generate_metric_chart(
        tests, b3_labels, "parseTime",
        "B3: Parse Time — Many Conflicts (N=6000)",
        os.path.join(args.outdir, "b3_parsetime.png"), use_log=True, group="B3",
    )

    # === Overview / summary charts ===
    print("\n── Summary ──")
    generate_overview_chart(tests, args.outdir)
    generate_radar_chart(tests, args.outdir)

    print(f"\nDone! {len(os.listdir(args.outdir))} charts written to {args.outdir}/")


if __name__ == "__main__":
    main()
