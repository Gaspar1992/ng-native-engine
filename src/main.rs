use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::RenderEngine;
use ng_native_engine::platforms::AnyPlatformBackend;
use ng_native_engine::runtime::JsRuntime;
use std::fs;

fn main() {
    // 1. Read `angular.json` to configure the project & target platform
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    let bundle_path = format!("{}/main.ngb", cfg.output_path);
    let bootstrap_script = fs::read_to_string(&bundle_path).unwrap_or_else(|_| {
        panic!("Bundle not found at '{bundle_path}'. Run `node examples/counter-app/build-bundle.mjs` first.")
    });

    println!(
        "Loaded angular.json -> project='{}', platform='{}', bundle='{}'\n",
        cfg.project_name, cfg.target_platform, bundle_path
    );

    // 2. Resolve the OS Backend via Factory Method (`linux-gtk4`, `windows-winui`, `macos-appkit`)
    let backend = AnyPlatformBackend::from_target(&cfg.target_platform).unwrap();
    let mut engine = RenderEngine::new(backend);
    let mut js_runtime = JsRuntime::new();

    // 3. Tick #1: JS Runtime evaluates the compiled Angular bundle (styles + template) and flushes Frame #1
    js_runtime
        .eval_tick(&mut engine, &bootstrap_script)
        .unwrap();

    // 4. Render visual preview of the mounted native window
    let win_id = js_runtime.resolve_var("win").unwrap();
    println!("\n--- Rendered Window Preview ---");
    println!("{}", engine.render_window_preview(win_id).unwrap());

    // 5. Register click event on the button handle resolved from JS runtime
    let btn_id = js_runtime.resolve_var("btn").unwrap();
    engine.listen(btn_id, "clicked", 100).unwrap();

    // 6. Tick #2: User clicks the button -> JS updates label -> Engine flushes Frame #2
    println!("\n--- User clicks 'Increment' button ---");
    engine
        .emit_event(btn_id, "clicked", |eng, _listener_id| {
            js_runtime.eval_tick(eng, "text lbl Count: 1").unwrap();
        })
        .unwrap();

    println!("\n--- Scene Graph Snapshot ---");
    engine.print_tree(win_id, 0).unwrap();

    engine.unlisten(100);
}
