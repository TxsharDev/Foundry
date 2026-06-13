#!/usr/bin/env python3
"""Run all benchmarks and aggregate results into bench_results.json."""

import json
import os
import subprocess
import sys

BENCH_DIR = os.path.dirname(os.path.abspath(__file__))

BENCHMARKS = [
    ("bench_binary_size.py", "_size.json"),
    ("bench_startup.py", "_startup.json"),
    ("bench_parse_speed.py", "_parse.json"),
    # compile_speed is slow (clean build), run separately if needed
]


def main():
    results = {}
    python = sys.executable

    for script, output_file in BENCHMARKS:
        script_path = os.path.join(BENCH_DIR, script)
        print(f"\n{'='*60}")
        print(f"Running {script}...")
        print(f"{'='*60}\n")

        result = subprocess.run(
            [python, script_path],
            cwd=BENCH_DIR,
        )
        if result.returncode != 0:
            print(f"WARNING: {script} failed")
            continue

        out_path = os.path.join(BENCH_DIR, output_file)
        if os.path.exists(out_path):
            with open(out_path) as f:
                data = json.load(f)
            key = script.replace("bench_", "").replace(".py", "")
            results[key] = data
            os.remove(out_path)  # cleanup temp files

    # Save aggregated results
    out = os.path.join(BENCH_DIR, "bench_results.json")
    with open(out, "w") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to {out}")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
