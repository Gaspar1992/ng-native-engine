use ng_native_engine::engine::{EventPayload, RenderCommand, RenderEngine};
use ng_native_engine::platforms::HeadlessBackend;
use ng_native_engine::runtime::JsRuntime;

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
fn node_based_event_dispatch_resolves_js_handles_through_reverse_index() {
    let js_bundle = r#"
        __NG_NATIVE__.createElement("win", "ng-window");
        __NG_NATIVE__.createElement("entry", "ng-entry");
        __NG_NATIVE__.createElement("echo", "ng-label");
        __NG_NATIVE__.appendChild("win", "entry");
        __NG_NATIVE__.listen("entry", "input", function(evt) {
            __NG_NATIVE__.setText("echo", "Echo: " + evt.value);
        });
    "#;

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::from_engine_config("jsc").unwrap();
    runtime.eval_js(&mut engine, js_bundle).unwrap();

    let entry = runtime.resolve_var("entry").unwrap();
    runtime
        .trigger_node_event_with_payload(
            &mut engine,
            entry,
            "input",
            &EventPayload::Text(String::from("ReverseIndex")),
        )
        .unwrap();

    assert!(engine.backend().frames.iter().flatten().any(|cmd| matches!(
        cmd,
        RenderCommand::SetText { text, .. } if text == "Echo: ReverseIndex"
    )));

    // Destroyed handles must no longer route node-based events to JS listeners.
    runtime.eval_tick(&mut engine, "destroy entry").unwrap();
    assert!(runtime.resolve_var("entry").is_err());
}

#[test]
fn scene_graph_prevents_cycles_and_cleans_up_orphaned_listeners() {
    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::new();

    let script = r"
        create win ng-window
        create box1 ng-box
        create box2 ng-box
        append win box1
        append box1 box2
    ";
    runtime.eval_tick(&mut engine, script).unwrap();

    let win = runtime.resolve_var("win").unwrap();
    let box1 = runtime.resolve_var("box1").unwrap();
    let _box2 = runtime.resolve_var("box2").unwrap();

    assert!(engine.get_attribute(box1, "nonexistent").is_none());

    // Destroy intermediate container: child node can still be referenced by handle, but is unparented
    runtime.eval_tick(&mut engine, "destroy box1").unwrap();
    assert!(runtime.resolve_var("box1").is_err());
    assert!(runtime.resolve_var("box2").is_ok());

    let preview = engine.render_window_preview(win).unwrap();
    assert!(!preview.contains("box1"));
}
