# Changelog

## v0.2.0 (2026-06-30)

Persistence and data access. From renderer to app platform.

### Added
- **localStorage API** — native file-backed key-value storage exposed via JS bridge
  - `getItem(key)`, `setItem(key, value)`, `removeItem(key)`, `clear()`
  - Persists to JSON file on disk, survives app restarts
  - Same API as browser localStorage
- **fetch() API** — synchronous HTTP via native Rust networking (ureq)
  - Returns `{ ok, status, text }` object
  - No Promise chains — blocking call, suitable for native apps
- **Dependencies**: serde, serde_json, ureq
- **12 new tests** (+ 1 network test gated with `#[ignore]`) in `tests/js_api_tests.rs`
  - localStorage: set/get, missing key returns null, remove, clear, overwrite, file persistence
  - fetch: returns object structure, handles bad URLs gracefully
  - DOM mutations: regression tests for setTextContent, setStyle, addClass, removeClass

### Changed
- `JsEngine::new()` now also registers localStorage and fetch APIs
- `JsEngine::with_storage_path()` constructor for file-backed persistence
- Version bump to 0.2.0

### Testing
- 67 total tests pass (55 existing + 12 new)
- Zero warnings on `cargo build`

---

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
