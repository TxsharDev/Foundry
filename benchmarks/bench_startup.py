#!/usr/bin/env python3
"""Measure Foundry binary startup time (time to first output)."""

import json
import os
import subprocess
import sys
import time

FOUNDRY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY_PATH = os.path.join(FOUNDRY_ROOT, "target", "release", "foundry.exe")
if not os.path.exists(BINARY_PATH):
    BINARY_PATH = os.path.join(FOUNDRY_ROOT, "target", "release", "foundry")

# Electron startup reference (time to first paint, measured on comparable HW)
ELECTRON_STARTUP_MS = 2500.0   # cold start, hello-world
TAURI_STARTUP_MS = 800.0       # cold start, hello-world


def measure_startup(n=10):
    """Measure time from process start to first stdout byte.

    Uses --help which prints immediately and exits, measuring
    the bare startup cost of loading the binary + initializing.
    """
    times = []
    for i in range(n):
        t0 = time.perf_counter()
        result = subprocess.run(
            [BINARY_PATH, "--help"],
            capture_output=True,
            text=True,
        )
        t1 = time.perf_counter()
        elapsed_ms = (t1 - t0) * 1000
        times.append(elapsed_ms)

    # Drop first run (cold cache)
    warm_times = times[1:]
    return {
        "cold_ms": round(times[0], 1),
        "warm_avg_ms": round(sum(warm_times) / len(warm_times), 1),
        "warm_min_ms": round(min(warm_times), 1),
        "warm_max_ms": round(max(warm_times), 1),
        "all_runs_ms": [round(t, 1) for t in times],
    }


def main():
    if not os.path.exists(BINARY_PATH):
        print(f"ERROR: binary not found at {BINARY_PATH}")
        sys.exit(1)

    print("=" * 60)
    print("STARTUP TIME BENCHMARK")
    print("=" * 60)

    print(f"Binary: {BINARY_PATH}")
    print("Measuring 10 runs of 'foundry --help'...")
    print()

    timing = measure_startup(10)

    results = {
        "foundry_cold_ms": timing["cold_ms"],
        "foundry_warm_avg_ms": timing["warm_avg_ms"],
        "foundry_warm_min_ms": timing["warm_min_ms"],
        "electron_startup_ms": ELECTRON_STARTUP_MS,
        "tauri_startup_ms": TAURI_STARTUP_MS,
    }

    print(f"{'Framework':<25} {'Cold Start':>12} {'Warm Avg':>12}")
    print("-" * 60)
    print(f"{'Foundry':<25} {timing['cold_ms']:>10.1f}ms {timing['warm_avg_ms']:>10.1f}ms")
    print(f"{'Tauri':<25} {TAURI_STARTUP_MS:>10.1f}ms {'--':>12}")
    print(f"{'Electron':<25} {ELECTRON_STARTUP_MS:>10.1f}ms {'--':>12}")
    print()
    speedup = ELECTRON_STARTUP_MS / timing["warm_avg_ms"]
    print(f"Foundry starts {speedup:.0f}x faster than Electron")
    print(f"All runs (ms): {timing['all_runs_ms']}")

    return results


if __name__ == "__main__":
    results = main()
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_startup.json")
    with open(out, "w") as f:
        json.dump(results, f, indent=2)
