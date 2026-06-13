#!/usr/bin/env python3
"""Measure Foundry's HTML/CSS parsing speed using cargo test timing."""

import json
import os
import subprocess
import sys
import time

FOUNDRY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Test HTML files of varying complexity
TEST_FILES = {
    "showcase.html": os.path.join(FOUNDRY_ROOT, "examples", "showcase.html"),
    "counter.html": os.path.join(FOUNDRY_ROOT, "examples", "counter.html"),
    "dashboard.html": os.path.join(FOUNDRY_ROOT, "examples", "dashboard.html"),
    "todo.html": os.path.join(FOUNDRY_ROOT, "examples", "todo.html"),
}

PARSE_BENCH_RS = r'''
// Inline parse benchmark -- exercises HTML + CSS parse without GPU
use foundry_runtime::html;
use foundry_runtime::css;
use std::time::Instant;

fn main() {
    let files: Vec<(&str, &str)> = vec![
        PLACEHOLDER
    ];

    for (name, path) in &files {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip {}: {}", name, e);
                continue;
            }
        };

        // Warm up
        let _ = html::parse_html_full(&content);

        // Measure N iterations
        let n = 1000;
        let t0 = Instant::now();
        for _ in 0..n {
            let (mut scene, styles, _, _, _) = html::parse_html_full(&content);
            for s in &styles {
                let (rules, kfs) = css::parse_stylesheet_with_keyframes(s);
                scene.keyframes.extend(kfs);
                css::apply_styles(&mut scene, &rules);
            }
        }
        let elapsed = t0.elapsed();
        let per_iter_us = elapsed.as_micros() as f64 / n as f64;
        let nodes = {
            let (scene, _, _, _, _) = html::parse_html_full(&content);
            scene.node_count()
        };
        println!("RESULT:{}:{}:{:.1}:{}", name, nodes, per_iter_us, content.len());
    }
}
'''


def main():
    cargo = os.environ.get("CARGO", "cargo")

    # Build file list for the bench program
    file_entries = []
    for name, path in TEST_FILES.items():
        if os.path.exists(path):
            escaped = path.replace("\\", "/")
            file_entries.append(f'("{name}", "{escaped}")')

    if not file_entries:
        print("ERROR: no test HTML files found")
        sys.exit(1)

    placeholder = ",\n        ".join(file_entries)
    bench_src = PARSE_BENCH_RS.replace("PLACEHOLDER", placeholder)

    # Write temporary bench binary
    bench_dir = os.path.join(FOUNDRY_ROOT, "target", "_bench_parse")
    bench_src_dir = os.path.join(bench_dir, "src")
    os.makedirs(bench_src_dir, exist_ok=True)

    foundry_path = FOUNDRY_ROOT.replace("\\", "/")
    cargo_toml = f'''[package]
name = "bench_parse"
version = "0.1.0"
edition = "2021"

[dependencies]
foundry_runtime = {{ package = "alia-foundry", path = "{foundry_path}" }}
'''
    with open(os.path.join(bench_dir, "Cargo.toml"), "w") as f:
        f.write(cargo_toml)
    with open(os.path.join(bench_src_dir, "main.rs"), "w") as f:
        f.write(bench_src)

    # Build
    print("Building parse benchmark...")
    result = subprocess.run(
        [cargo, "build", "--release", "--manifest-path",
         os.path.join(bench_dir, "Cargo.toml")],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        print(f"Build failed:\n{result.stderr}")
        sys.exit(1)

    # Run
    bench_exe = os.path.join(bench_dir, "target", "release", "bench_parse")
    if sys.platform == "win32":
        bench_exe += ".exe"

    print("Running parse benchmark (1000 iterations per file)...")
    result = subprocess.run([bench_exe], capture_output=True, text=True)

    results = {}
    print()
    print("=" * 60)
    print("PARSE SPEED BENCHMARK")
    print("=" * 60)
    print(f"{'File':<20} {'Nodes':>6} {'Size':>8} {'Parse+CSS':>12}")
    print("-" * 60)

    for line in result.stdout.strip().split("\n"):
        if line.startswith("RESULT:"):
            parts = line.split(":")
            name = parts[1]
            nodes = int(parts[2])
            us_per = float(parts[3])
            size = int(parts[4])
            print(f"{name:<20} {nodes:>6} {size:>6} B {us_per:>9.1f} us")
            results[name] = {
                "nodes": nodes,
                "html_bytes": size,
                "parse_css_us": round(us_per, 1),
            }

    if result.stderr:
        for line in result.stderr.strip().split("\n"):
            if line.startswith("skip"):
                print(f"  {line}")

    print()
    return results


if __name__ == "__main__":
    results = main()
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_parse.json")
    with open(out, "w") as f:
        json.dump(results, f, indent=2)
