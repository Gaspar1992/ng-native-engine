use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::{EngineError, RenderCommand, RenderEngine};
use ng_native_engine::platforms::{AnyPlatformBackend, HeadlessBackend, PlatformBackend};
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

    // Tick 1: Mount Angular component (signals + styles + template + declarative events)
    runtime.eval_tick(&mut engine, &script).unwrap();
    assert_eq!(engine.backend().frames.len(), 1);
    assert_eq!(engine.backend().frames[0].len(), 19);

    let win = runtime.resolve_var("win").unwrap();
    let hello = runtime.resolve_var("hello").unwrap();
    let btn = runtime.resolve_var("btn").unwrap();
    let lbl = runtime.resolve_var("lbl").unwrap();

    let preview = engine.render_window_preview(win).unwrap();
    assert!(preview.contains("NgNative - Hola Mundo"));
    assert!(preview.contains("Hola Mundo desde Angular Native!"));
    assert!(preview.contains(".hero-title"));

    assert!(
        engine.backend().frames[0].contains(&RenderCommand::SetText {
            id: hello,
            text: String::from("Hola Mundo desde Angular Native!"),
        })
    );

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
