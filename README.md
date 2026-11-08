# 🚀 NgNative Engine (`ng-native-engine`)

A high-performance, memory-safe **Native Desktop Rendering Engine for Angular**, written in **Rust**.

`ng-native-engine` replaces `@angular/platform-browser` and the browser DOM/WebView completely. Instead of bundling Chromium (Electron) or relying on an embedded HTML WebView (Tauri), Angular components drive **100% native OS widgets** — starting with **Linux (GTK4 / Libadwaita)** and architected via the **Bridge** and **Abstract Factory** patterns to target **Windows (WinUI 3)** and **macOS (AppKit / Cocoa)**.

---

## 🎯 Architectural Principles

1. **Zero WebView Overhead:** Angular templates compile and execute against native operating system controls (`AdwApplicationWindow`, `GtkBox`, `GtkLabel`, `GtkButton`), delivering sub-second startup and a tiny memory footprint.
2. **Rust-Grade Memory Safety Across the JS/Native Boundary:**
   - **Generational Arena (`Arena` + `NodeId`):** JavaScript never holds raw C/GTK pointers. Every native widget is stored inside a Rust generational arena (`index` + `generation`), preventing use-after-free bugs when components unmount.
   - **Runtime Borrow Checker (`BorrowState`):** Enforces Rust's aliasing rules (`Shared(usize)` vs `Exclusive`) at runtime across asynchronous UI events, blocking illegal node destruction while an event callback is actively borrowing a widget.
3. **Game-Engine Inspired Render Pipeline (`CommandQueue`):**
   - Angular mutations during a change-detection tick are batched into an OS-agnostic [`RenderCommand`](./src/engine/commands.rs) buffer (`CreateWidget`, `SetText`, `SetWindowIcon`, `RegisterStyle`, `ApplyStyleClass`, `AppendChild`, `DestroyWidget`) and flushed atomically per frame.
4. **SOLID & Hexagonal Platform Architecture (`PlatformBackend`):**
   - The core [`RenderEngine<B: PlatformBackend>`](./src/engine/mod.rs) depends exclusively on an abstract trait, never on concrete OS toolkits.
   - Concrete platform adapters live in [`src/platforms/`](./src/platforms/):
     - **Linux GTK4 (`GtkLinuxBackend`):** Dynamically loads `libgtk-4.so.1`, `libgobject-2.0.so.0`, `libgio-2.0.so.0`, and `libglib-2.0.so.0` via an isolated FFI boundary ([`gtk4_sys.rs`](./src/platforms/gtk4_sys.rs)).
     - **Windows WinUI 3 (`WinUiBackend`):** Maps abstract primitives to `Microsoft.UI.Xaml` controls.
     - **macOS AppKit (`MacOsAppKitBackend`):** Maps abstract primitives to Cocoa `NSWindow` / `NSView` controls.
     - **Headless CI (`HeadlessBackend`):** Records every frame's `RenderCommand` batch in memory for deterministic integration testing in CI without a display server.

---

## 📂 Repository Structure

```text
ng-native-engine/
├── src/
│   ├── lib.rs                    # Library entry point
│   ├── main.rs                   # Native engine runner & GTK4 event loop host
│   ├── config/
│   │   └── mod.rs                # Angular CLI (angular.json) native target parser
│   ├── engine/                   # OS-agnostic core rendering engine (100% safe Rust)
│   │   ├── mod.rs                # RenderEngine<B> scene graph & frame orchestrator
│   │   ├── arena.rs              # Generational Arena + runtime BorrowState checker
│   │   ├── commands.rs           # Frame-batched RenderCommand queue
│   │   ├── events.rs             # Native-to-Angular event listener registry
│   │   ├── icons.rs              # Binary .ico multi-resolution parser & WindowIconAsset
│   │   ├── node.rs               # NodeId, StyleId, BorrowState, and EngineError types
│   │   ├── preview.rs            # Scene graph tree inspector & ASCII window previewer
│   │   ├── styles.rs             # Deduplicated StyleRegistry (CSS resource manager)
│   │   └── widget.rs             # Cross-platform semantic widget model (ng-window, ng-box, ...)
│   ├── platforms/                # Concrete OS backend adapters (Bridge + Factory)
│   │   ├── mod.rs                # PlatformBackend trait & AnyPlatformBackend factory
│   │   ├── gtk4_sys.rs           # Isolated C FFI dynamic loader for libgtk-4.so.1
│   │   ├── linux_gtk.rs          # Native Linux GTK4 + Freedesktop hicolor icon adapter
│   │   ├── windows_winui.rs      # Windows WinUI 3 adapter
│   │   ├── macos_appkit.rs       # macOS AppKit (Cocoa) adapter
│   │   └── headless.rs           # Headless frame recorder for automated CI tests
│   └── runtime/
│       ├── mod.rs                # Embedded reactive Signal, HMR & event runtime
│       └── parser.rs             # Bytecode (.ngb) instruction IR & syntax parser
├── examples/
│   └── counter-app/              # Official Angular 21 application workspace
│       ├── build-bundle.mjs      # Template, CSS, Signal & multi-DPI icon compiler (--watch capable)
│       ├── dev-server.mjs        # Live Hot-Reload dev orchestrator (file watcher + native window)
│       ├── public/favicon.ico    # Multi-resolution (16x16, 32x32, 48x48) Angular icon
│       └── src/app/              # Angular 21 @Component (app.ts, app.html, app.css)
├── tests/
│   └── engine_integration.rs     # End-to-end HeadlessBackend & cross-platform factory tests
├── ROADMAP.md                    # Deep architectural roadmap toward 100% Angular support
└── CHANGELOG.md                  # Release & milestone history
```

---

## ⚡ Quick Start

### Prerequisites

- **Rust** (stable toolchain, 2024 edition)
- **Node.js** `>= 24` & **pnpm** `>= 11`
- **Linux Desktop (optional for live GUI):** `libgtk-4-1` runtime library (`libgtk-4.so.1` — pre-installed on modern Ubuntu/Debian/Fedora/Arch desktops; no `-dev` C headers required).

### 1. Install Angular & Tooling Dependencies

```bash
pnpm install
```

### 2. Compile the Angular 21 Project & Native Bundle (`.ngb`)

```bash
pnpm run build
```

This runs the official Angular 21 compiler (`ng build`) followed by the native compiler (`examples/counter-app/build-bundle.mjs`), generating:

- `dist/ng-linux-counter/main.ngb` (compiled native instructions, styles, and reactive signal bindings)
- `dist/ng-linux-counter/icons/hicolor/<size>x<size>/apps/ng-linux-counter.png` (`16x16` through `256x256` taskbar icons)

### 3. Live Development with State-Preserving Hot Reload (`pnpm run dev`)

```bash
pnpm run dev
```

Starts the file watcher on `examples/counter-app/src/` and opens the native Linux GTK4 window. Whenever you save `app.ts`, `app.html`, or `app.css`, `build-bundle.mjs` recompiles `main.ngb` and the Rust engine hot-reloads styles, text, and bindings **in-place without closing the window or losing current Angular Signal state** (`count`).

### 4. Launch the Native Linux GTK4 Window Directly

```bash
cargo run
```

To run in non-blocking batch mode (e.g., in terminal/CI scripts):

```bash
NG_NATIVE_BATCH=1 cargo run
```

---

## 🛠️ Quality & Verification Commands

| Command                                     | Description                                                          |
| :------------------------------------------ | :------------------------------------------------------------------- |
| `pnpm run dev`                              | Watch Angular files & hot-reload native GTK4 window in real time     |
| `pnpm run check`                            | Run full verification gate (OxFmt + OxLint + ESLint + Clippy + Test) |
| `pnpm run format:all`                       | Format both TypeScript/Angular (`oxfmt`) and Rust (`cargo fmt`)      |
| `pnpm run format`                           | Format TypeScript, HTML, CSS, JSON, and Markdown via `oxfmt`         |
| `pnpm run format:check`                     | Verify formatting with `oxfmt`                                       |
| `pnpm run lint`                             | Lint TypeScript & Angular templates via `oxlint` + `angular-eslint`  |
| `cargo fmt --all -- --check`                | Verify Rust formatting                                               |
| `cargo clippy --all-targets -- -D warnings` | Run strict Rust Clippy lints (pedantic + nursery)                    |
| `cargo test`                                | Execute unit tests (`Arena`, `IcoParser`) + `HeadlessBackend` E2E    |

---

## 🗺️ Roadmap & Changelog

- See [**ROADMAP.md**](./ROADMAP.md) for the complete technical analysis and phased milestones to reach **100% Angular ecosystem compatibility** on Linux and desktop platforms.
- See [**CHANGELOG.md**](./CHANGELOG.md) for completed milestones and release notes.
