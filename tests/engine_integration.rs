use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::{EngineError, EventPayload, RenderCommand, RenderEngine};
use ng_native_engine::packager::stage_linux_distribution;
use ng_native_engine::platforms::{
    AnyPlatformBackend, HeadlessBackend, MacOsAppKitBackend, PlatformBackend, WinUiBackend,
};
use ng_native_engine::runtime::JsRuntime;
use std::fs;

#[test]
fn full_mvp_pipeline_with_headless_backend() {
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    assert_eq!(cfg.project_name, "ng-linux-counter");
    assert_eq!(cfg.target_platform, "linux-gtk4");

    let bundle_path = format!("{}/main.ngb", cfg.output_path);
    let script = fs::read_to_string(bundle_path).unwrap();

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::new();

    // Tick 1: Mount Angular component (signals + styles + template + icon + declarative events)
    runtime.eval_tick(&mut engine, &script).unwrap();
    assert_eq!(engine.backend().frames.len(), 1);
    assert_eq!(engine.backend().frames[0].len(), 20);

    let win = runtime.resolve_var("win").unwrap();
    let hello = runtime.resolve_var("hello").unwrap();
    let btn = runtime.resolve_var("btn").unwrap();
    let lbl = runtime.resolve_var("lbl").unwrap();

    let preview = engine.render_window_preview(win).unwrap();
    assert!(preview.contains("NgNative - Hola Mundo"));
    assert!(preview.contains("[ICO:16x16, 32x32, 48x48]"));
    assert!(preview.contains("Hola Mundo desde Angular Native!"));
    assert!(preview.contains(".hero-title"));

    assert!(engine.backend().frames[0].iter().any(|cmd| matches!(
        cmd,
        RenderCommand::SetText { id, text }
            if *id == hello && text.starts_with("Hola Mundo desde Angular Native!")
    )));

    // Tick 2: Declarative `(click)="increment()"` updates signal `count` -> `Count: 1`
    runtime
        .trigger_event(&mut engine, "btn", "clicked")
        .unwrap();
    assert_eq!(engine.backend().frames.len(), 2);
    assert_eq!(
        engine.backend().frames[1],
        vec![RenderCommand::SetText {
            id: lbl,
            text: String::from("Count: 1"),
        }]
    );

    // Tick 3: Second click increments signal `count` -> `Count: 2`
    runtime
        .trigger_event(&mut engine, "btn", "clicked")
        .unwrap();
    assert_eq!(engine.backend().frames.len(), 3);
    assert_eq!(
        engine.backend().frames[2],
        vec![RenderCommand::SetText {
            id: lbl,
            text: String::from("Count: 2"),
        }]
    );

    // Verify borrow conflict protection during an active event dispatch
    engine
        .emit_event(btn, "clicked", |eng, _| {
            assert!(matches!(
                eng.destroy_node(btn),
                Err(EngineError::BorrowConflict { .. })
            ));
        })
        .unwrap();
}

#[test]
fn cross_platform_factory_supports_linux_windows_and_macos() {
    let linux = AnyPlatformBackend::from_target("linux-gtk4").unwrap();
    assert_eq!(linux.map_native_widget("ng-window"), "AdwApplicationWindow");

    let windows = AnyPlatformBackend::from_target("windows-winui").unwrap();
    assert_eq!(
        windows.map_native_widget("ng-window"),
        "Microsoft.UI.Xaml.Window"
    );

    let macos = AnyPlatformBackend::from_target("macos-appkit").unwrap();
    assert_eq!(macos.map_native_widget("ng-window"), "NSWindow");
}

#[test]
fn hot_reload_preserves_signal_state_and_updates_template_and_styles() {
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    let bundle_path = format!("{}/main.ngb", cfg.output_path);
    let initial_script = fs::read_to_string(bundle_path).unwrap();

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::new();

    // 1. Initial mount and increment signal twice (`count` = 2)
    runtime.eval_tick(&mut engine, &initial_script).unwrap();
    runtime
        .trigger_event(&mut engine, "btn", "clicked")
        .unwrap();
    runtime
        .trigger_event(&mut engine, "btn", "clicked")
        .unwrap();

    let win = runtime.resolve_var("win").unwrap();
    let before_preview = engine.render_window_preview(win).unwrap();
    assert!(before_preview.contains("Count: 2"));

    // 2. Simulate saving updated `app.html` and `app.css` while window is open
    let updated_script = initial_script
        .lines()
        .map(|line| {
            if line.starts_with("text hello ") {
                "text hello Hola Mundo en Vivo (HMR)!"
            } else if line.starts_with("bind_text lbl count ") {
                "bind_text lbl count Contador HMR: {}"
            } else if line.starts_with("style hero-title ") {
                "style hero-title none #22c55e 8 0"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    runtime.hot_reload(&mut engine, &updated_script).unwrap();

    // 3. Verify signal `count` stayed at 2 while template text and CSS updated in-place
    let after_preview = engine.render_window_preview(win).unwrap();
    assert!(after_preview.contains("Hola Mundo en Vivo (HMR)!"));
    assert!(after_preview.contains("Contador HMR: 2"));
    assert!(after_preview.contains(".hero-title(bg:transparent, fg:#22c55e, pad:8px)"));

    // 4. Verify event listener re-bound cleanly after HMR (`count` -> 3)
    runtime
        .trigger_event(&mut engine, "btn", "clicked")
        .unwrap();
    let next_preview = engine.render_window_preview(win).unwrap();
    assert!(next_preview.contains("Contador HMR: 3"));
}

#[test]
fn phase2_scene_graph_containers_attributes_and_anchor_operations() {
    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::new();

    let script = r"
        create win ng-window
        text win NgNative Phase 2
        create hdr ng-header-bar
        text hdr Panel Principal
        attr hdr subtitle Libadwaita + GTK4
        create scroll ng-scroll
        attr scroll min-height 320px
        create root_box ng-box
        attr root_box orientation vertical
        attr root_box spacing 16px
        attr root_box hexpand true
        attr root_box vexpand true
        attr root_box halign center
        attr root_box valign start
        attr root_box margin 12px
        create grid ng-grid
        attr grid row-spacing 8px
        attr grid column-spacing 12px
        create sep ng-separator
        attr sep orientation horizontal
        comment anchor_if @if(showExtra)
        create lbl_fixed ng-label
        text lbl_fixed Elemento Fijo
        append win hdr
        append win scroll
        append scroll root_box
        append root_box grid
        append root_box sep
        append root_box anchor_if
        append root_box lbl_fixed
    ";

    runtime.eval_tick(&mut engine, script).unwrap();

    let win = runtime.resolve_var("win").unwrap();
    let hdr = runtime.resolve_var("hdr").unwrap();
    let scroll = runtime.resolve_var("scroll").unwrap();
    let root_box = runtime.resolve_var("root_box").unwrap();

    assert_eq!(
        engine.get_attribute(hdr, "subtitle"),
        Some("Libadwaita + GTK4")
    );
    assert_eq!(engine.get_attribute(scroll, "min-height"), Some("320px"));
    assert_eq!(engine.get_attribute(root_box, "halign"), Some("center"));
    assert_eq!(engine.get_attribute(root_box, "margin"), Some("12px"));

    // Dynamically insert a node before the `@if` comment anchor, then remove it
    let dynamic_insert = r"
        create lbl_dyn ng-label
        text lbl_dyn Insertado via @if
        insert_before root_box lbl_dyn anchor_if
    ";
    runtime.eval_tick(&mut engine, dynamic_insert).unwrap();

    let preview_with_dyn = engine.render_window_preview(win).unwrap();
    assert!(preview_with_dyn.contains("[HeaderBar: Panel Principal — Libadwaita + GTK4]"));
    assert!(preview_with_dyn.contains("[Grid row-gap:8px col-gap:12px]"));
    assert!(preview_with_dyn.contains("Insertado via @if"));

    let dyn_idx = preview_with_dyn.find("Insertado via @if").unwrap();
    let fixed_idx = preview_with_dyn.find("Elemento Fijo").unwrap();
    assert!(
        dyn_idx < fixed_idx,
        "Expected dynamically inserted node before anchor to render ahead of trailing sibling"
    );

    runtime
        .eval_tick(&mut engine, "remove_child root_box lbl_dyn")
        .unwrap();
    let preview_after_remove = engine.render_window_preview(win).unwrap();
    assert!(!preview_after_remove.contains("Insertado via @if"));
    assert!(preview_after_remove.contains("Elemento Fijo"));
}

#[test]
fn dual_js_engines_jsc_and_quickjs_drive_render_engine_via_ng_native_bridge() {
    let js_bundle = r#"
        __NG_NATIVE__.declareSignal("clicks", 5);
        __NG_NATIVE__.createElement("win", "ng-window");
        __NG_NATIVE__.setText("win", "JS Bridge Window");
        __NG_NATIVE__.createElement("box", "ng-box");
        __NG_NATIVE__.createElement("lbl", "ng-label");
        __NG_NATIVE__.setText("lbl", "Clicks: 5");
        __NG_NATIVE__.createElement("btn", "ng-button");
        __NG_NATIVE__.setText("btn", "+1");
        __NG_NATIVE__.appendChild("box", "lbl");
        __NG_NATIVE__.appendChild("box", "btn");
        __NG_NATIVE__.appendChild("win", "box");
        __NG_NATIVE__.bindText("lbl", "clicks", "Clicks: {}");
        __NG_NATIVE__.onAdd("btn", "clicked", "clicks", 1);
    "#;

    for engine_kind in ["jsc", "quickjs"] {
        let mut engine = RenderEngine::new(HeadlessBackend::new());
        let mut runtime = JsRuntime::from_engine_config(engine_kind).unwrap();

        runtime.eval_js(&mut engine, js_bundle).unwrap();
        let win = runtime.resolve_var("win").unwrap();
        let preview = engine.render_window_preview(win).unwrap();
        assert!(preview.contains("JS Bridge Window"));
        assert!(preview.contains("Clicks: 5"));

        runtime
            .trigger_event(&mut engine, "btn", "clicked")
            .unwrap();
        let updated_preview = engine.render_window_preview(win).unwrap();
        assert!(updated_preview.contains("Clicks: 6"));
    }
}

#[test]
fn phase4_interactive_form_controls_and_typed_event_payloads() {
    let js_bundle = r#"
        __NG_NATIVE__.createElement("win", "ng-window");
        __NG_NATIVE__.setText("win", "Form Controls Window");
        __NG_NATIVE__.createElement("box", "ng-box");
        __NG_NATIVE__.createElement("entry", "ng-entry");
        __NG_NATIVE__.setAttribute("entry", "placeholder", "Username...");
        __NG_NATIVE__.setText("entry", "InitialUser");
        __NG_NATIVE__.createElement("sw", "ng-switch");
        __NG_NATIVE__.setAttribute("sw", "active", "false");
        __NG_NATIVE__.createElement("chk", "ng-checkbox");
        __NG_NATIVE__.setText("chk", "Accept license");
        __NG_NATIVE__.setAttribute("chk", "active", "true");
        __NG_NATIVE__.createElement("slider", "ng-slider");
        __NG_NATIVE__.setAttribute("slider", "min", "0");
        __NG_NATIVE__.setAttribute("slider", "max", "100");
        __NG_NATIVE__.setAttribute("slider", "value", "45");
        __NG_NATIVE__.createElement("sel", "ng-select");
        __NG_NATIVE__.setAttribute("sel", "options", "Gtk4,WinUI3,AppKit");
        __NG_NATIVE__.setAttribute("sel", "selected", "0");
        __NG_NATIVE__.createElement("summary", "ng-label");
        __NG_NATIVE__.setText("summary", "User: InitialUser | Dark: false | Vol: 45");
        __NG_NATIVE__.appendChild("box", "entry");
        __NG_NATIVE__.appendChild("box", "sw");
        __NG_NATIVE__.appendChild("box", "chk");
        __NG_NATIVE__.appendChild("box", "slider");
        __NG_NATIVE__.appendChild("box", "sel");
        __NG_NATIVE__.appendChild("box", "summary");
        __NG_NATIVE__.appendChild("win", "box");

        globalThis.__formState = { user: "InitialUser", dark: false, vol: 45 };
        function syncSummary() {
            const s = globalThis.__formState;
            __NG_NATIVE__.setText("summary", "User: " + s.user + " | Dark: " + s.dark + " | Vol: " + s.vol);
        }

        __NG_NATIVE__.listen("entry", "input", function(evt) {
            globalThis.__formState.user = evt.value;
            __NG_NATIVE__.setText("entry", evt.value);
            syncSummary();
        });
        __NG_NATIVE__.listen("sw", "toggled", function(evt) {
            globalThis.__formState.dark = evt.checked;
            __NG_NATIVE__.setAttribute("sw", "active", evt.checked ? "true" : "false");
            syncSummary();
        });
        __NG_NATIVE__.listen("slider", "value-changed", function(evt) {
            globalThis.__formState.vol = evt.value;
            __NG_NATIVE__.setAttribute("slider", "value", String(evt.value));
            syncSummary();
        });
    "#;

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::from_engine_config("jsc").unwrap();
    runtime.eval_js(&mut engine, js_bundle).unwrap();

    let win = runtime.resolve_var("win").unwrap();
    let initial_preview = engine.render_window_preview(win).unwrap();
    assert!(initial_preview.contains("[Entry: \"InitialUser\"]"));
    assert!(initial_preview.contains("[Switch: OFF]"));
    assert!(initial_preview.contains("[x] Accept license"));
    assert!(initial_preview.contains("[Slider: 45 (0..100)]"));
    assert!(initial_preview.contains("[Select: \"Gtk4\" (0)]"));

    runtime
        .trigger_event_with_payload(
            &mut engine,
            "entry",
            "input",
            &EventPayload::Text(String::from("GraceHopper")),
        )
        .unwrap();
    runtime
        .trigger_event_with_payload(&mut engine, "sw", "toggled", &EventPayload::Bool(true))
        .unwrap();
    runtime
        .trigger_event_with_payload(
            &mut engine,
            "slider",
            "value-changed",
            &EventPayload::Int(90),
        )
        .unwrap();

    let updated_preview = engine.render_window_preview(win).unwrap();
    assert!(updated_preview.contains("User: GraceHopper | Dark: true | Vol: 90"));
    assert!(updated_preview.contains("[Entry: \"GraceHopper\"]"));
    assert!(updated_preview.contains("[Switch: ON]"));
    assert!(updated_preview.contains("[Slider: 90 (0..100)]"));
}

#[test]
fn phase5_desktop_services_and_native_http_bridge() {
    let js_bundle = r#"
        __NG_NATIVE__.registerHttpEndpoint("GET", "https://api.ng-native.dev/version", 200, '{"version":"0.5.0"}');
        const res = __NG_NATIVE__.httpRequest("GET", "https://api.ng-native.dev/version");
        const parsed = JSON.parse(res.body);
        __NG_NATIVE__.setSystemTheme("light");
        __NG_NATIVE__.setClipboard("Clip:" + parsed.version);
        const clip = __NG_NATIVE__.getClipboard();
        const theme = __NG_NATIVE__.getSystemTheme();
        const opened = __NG_NATIVE__.openFileDialog("Open", "/etc/hostname");

        __NG_NATIVE__.createElement("win", "ng-window");
        __NG_NATIVE__.setText("win", "Phase 5 Desktop Window");
        __NG_NATIVE__.createElement("lbl", "ng-label");
        __NG_NATIVE__.setText("lbl", `Theme:${theme} | ${clip} | File:${opened}`);
        __NG_NATIVE__.appendChild("win", "lbl");
    "#;

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::from_engine_config("jsc").unwrap();
    runtime.eval_js(&mut engine, js_bundle).unwrap();

    let win = runtime.resolve_var("win").unwrap();
    let preview = engine.render_window_preview(win).unwrap();
    assert!(preview.contains("Theme:light | Clip:0.5.0 | File:/etc/hostname"));
    assert_eq!(runtime.desktop().theme().as_str(), "light");
    assert_eq!(runtime.desktop().clipboard(), "Clip:0.5.0");
}

#[test]
fn phase6_linux_distribution_packager_and_winui_appkit_stateful_backends() {
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    let artifacts = stage_linux_distribution(&cfg, "0.2.0").unwrap();
    assert_eq!(artifacts.len(), 4);
    for path in &artifacts {
        assert!(
            fs::metadata(path).is_ok(),
            "Missing staged artifact: {path}"
        );
    }

    let script = r"
        style card #242424 #ffffff 16 12
        create win ng-window
        text win CrossPlatform App
        icon win ng-linux-counter examples/counter-app/public/favicon.ico
        create box ng-box
        add_class box card
        attr box spacing 12px
        create entry ng-entry
        text entry NativeText
        append box entry
        append win box
    ";

    let mut winui_engine = RenderEngine::new(WinUiBackend::new());
    let mut winui_rt = JsRuntime::new();
    winui_rt.eval_tick(&mut winui_engine, script).unwrap();
    let win_id = winui_rt.resolve_var("win").unwrap();
    let box_id = winui_rt.resolve_var("box").unwrap();
    let entry_id = winui_rt.resolve_var("entry").unwrap();
    assert_eq!(winui_engine.backend().control_count(), 3);
    assert_eq!(winui_engine.backend().scene_state().style_count(), 1);
    assert_eq!(
        winui_engine
            .backend()
            .scene_state()
            .get_window_icon(win_id)
            .unwrap()
            .app_id,
        "ng-linux-counter"
    );
    assert_eq!(
        winui_engine
            .backend()
            .get_control(box_id)
            .unwrap()
            .style_classes
            .len(),
        1
    );
    assert_eq!(
        winui_engine
            .backend()
            .get_control(entry_id)
            .unwrap()
            .native_class,
        "Microsoft.UI.Xaml.Controls.TextBox"
    );
    assert_eq!(
        winui_engine.backend().get_control(entry_id).unwrap().text,
        "NativeText"
    );

    let mut appkit_engine = RenderEngine::new(MacOsAppKitBackend::new());
    let mut appkit_rt = JsRuntime::new();
    appkit_rt.eval_tick(&mut appkit_engine, script).unwrap();
    let mac_entry_id = appkit_rt.resolve_var("entry").unwrap();
    assert_eq!(appkit_engine.backend().view_count(), 3);
    assert_eq!(appkit_engine.backend().scene_state().style_count(), 1);
    assert_eq!(
        appkit_engine
            .backend()
            .get_view(mac_entry_id)
            .unwrap()
            .native_class,
        "NSTextFieldEditable"
    );
    assert_eq!(
        appkit_engine.backend().get_view(mac_entry_id).unwrap().text,
        "NativeText"
    );
}
