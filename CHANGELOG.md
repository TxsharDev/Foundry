# Changelog

## v0.1.0 (2026-06-11)

### Binary Compilation
- `foundry build index.html -o app.exe` compiles HTML/CSS/JS to a standalone native binary
- Generated binary is ~7.6MB, runs independently with no Foundry or Rust toolchain needed
- Embeds HTML source into a generated Rust project, links against foundry_runtime library

### Rendering Engine
- wgpu GPU renderer with SDF fragment shader (rounded corners, borders, anti-aliasing in one draw call)
- glyphon/cosmic-text GPU text rendering with system font discovery
- Dynamic clear color from body background
- Body and html elements auto-fill viewport

### CSS Engine
- Full cascade, specificity, and inheritance resolution (one-time, not per-frame)
- Compound selectors: `div.btn`, `.card.active`, `.card h1`, `#main .text`
- `:hover` pseudo-class with automatic style swapping
- `transition` property with smooth color/opacity/padding/border-radius interpolation
- `@keyframes` with percentage stops, `from`/`to`, `infinite`, `alternate`
- `animation` shorthand (name, duration, delay, iteration-count, direction)
- 40+ named colors, hex (#RGB/#RRGGBB/#RRGGBBAA), rgb(), rgba()
- Units: px, %, em, rem, vh, vw

### Layout
- taffy-backed flexbox (display, flex-direction, justify-content, align-items, flex-wrap, gap, flex-grow)
- Box model (margin, padding, border, border-radius)
- Position (relative, absolute, fixed)
- Overflow (visible, hidden, scroll)
- Font-aware text measurement with parent font-size inheritance

### JavaScript
- boa engine with DOM bridge: getElementById, setTextContent, setStyle, addClass, removeClass
- Event handling: onclick, onmouseenter, onmouseleave
- Event bubbling (walk up from hit node)
- console.log support

### Developer Experience
- `foundry dev` with hot reload (file watching via notify crate)
- Single-pass HTML parsing (no redundant DOM walks)
- External `<link>` stylesheets and `<script src>` resolution

### Testing
- 55 tests across 4 test suites
- Covers: CSS parsing, color parsing, selectors, compound selectors, hover, transitions, animations, scene graph, element types

### Examples
- counter.html (14 nodes) -- interactive counter with hover buttons
- todo.html (32 nodes) -- task list with status
- dashboard.html (117 nodes) -- monitoring dashboard with stats, service table, action buttons
- showcase.html (78 nodes) -- animated landing page with @keyframes
