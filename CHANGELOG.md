# Changelog

All notable changes and completed roadmap tasks for `ng-native-engine` are documented in this file.
This project adheres to [Conventional Commits](https://www.conventionalcommits.org/) and [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Features (Angular 22 Upgrade & Post-Phase 5 Hardening)

- **angular:** upgrade workspace and all `@ng-native/*` packages (`platform`, `forms`, `router`, `http`, `desktop`) to **Angular 22 (`22.2.5`)**, `@angular/build` `22.2.6` (powered by Rust `rolldown` `1.2.16` + `oxc-parser` `0.156.0` + `vite` `8.3.5`), and `typescript` `6.0.3` (removing deprecated `baseUrl` in `tsconfig.json` for TS 6/7 compatibility).
- **runtime:** synchronize `DesktopServices` directly inside `JsRuntime` (`theme` & `clipboard` opcodes), extract `src/runtime/bootstrap_js.rs`, and extract GTK4 CSS/attribute styling into `src/platforms/gtk4_styling.rs` (`8d4aab9`).

### Features (Phase 5 — `@ng-native/router`, `@ng-native/http`, & `@ng-native/desktop`)

- **router:** create `@ng-native/router` (`packages/router-native`) implementing DOM-free `NativePlatformLocation` (in-memory URL/state history stack, `pushState`, `replaceState`, `back`, `forward`, `onPopState`) and `provideNativeRouter()`.
- **http:** create `@ng-native/http` (`packages/http-native`) implementing `NativeHttpBackend` and `provideNativeHttpClient()`, compatible with `HttpClient`, `withInterceptors()`, and `httpResource()`.
- **desktop:** create `src/desktop/mod.rs` (`DesktopServices`) and `@ng-native/desktop` (`packages/desktop-native`) providing `NativeDesktopService` with reactive `theme` Signal (`'dark' | 'light'`), clipboard access, desktop notifications, and file dialogs.
- **platforms:** extract GTK4 C signal trampolines into `src/platforms/gtk4_callbacks.rs`, extract `QuickJsEngine` into `src/runtime/quickjs_engine.rs`, and wire live GTK4 widget getters (`gtk_editable_get_text`, `gtk_switch_get_active`, `gtk_check_button_get_active`, `gtk_range_get_value`, `gtk_drop_down_get_selected`, `gtk_string_list_new`) into `GtkLinuxBackend` (`8e467a7`).

### Features (Phase 4 — Interactive Controls & `@ng-native/forms`)

- **widgets:** add `<ng-entry>` (`GtkEntry`), `<ng-switch>` (`GtkSwitch`), `<ng-checkbox>` (`GtkCheckButton`), `<ng-slider>` (`GtkScale`), and `<ng-select>` (`GtkDropDown`) across `WidgetKind`, `RenderEngine` preview, `Gtk4Sys`, `GtkLinuxBackend`, `WinUiBackend`, and `MacOsAppKitBackend`.
- **forms:** create `@ng-native/forms` (`packages/forms-native`) providing `NativeTextValueAccessor`, `NativeBooleanValueAccessor`, and `NativeNumberValueAccessor` (`ControlValueAccessor` directives) for two-way reactive and template-driven Angular forms over `NativeRenderer2`.
- **events:** support typed `EventPayload` (`None`, `Text`, `Bool`, `Int`) across `EventRegistry`, `JsRuntime`, and `JsEngineBackend` (`__dispatchEvent`).

### Features (Phase 3 — Pluggable JS Engines & `@ng-native/platform`)

- **runtime:** add hexagonal `JsEngineBackend` trait (`src/runtime/js_engine.rs`) with dual adapters: Apple/WebKit **JavaScriptCore (`JscEngine`)** via isolated dynamic FFI (`src/runtime/jsc_sys.rs`, `libjavascriptcoregtk-6.0.so.1` 4-tier JIT) and portable **QuickJS (`QuickJsEngine`)**, selectable via `angular.json` (`"jsEngine": "jsc" | "quickjs"`).
- **platform:** create `@ng-native/platform` (`packages/platform-native`) implementing `NativeRenderer2`, `NativeRendererFactory2`, `NativeElementNode`, `provideNativePlatform()`, and `bootstrapNativeApplication()`.

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
