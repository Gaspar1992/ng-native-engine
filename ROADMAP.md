# 🗺️ NgNative Engine — Architectural Roadmap to 100% Angular Compatibility

This document defines the deep technical analysis and phased engineering roadmap required to run **100% of the Angular 21+ ecosystem** (`@angular/core`, `@angular/common`, `@angular/forms`, `@angular/router`, `@angular/common/http`, `@angular/animations`, and `@angular/cdk`) natively on **Linux (GTK4 / Libadwaita)**, and progressively on **Windows (WinUI 3)** and **macOS (AppKit)**.

---

## 🧠 Deep Technical Gap Analysis: What Angular Needs Without a Browser

When `@angular/platform-browser` is removed, Angular's Ivy runtime still requires **seven foundational pillars** to operate at 100% parity:

### 1. Embedded JavaScript Engine + Unified Event Loop (`rquickjs` / V8 + `glib::MainContext`)

- **Why:** To execute real compiled Angular bundles (`main.js` output by `@angular/build:application`), the Rust binary must embed a spec-compliant ES2024+ JS engine (QuickJS-NG via `rquickjs` or V8 via `deno_core`).
- **Event Loop Unification:** Angular Zoneless change detection schedules UI refreshes via `queueMicrotask`, `Promise.then`, and `setTimeout` / `requestAnimationFrame`. The JS engine's job queue must be hooked directly into GTK4's `glib::MainContext` (`g_idle_add` / `g_timeout_add`), ensuring every microtask checkpoint flushes `RenderEngine::flush_frame()` on the main thread without race conditions.

### 2. Custom Angular Platform (`@ng-native/platform`) & `RendererFactory2`

- **Why:** Angular's component tree interacts with the host environment exclusively through `PlatformRef`, `RendererFactory2`, `Renderer2`, `Sanitizer`, `ErrorHandler`, and `ApplicationRef`.
- **What Must Be Implemented in TypeScript (`packages/platform-native`):**
  - `bootstrapNativeApplication(AppComponent, appConfig)`: Bootstraps Angular in **Zoneless mode** (`provideZonelessChangeDetection()`).
  - `NativeRendererFactory2` & `NativeRenderer2`: Implements all 19 methods of `Renderer2` (`createElement`, `createComment`, `createText`, `appendChild`, `insertBefore`, `removeChild`, `selectRootElement`, `parentNode`, `nextSibling`, `setAttribute`, `removeAttribute`, `addClass`, `removeClass`, `setStyle`, `removeStyle`, `setProperty`, `setValue`, `listen`) by calling the Rust FFI bridge (`globalThis.__NG_NATIVE__`) using generational `NodeId` handles.
  - **Anchor & Comment Nodes (`createComment` / `insertBefore`):** Essential for Angular's built-in control flow (`@if`, `@for`, `@switch`, `@defer`) and `<ng-container>` / `ViewContainerRef`, which rely on invisible anchor nodes in the scene graph to insert and remove dynamic views at exact child indices.

### 3. Complete Template Control Flow, Lifecycle, & Content Projection

- **Structural Directives & Control Flow (`@if`, `@for`, `@switch`, `@defer`):** Requires `insertBefore(parent, newChild, refChild)`, `removeChild(parent, oldChild)`, and virtual comment/anchor nodes in `Arena`.
- **Component Hierarchy & Content Projection (`<ng-content>`):** Nested custom components (`<app-header>`, `<app-card>`) must act as logical pass-through containers or `GtkBox` wrappers in the Rust scene graph while preserving multi-slot projection.
- **Lifecycle & Queries:** `ngOnInit`, `ngOnDestroy`, `afterNextRender`, `afterRenderEffect`, `viewChild()`, `viewChildren()`, `contentChild()`, and `ElementRef<NativeElementProxy>`.

### 4. Forms Ecosystem (`@angular/forms` + Signal Forms)

- **Why:** Angular `FormsModule` and `ReactiveFormsModule` (`FormControl`, `FormGroup`, `Validators`) do not depend on HTML `<input>` directly — they depend on **`ControlValueAccessor` (`CVA`)** directives.
- **What Must Be Implemented:**
  - Native input widgets in Rust/GTK4: `<ng-entry>` (`GtkEntry` / `GtkPasswordEntry`), `<ng-switch>` (`GtkSwitch`), `<ng-check>` (`GtkCheckButton`), `<ng-Scale>` (`GtkScale`), `<ng-dropdown>` (`GtkDropDown`), `<ng-text-view>` (`GtkTextView`).
  - Default `ControlValueAccessor` directives in `@ng-native/forms` that listen to native GTK4 signals (`notify::text`, `notify::active`, `notify::selected`) and call `onChange` / `onTouched`.

### 5. Navigation & Multi-Window Routing (`@angular/router`)

- **Why:** `@angular/router` defaults to `BrowserPlatformLocation` (`window.location` / `history.pushState`).
- **What Must Be Implemented:**
  - `MemoryPlatformLocation` / `NativePlatformLocation`: An in-memory URL state machine backed by Rust so `RouterLink`, `<router-outlet>`, route guards (`canActivate`, `canDeactivate`), lazy-loaded routes (`loadComponent`, `loadChildren`), and `ActivatedRoute` signals work 100% unchanged.
  - Native Linux navigation integration with `AdwNavigationView` / `AdwNavigationSplitView` / `GtkStack` transitions.

### 6. Network, Storage, & System I/O (`@angular/common/http` + OS Bridges)

- **Why:** `HttpClient` delegates network calls to the abstract `HttpBackend` token (which in browsers uses `HttpXhrBackend` or `FetchBackend`).
- **What Must Be Implemented:**
  - `NativeHttpBackend` (or global `fetch` polyfill backed by Rust/libgio/curl): Enables `HttpClient`, `httpResource()`, `rxResource()`, and all functional `HttpInterceptorFn` pipelines to execute natively off the UI thread and resolve back onto the main loop.
  - **Linux Desktop Integration (Freedesktop / XDG Portals / DBus):** Native file chooser dialogs (`GtkFileDialog`), system notifications (`GNotification`), clipboard (`GdkClipboard`), system tray (StatusNotifierItem), dark/light theme reactivity (`AdwStyleManager`), and secure keyring storage (`libsecret`).

### 7. Layout, Styling, & Animation System

- **Layout:** Support full GTK4 layout attributes (`orientation`, `spacing`, `hexpand`, `vexpand`, `halign`, `valign`, `margin-*`) plus `GtkGrid`, `GtkPaned`, `GtkScrolledWindow`, `AdwHeaderBar`, `AdwToolbarView`, and `AdwClamp`.
- **Styling:** Full component-scoped CSS encapsulation (`ViewEncapsulation.Emulated` via `.nghost-*` / `.ngcontent-*` class rewriting into `GtkCssProvider`) and CSS custom properties (`--var`).
- **Animations:** Bridge `@angular/animations` and CSS transitions into GTK4's frame clock (`GdkFrameClock`) and native `AdwTimedAnimation` / `AdwSpringAnimation`.

---

## 📋 Phased Implementation Roadmap

### Phase 1: Core Engine Foundations & Native GTK4 MVP (Completed ✅)

- [x] Generational `Arena` with `NodeId { index, generation }` and runtime `BorrowState` (`Shared` / `Exclusive`) checker.
- [x] Frame-batched `CommandQueue` and OS-agnostic `RenderCommand` pipeline.
- [x] SOLID `PlatformBackend` trait + `AnyPlatformBackend` factory (`LinuxGtkBackend`, `WinUiBackend`, `MacOsAppKitBackend`, `HeadlessBackend`).
- [x] Isolated C FFI dynamic loader (`Gtk4Sys`) for `libgtk-4.so.1`, `libgobject-2.0.so.0`, `libgio-2.0.so.0`, and `libglib-2.0.so.0`.
- [x] Official Angular 21 workspace (`@angular/core`, `@angular/cli`, `oxfmt`, `oxlint`, `angular-eslint`, `vitest`).
- [x] Multi-resolution `.ico` parser (`WindowIconAsset`) + Freedesktop `hicolor` (`16x16`..`256x256`) & Wayland/KDE Plasma taskbar integration.
- [x] Declarative template compiler (`build-bundle.mjs`) supporting initial Signals (`signal()`, `.update()`, `.set()`) and `(click)` events.

### Phase 2: Scene Graph Completeness & Layout Primitives (Next Up 🚧)

- [ ] **Scene Graph Anchor & Sibling Operations:** Add `Comment` / `Anchor` nodes, `insert_before(parent, new_child, ref_child)`, and `remove_child(parent, child)` to `Arena`, `WidgetKind`, `RenderCommand`, and `GtkLinuxBackend`.
- [ ] **Attribute & Property Pipeline (`setAttribute` / `setProperty`):** Support layout attributes directly from Angular templates (`orientation="horizontal"`, `spacing="16"`, `hexpand="true"`, `vexpand="true"`, `halign="center"`, `valign="center"`, `margin="12"`).
- [ ] **Expanded GTK4 / Libadwaita Container Catalog:**
  - [ ] `<ng-header-bar>` (`GtkHeaderBar` / `AdwHeaderBar`)
  - [ ] `<ng-scroll>` (`GtkScrolledWindow`)
  - [ ] `<ng-grid>` (`GtkGrid`)
  - [ ] `<ng-Separator>` (`GtkSeparator`)

### Phase 3: Embedded JS Engine & Custom `@ng-native/platform` (100% Ivy Runtime)

- [ ] **Embed QuickJS-NG (`rquickjs`) in Rust:** Execute standard ES2022 bundles produced by `@angular/build:application` directly inside the Rust process.
- [ ] **Bridge `glib::MainContext` + JS Microtask Queue:** Wire `setTimeout`, `clearTimeout`, `queueMicrotask`, and `console.*` globals in Rust so `provideZonelessChangeDetection()` runs natively.
- [ ] **Create `@ng-native/platform` TypeScript Package:**
  - [ ] Implement `NativeRendererFactory2` and `NativeRenderer2` over `globalThis.__NG_NATIVE__`.
  - [ ] Support Angular `@if`, `@for`, `@switch`, and `@defer` blocks using `insert_before` and anchor nodes.
  - [ ] Support nested `@Component` composition, `input()`, `output()`, `model()`, and `<ng-content>` projection.

### Phase 4: Interactive Controls & `@angular/forms` Compatibility

- [ ] **Native Input Widgets in `GtkLinuxBackend`:**
  - [ ] `<ng-entry>` (`GtkEntry` with `changed` / `notify::text` signals)
  - [ ] `<ng-switch>` (`GtkSwitch` with `state-set` / `notify::active` signals)
  - [ ] `<ng-checkbox>` (`GtkCheckButton` with `toggled` signal)
  - [ ] `<ng-slider>` (`GtkScale` with `value-changed` signal)
  - [ ] `<ng-select>` (`GtkDropDown` with `notify::selected` signal)
- [ ] **`@ng-native/forms` ControlValueAccessors:**
  - [ ] Implement `DefaultValueAccessor`, `CheckboxControlValueAccessor`, and `NumberValueAccessor` for native widgets.
  - [ ] Full compatibility with `FormsModule` (`[(ngModel)]`), `ReactiveFormsModule` (`formControl`, `formGroup`, `Validators`), and Angular Signal Forms.

### Phase 5: `@angular/router`, `@angular/common/http`, & Desktop OS APIs

- [ ] **`@ng-native/router` (`NativePlatformLocation`):**
  - [ ] Support `<router-outlet>`, `routerLink`, route guards, resolvers, and lazy loading without `window.location`.
- [ ] **`@ng-native/http` (`NativeHttpBackend`):**
  - [ ] Implement async HTTP client bridge in Rust (`GIO` / `reqwest`) feeding `HttpClient`, `HttpInterceptorFn`, and `httpResource()`.
- [ ] **Linux Desktop System Services (`@ng-native/desktop`):**
  - [ ] Native File Dialogs (`GtkFileDialog` open/save).
  - [ ] Desktop Notifications (`GNotification`) & System Clipboard (`GdkClipboard`).
  - [ ] System Dark/Light Theme Signal (`themeSignal`) synchronized with `GtkSettings` / `AdwStyleManager`.

### Phase 6: Packaging & Cross-Platform Native Backends (Windows & macOS)

- [ ] **Angular CLI Builder (`@ng-native/builders`):** Single-command `ng build` / `ng serve` integration with hot-reload.
- [ ] **Linux Distribution Bundler:** Generate self-contained `.deb`, `.rpm`, `AppImage`, and `Flatpak` packages with desktop entries and `hicolor` icon sets.
- [ ] **Windows Native Backend (`WinUiBackend`):** Connect FFI to Win32 / WinUI 3 (`CreateWindowExW` / XAML Islands).
- [ ] **macOS Native Backend (`MacOsAppKitBackend`):** Connect Objective-C runtime FFI to `NSApplication`, `NSWindow`, `NSStackView`, `NSTextField`, and `NSButton`.
