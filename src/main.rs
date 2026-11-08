use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::RenderEngine;
use ng_native_engine::platforms::AnyPlatformBackend;
use ng_native_engine::runtime::JsRuntime;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

fn bundle_modified_time(path: &str) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn ensure_bundle_compiled(bundle_path: &str) -> String {
    if !Path::new(bundle_path).exists() {
        println!(
            "[NgNative CLI] Bundle not found at '{bundle_path}'. Auto-compiling via build-bundle.mjs..."
        );
        let _ = Command::new("node")
            .arg("examples/counter-app/build-bundle.mjs")
            .status();
    }
    fs::read_to_string(bundle_path).unwrap_or_else(|_| {
        panic!("Bundle not found at '{bundle_path}'. Run `pnpm run build` first.")
    })
}

fn main() {
    // 1. Read `angular.json` to configure the project & target platform
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    let bundle_path = format!("{}/main.ngb", cfg.output_path);
    let bootstrap_script = ensure_bundle_compiled(&bundle_path);
    let mut last_mtime = bundle_modified_time(&bundle_path);

    println!(
        "Loaded angular.json -> project='{}', platform='{}', bundle='{}'\n",
        cfg.project_name, cfg.target_platform, bundle_path
    );

    // 2. Resolve the OS Backend via Factory Method (`linux-gtk4`, `windows-winui`, `macos-appkit`)
    let backend = AnyPlatformBackend::from_target(&cfg.target_platform).unwrap();
    let mut engine = RenderEngine::new(backend);
    let mut js_runtime = JsRuntime::new();

    // 3. Tick #1: JS Runtime evaluates the compiled Angular bundle (styles + template + bindings)
    js_runtime
        .eval_tick(&mut engine, &bootstrap_script)
        .unwrap();

    // 4. Render visual preview of the mounted native window
    let win_id = js_runtime.resolve_var("win").unwrap();
    println!("\n--- Rendered Window Preview ---");
    println!("{}", engine.render_window_preview(win_id).unwrap());

    if engine.backend().is_native_display_active() && env::var("NG_NATIVE_BATCH").is_err() {
        println!(
            "\n[GTK4 Native Window Active] Opened native Linux window '{}'. Watching '{}' for Hot Reload...",
            cfg.project_name, bundle_path
        );
        while engine.backend().open_window_count() > 0 {
            let clicked_nodes = engine.backend().step_event_loop(false);
            for clicked_id in clicked_nodes {
                js_runtime
                    .trigger_node_event(&mut engine, clicked_id, "clicked")
                    .unwrap();
            }

            let current_mtime = bundle_modified_time(&bundle_path);
            if current_mtime.is_some()
                && current_mtime != last_mtime
                && let Ok(updated_script) = fs::read_to_string(&bundle_path)
                && !updated_script.trim().is_empty()
            {
                last_mtime = current_mtime;
                if let Err(err) = js_runtime.hot_reload(&mut engine, &updated_script) {
                    eprintln!("[Hot Reload Error] Failed to apply bundle update: {err:?}");
                } else {
                    println!(
                        "\n[Hot Reload] Applied live update from '{bundle_path}' (signals preserved):"
                    );
                    println!("{}", engine.render_window_preview(win_id).unwrap());
                }
            }

            thread::sleep(Duration::from_millis(16));
        }
    } else {
        // Pump non-blocking GTK4 events if available and run declarative click simulation
        let _ = engine.backend().step_event_loop(false);
        println!("\n--- Simulating declarative (click)=\"increment()\" on 'btn' ---");
        js_runtime
            .trigger_event(&mut engine, "btn", "clicked")
            .unwrap();
    }

    println!("\n--- Scene Graph Snapshot ---");
    engine.print_tree(win_id, 0).unwrap();
}
