# MarkZap Presentation

A fast Markdown viewer built with Rust

---

## Slide 2 — Features

- GPU-accelerated rendering with GPUI
- Instant preview / edit switching
- Native macOS & Windows support

---

## Slide 3 — Architecture

```
main.rs      → CLI + Window
app.rs       → AppView + State
slidev.rs    → Detection + HTML
presentation → WebView + Reveal.js
```

---

## Slide 4 — Thank you!

Built with Rust, GPUI, and gpui-component.
