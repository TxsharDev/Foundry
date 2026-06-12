# How It Works

## Overview

Foundry processes HTML/CSS/JS in a compile-time pipeline. Static styles are resolved once. Animations and JS mutations trigger incremental updates at runtime.

```
HTML ──> Single-Pass Parser ──> Scene Graph + Styles + Scripts
                                     │
                          CSS Cascade + @keyframes ──> Resolved Styles + Hover Styles + Animations
                                     │
                              taffy Flexbox ──> Layout Rects
                                     │
                           wgpu SDF Renderer ──> GPU Draw Calls
                                     │
                          boa JS Engine ──> DOM Mutations ──> Dirty Flags ──> Re-layout
```

## Stage 1: Single-Pass HTML Parsing

`html5ever` (Mozilla's parser from Servo) builds a DOM tree. Foundry walks it once and extracts everything in a single pass:
- Scene graph nodes (elements, text, attributes)
- Inline `<style>` blocks
- Inline `<script>` blocks
- External `<link rel="stylesheet" href="...">` paths
- External `<script src="...">` paths

No triple-parse. One DOM walk produces all data.

## Stage 2: CSS Resolution

The CSS parser handles:
- **Regular rules** with compound selectors (`div.btn.active`, `.card h1`)
- **`:hover` pseudo-class** rules (stored separately as hover style overrides)
- **`@keyframes`** blocks (parsed into keyframe arrays with percentage stops)
- **`transition`** property (duration, property selection)
- **`animation`** property (name, duration, delay, iteration-count, direction)

Resolution order (same as browsers):
1. Element defaults (h1 = 32px bold, span = inline)
2. Stylesheet rules sorted by specificity (tag < class < id)
3. Inline `style` attributes (highest specificity)
4. Inherited properties (color, font-family) from parent

This resolution happens once. No per-frame style recalculation.

## Stage 3: Layout

`taffy` computes flexbox layout. Text nodes inherit parent font-size for accurate measurement. Viewport-relative units (vh, vw) resolve against window size. HTML and body elements automatically fill the viewport.

## Stage 4: GPU Rendering

The SDF fragment shader renders ALL UI primitives in one draw call:
- Rounded rectangles with per-corner radius
- Borders (inner SDF subtracted from outer SDF)
- Anti-aliased edges via `smoothstep`
- Transparency and opacity blending

Text is rendered by glyphon (GPU text atlas) with system font discovery. The clear color matches the body background.

## Stage 5: Animations

Two animation systems run per-frame in `tick_animations()`:

**CSS Transitions:** When `:hover` activates, the style lerps from base to hover over the transition duration. Colors, opacity, padding, border-radius, margin all interpolate smoothly.

**CSS @keyframes:** Multi-stop keyframe animations with support for:
- Percentage stops (0%, 50%, 100%)
- `from`/`to` shorthand
- `infinite` iteration count
- `alternate` direction (ping-pong)
- Delay before start
- Style property interpolation between keyframes

## Stage 6: JS Execution

`boa` (pure-Rust ECMAScript engine) executes scripts after the first layout. The DOM bridge:
- `document.getElementById()` returns a proxy object
- `.setTextContent()`, `.setStyle()`, `.addClass()`, `.removeClass()` queue mutations
- Mutations apply in batch after execution, trigger dirty flags, re-layout affected subtrees
- Event handlers fire on click/hover via bounding-box hit testing with event bubbling

## Stage 7: Binary Compilation

`foundry build` generates a Rust project that embeds the HTML as a string constant, depends on `foundry_runtime` (the library), and compiles to a standalone binary via `cargo build --release`. The output is a self-contained .exe with no external dependencies.

## What "Compile-Time Resolution" Means

- **Static styles:** resolved once at load time, never recomputed
- **Dynamic styles (JS):** trigger incremental re-layout via dirty flags
- **Hover styles:** swapped on mouse enter/leave, optionally animated
- **Animations:** interpolated per-frame from pre-resolved keyframes
- **Viewport units:** resolved at window creation and on resize

This is architecturally different from a browser, which runs cascade resolution on every style change.
