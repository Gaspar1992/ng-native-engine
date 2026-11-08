# Changelog

All notable changes and completed roadmap tasks for `ng-native-engine` are documented in this file.
This project adheres to [Conventional Commits](https://www.conventionalcommits.org/) and [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Features (Phase 2 — Scene Graph Completeness & Layout Primitives)

- **engine:** add `Anchor` / `Comment` nodes, `insert_before`, `remove_child`, and `set_attribute` / `get_attribute` to `RenderEngine`, `WidgetKind`, and `RenderCommand`.
- **widgets:** expand cross-platform and GTK4 FFI catalog (`Gtk4Sys` & `GtkLinuxBackend`) with `<ng-header-bar>`, `<ng-scroll>`, `<ng-grid>`, and `<ng-separator>`.
- **architecture:** extract scene graph diagnostic preview into `src/engine/preview.rs` and decouple `.ngb` syntax parsing into `src/runtime/parser.rs` (`Instruction` IR).
- **dx:** emit HTML layout attributes automatically from `build-bundle.mjs`, auto-compile missing `.ngb` bundles on `cargo run`, and add unified `pnpm run check` / `pnpm run format:all` scripts.

---

## [0.1.0] - 2026-11-08

### Features

- **dev:** add state-preserving live Hot Reload (`JsRuntime::hot_reload`), non-blocking 60 FPS `mtime` watcher in the native GTK4 loop, `--watch` mode in `build-bundle.mjs`, and `pnpm run dev` (`examples/counter-app/dev-server.mjs`).
- **icons:** support multi-resolution Angular `favicon.ico` (`16x16`, `32x32`, `48x48`) in `WindowIconAsset`, generate Freedesktop `hicolor` PNG pyramid (`16x16`..`256x256`), and wire Wayland/KDE Plasma `app_id` (`g_set_prgname`) + `GtkIconTheme` taskbar integration (`0af3563`).
- **angular:** replace mock setup with an official **Angular 21** CLI workspace (`@angular/core` 21.2, `CUSTOM_ELEMENTS_SCHEMA`, `signal()`) and a declarative template/CSS/event compiler (`build-bundle.mjs`) (`974b95b`).
- **engine:** implement dynamic Linux GTK4 native window driver (`Gtk4Sys` loading `libgtk-4.so.1`, `libgobject-2.0.so.0`, `libgio-2.0.so.0`, `libglib-2.0.so.0`), real `GtkCssProvider` styling, and reactive Signal/event dispatch in `JsRuntime` (`d535518`).
- **core:** initialize `ng-native-engine` with Generational `Arena`, runtime `BorrowState` checker, `CommandQueue`, `StyleRegistry`, and cross-platform `PlatformBackend` adapters (`GtkLinuxBackend`, `WinUiBackend`, `MacOsAppKitBackend`, `HeadlessBackend`) (`3642561`).

### Tooling & CI

- **tooling:** configure `oxfmt`, `oxlint`, `eslint` with `angular-eslint` and `eslint-plugin-oxlint`, and `.vscode/settings.json` format-on-save (`02a915d`).
- **ci:** configure GitHub Actions (`ubuntu-24.04`, Node 24, `pnpm`, `oxfmt`, `oxlint`, `eslint`, `rustfmt`, `clippy`, and `cargo test` with `HeadlessBackend`) (`dc94445`).
