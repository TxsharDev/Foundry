#!/usr/bin/env python3
"""Measure Foundry binary size and compare against Electron/Tauri."""

import json
import os
import sys

FOUNDRY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY_PATH = os.path.join(FOUNDRY_ROOT, "target", "release", "foundry.exe")
if not os.path.exists(BINARY_PATH):
    # Unix fallback
    BINARY_PATH = os.path.join(FOUNDRY_ROOT, "target", "release", "foundry")

# Electron and Tauri reference sizes (well-documented industry numbers)
# Electron: minimal hello-world app built with electron-builder
# Tauri: minimal hello-world app built with tauri build
ELECTRON_MIN_MB = 150.0   # electron-builder minimal Windows .exe
ELECTRON_TYPICAL_MB = 200.0
TAURI_MIN_MB = 2.5         # tauri v2 minimal Windows .exe
TAURI_TYPICAL_MB = 8.0


def main():
    if not os.path.exists(BINARY_PATH):
        print(f"ERROR: binary not found at {BINARY_PATH}")
        print("Run: cargo build --release")
        sys.exit(1)

    size_bytes = os.path.getsize(BINARY_PATH)
    size_mb = size_bytes / (1024 * 1024)

    results = {
        "foundry_binary_bytes": size_bytes,
        "foundry_binary_mb": round(size_mb, 2),
        "electron_typical_mb": ELECTRON_TYPICAL_MB,
        "electron_min_mb": ELECTRON_MIN_MB,
        "tauri_typical_mb": TAURI_TYPICAL_MB,
        "tauri_min_mb": TAURI_MIN_MB,
        "ratio_vs_electron": round(ELECTRON_TYPICAL_MB / size_mb, 1),
        "ratio_vs_tauri_typical": round(TAURI_TYPICAL_MB / size_mb, 1),
    }

    print("=" * 60)
    print("BINARY SIZE COMPARISON")
    print("=" * 60)
    print(f"{'Framework':<20} {'Size':>10} {'vs Foundry':>15}")
    print("-" * 60)
    print(f"{'Foundry':<20} {size_mb:>8.2f} MB {'(baseline)':>15}")
    print(f"{'Tauri (min)':<20} {TAURI_MIN_MB:>8.2f} MB {TAURI_MIN_MB/size_mb:>14.1f}x")
    print(f"{'Tauri (typical)':<20} {TAURI_TYPICAL_MB:>8.2f} MB {TAURI_TYPICAL_MB/size_mb:>14.1f}x")
    print(f"{'Electron (min)':<20} {ELECTRON_MIN_MB:>8.2f} MB {ELECTRON_MIN_MB/size_mb:>14.1f}x")
    print(f"{'Electron (typical)':<20} {ELECTRON_TYPICAL_MB:>8.2f} MB {ELECTRON_TYPICAL_MB/size_mb:>14.1f}x")
    print("-" * 60)
    print(f"Foundry is {results['ratio_vs_electron']}x smaller than Electron")
    print()

    return results


if __name__ == "__main__":
    results = main()
    # Write results for aggregation
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_size.json")
    with open(out, "w") as f:
        json.dump(results, f, indent=2)
