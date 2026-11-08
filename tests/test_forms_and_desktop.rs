use ng_native_engine::engine::{EventPayload, RenderEngine};
use ng_native_engine::platforms::HeadlessBackend;
use ng_native_engine::runtime::JsRuntime;

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
