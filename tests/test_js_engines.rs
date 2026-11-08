use ng_native_engine::engine::RenderEngine;
use ng_native_engine::platforms::HeadlessBackend;
use ng_native_engine::runtime::JsRuntime;

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
fn quickjs_and_jsc_report_syntax_errors_deterministically() {
    for engine_kind in ["jsc", "quickjs"] {
        let mut engine = RenderEngine::new(HeadlessBackend::new());
        let mut runtime = JsRuntime::from_engine_config(engine_kind).unwrap();

        let bad_syntax = "function() { not valid javascript syntax !!!";
        let err = runtime.eval_js(&mut engine, bad_syntax);
        assert!(err.is_err(), "Engine '{engine_kind}' should reject bad JS");
    }
}
