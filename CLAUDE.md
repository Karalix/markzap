# MarkZap

A native macOS Markdown viewer and presentation app built with Rust and GPUI.

## Tech Stack

- **Language:** Rust (nightly toolchain, edition 2024)
- **UI framework:** [GPUI](https://crates.io/crates/gpui) + [gpui-component](https://crates.io/crates/gpui-component)
- **Markdown parsing:** [comrak](https://crates.io/crates/comrak)
- **Presentation WebView:** [wry](https://crates.io/crates/lb-wry) (renders Reveal.js slides)
- **Platform:** macOS only (uses .app bundling, Launch Services, file associations)

## Project Structure

```
src/
  main.rs          — Entry point, CLI arg parsing, macOS open-url handling, window creation
  app.rs           — AppView: top bar (Preview/Edit toggle, Presentation button), content area
  state.rs         — AppMode enum (Preview, Edit)
  slidev.rs        — Slide detection heuristics, Reveal.js HTML generation
  views/
    mod.rs         — Module re-exports
    presentation.rs — Opens a separate WebView window for presentations
scripts/
  bundle.sh        — Creates a macOS .app bundle
resources/
  Info.plist       — macOS app metadata and file associations (.md)
```

## Build & Run

```bash
# Build (release)
cargo build --release

# Build and create .app bundle
make bundle

# Install to /Applications
make install

# Run directly
cargo run -- path/to/file.md

# Clean
make clean
```

## Key Concepts

- The app opens `.md` files via CLI argument or macOS file association (double-click).
- **Preview mode** renders Markdown via `TextView::markdown` from gpui-component.
- **Edit mode** uses a code editor (`InputState::code_editor("markdown")`).
- If the Markdown contains Slidev-style separators (`---`), a "Presentation" button appears that opens a Reveal.js-powered WebView in a separate window.
