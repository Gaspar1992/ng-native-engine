mod common;

use common::load_or_build_test_bundle;
use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::{EngineError, RenderCommand, RenderEngine};
use ng_native_engine::ipc::{HmrIpcClient, HmrIpcServer, HmrMessage, HmrResponse};
use ng_native_engine::platforms::HeadlessBackend;
use ng_native_engine::runtime::JsRuntime;

#[test]
fn full_mvp_pipeline_with_headless_backend() {
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    assert_eq!(cfg.project_name, "ng-linux-counter");
    assert_eq!(cfg.target_platform, "linux-gtk4");

    let script = load_or_build_test_bundle(&cfg.output_path);

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::new();

    // Tick 1: Mount Angular component (signals + styles + template + icon + declarative events)
    runtime.eval_tick(&mut engine, &script).unwrap();
    assert_eq!(engine.backend().frames.len(), 1);
    assert_eq!(engine.backend().frames[0].len(), 143);

    let win = runtime.resolve_var("win").unwrap();
    let hello = runtime.resolve_var("hello").unwrap();
    let btn = runtime.resolve_var("btn").unwrap();
    let lbl = runtime.resolve_var("lbl").unwrap();

    let preview = engine.render_window_preview(win).unwrap();
    assert!(preview.contains("NgNative - Hola Mundo"));
    assert!(preview.contains("[ICO:16x16, 32x32, 48x48]"));
    assert!(
        preview.contains("[HeaderBar: NgNative - Hola Mundo — Angular 22 + GTK4 / Libadwaita]")
    );
    assert!(preview.contains("Hola Mundo desde Angular Native!"));
    assert!(preview.contains(".hero-title"));
    assert!(preview.contains(
        ".hero-title(bg:transparent, fg:#78aeed, pad:8px, font-size:22px, font-weight:800)"
    ));
    assert!(preview.contains(".main-card(bg:#242424, fg:#ffffff, pad:16px, radius:14px)"));
    assert!(preview.contains("[Entry: \"ng-native-dev\"]"));
    assert!(preview.contains("[Switch: ON]"));
    assert!(preview.contains("[x] Enable native JIT"));
    assert!(preview.contains("[Slider: 60 (30..144)]"));
    assert!(preview.contains("[Select: \"Linux GTK4\" (0)]"));

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

    // Tick 4: Decrement button `(click)="decrement()"` -> `Count: 1`
    runtime
        .trigger_event(&mut engine, "btn_dec", "clicked")
        .unwrap();
    assert_eq!(
        engine.backend().frames[3],
        vec![RenderCommand::SetText {
            id: lbl,
            text: String::from("Count: 1"),
        }]
    );

    // Tick 5: Reset button `(click)="reset()"` -> `Count: 0`
    runtime
        .trigger_event(&mut engine, "btn_reset", "clicked")
        .unwrap();
    assert_eq!(
        engine.backend().frames[4],
        vec![RenderCommand::SetText {
            id: lbl,
            text: String::from("Count: 0"),
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
fn hot_reload_preserves_signal_state_and_updates_template_and_styles() {
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    let initial_script = load_or_build_test_bundle(&cfg.output_path);

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
fn hmr_ipc_server_applies_live_bundle_reloads_and_eval_to_scene_graph() {
    let socket_path = format!("/tmp/ng-native-pipeline-hmr-{}.sock", std::process::id());
    let mut hmr_server = HmrIpcServer::bind(&socket_path).unwrap();

    let initial_script = r"
        create win ng-window
        text win InitialApp
        create lbl ng-label
        text lbl Count: 10
        bind_text lbl count Count: {}
        signal count 10
        append win lbl
    ";

    let mut engine = RenderEngine::new(HeadlessBackend::new());
    let mut runtime = JsRuntime::from_engine_config("jsc").unwrap();
    runtime.eval_tick(&mut engine, initial_script).unwrap();

    let win = runtime.resolve_var("win").unwrap();
    let initial_preview = engine.render_window_preview(win).unwrap();
    assert!(initial_preview.contains("InitialApp"));
    assert!(initial_preview.contains("Count: 10"));

    let mut client = HmrIpcClient::connect(&socket_path).unwrap();
    let updated_bundle = r"
        create win ng-window
        text win HmrUpdatedApp
        create lbl ng-label
        text lbl Count: 0
        bind_text lbl count Updated Count: {}
        signal count 10
        append win lbl
    ";
    client.send_reload(updated_bundle).unwrap();

    let mut messages = Vec::new();
    for _ in 0..10 {
        messages = hmr_server.poll_messages();
        if !messages.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(messages.len(), 1);
    let (cid, msg) = &messages[0];
    if let HmrMessage::Reload { script } = msg {
        runtime.hot_reload(&mut engine, script).unwrap();
        hmr_server.send_response(*cid, &HmrResponse::Ok { reloaded: true });
    }

    let updated_preview = engine.render_window_preview(win).unwrap();
    assert!(updated_preview.contains("HmrUpdatedApp"));
    assert!(updated_preview.contains("Updated Count: 10"));

    let resp = client.read_response().unwrap();
    assert_eq!(resp, r#"{"status":"ok","reloaded":true}"#);

    client
        .send_eval(r#"__NG_NATIVE__.setText("lbl", "EvalOverIPC");"#)
        .unwrap();

    let mut eval_msgs = Vec::new();
    for _ in 0..10 {
        eval_msgs = hmr_server.poll_messages();
        if !eval_msgs.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(eval_msgs.len(), 1);
    if let HmrMessage::Eval { code } = &eval_msgs[0].1 {
        let res = runtime.eval_js(&mut engine, code).unwrap();
        hmr_server.send_response(eval_msgs[0].0, &HmrResponse::EvalResult(res));
    }

    let after_eval_preview = engine.render_window_preview(win).unwrap();
    assert!(after_eval_preview.contains("EvalOverIPC"));
}

#[test]
fn hmr_ipc_server_handles_client_disconnect_and_malformed_payloads() {
    let socket_path = format!(
        "/tmp/ng-native-pipeline-malformed-{}.sock",
        std::process::id()
    );
    let mut hmr_server = HmrIpcServer::bind(&socket_path).unwrap();

    let mut client = HmrIpcClient::connect(&socket_path).unwrap();
    client.send_ping().unwrap();

    let mut messages = Vec::new();
    for _ in 0..10 {
        messages = hmr_server.poll_messages();
        if !messages.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(messages.len(), 1);

    drop(client);
    // Poll again after client disconnects: server must prune dead streams cleanly
    let after_disconnect = hmr_server.poll_messages();
    assert_eq!(after_disconnect, []);
}
