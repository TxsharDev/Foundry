#!/usr/bin/env python3
"""Measure Foundry compilation time and compare against Electron packaging."""

import json
import os
import subprocess
import sys
import time

FOUNDRY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Electron packaging reference times (electron-builder on comparable hardware)
ELECTRON_FIRST_BUILD_S = 120.0   # first build with download
ELECTRON_REBUILD_S = 30.0        # incremental rebuild
TAURI_FIRST_BUILD_S = 180.0      # first cargo build + bundle
TAURI_REBUILD_S = 15.0           # incremental Rust rebuild


def run_clean_build():
    """Full clean + release build."""
    cargo = os.environ.get("CARGO", "cargo")

    # Clean first
    subprocess.run(
        [cargo, "clean"],
        cwd=FOUNDRY_ROOT,
        capture_output=True,
    )

    # Time the release build
    t0 = time.perf_counter()
    result = subprocess.run(
        [cargo, "build", "--release"],
        cwd=FOUNDRY_ROOT,
        capture_output=True,
        text=True,
    )
    t1 = time.perf_counter()

    if result.returncode != 0:
        print(f"ERROR: build failed\n{result.stderr}")
        sys.exit(1)

    return t1 - t0


def run_incremental_build():
    """Touch a source file and rebuild (incremental)."""
    cargo = os.environ.get("CARGO", "cargo")

    # Touch main.rs to trigger incremental
    main_rs = os.path.join(FOUNDRY_ROOT, "src", "main.rs")
    os.utime(main_rs)

    t0 = time.perf_counter()
    result = subprocess.run(
        [cargo, "build", "--release"],
        cwd=FOUNDRY_ROOT,
        capture_output=True,
        text=True,
    )
    t1 = time.perf_counter()

    if result.returncode != 0:
        print(f"ERROR: incremental build failed\n{result.stderr}")
        sys.exit(1)

    return t1 - t0


def main():
    print("=" * 60)
    print("COMPILE SPEED BENCHMARK")
    print("=" * 60)

    print("Running clean release build (this takes a while)...")
    clean_time = run_clean_build()
    print(f"  Clean build: {clean_time:.1f}s")

    print("Running incremental build...")
    incr_time = run_incremental_build()
    print(f"  Incremental: {incr_time:.1f}s")

    results = {
        "foundry_clean_build_s": round(clean_time, 1),
        "foundry_incremental_s": round(incr_time, 1),
        "electron_first_build_s": ELECTRON_FIRST_BUILD_S,
        "electron_rebuild_s": ELECTRON_REBUILD_S,
        "tauri_first_build_s": TAURI_FIRST_BUILD_S,
        "tauri_rebuild_s": TAURI_REBUILD_S,
    }

    print()
    print(f"{'Framework':<30} {'Clean Build':>12} {'Incremental':>12}")
    print("-" * 60)
    print(f"{'Foundry':<30} {clean_time:>10.1f}s {incr_time:>10.1f}s")
    print(f"{'Electron (electron-builder)':<30} {ELECTRON_FIRST_BUILD_S:>10.1f}s {ELECTRON_REBUILD_S:>10.1f}s")
    print(f"{'Tauri':<30} {TAURI_FIRST_BUILD_S:>10.1f}s {TAURI_REBUILD_S:>10.1f}s")
    print()

    return results


if __name__ == "__main__":
    results = main()
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_compile.json")
    with open(out, "w") as f:
        json.dump(results, f, indent=2)
