use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::{NodeId, RenderEngine};
use ng_native_engine::ipc::{HmrIpcServer, HmrMessage, HmrResponse};
use ng_native_engine::packager::stage_linux_distribution;
use ng_native_engine::platforms::AnyPlatformBackend;
use ng_native_engine::runtime::{JsEngineBackend, JsRuntime};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

fn bundle_modified_time(path: &str) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn handle_hmr_messages(
    server: &mut HmrIpcServer,
    engine: &mut RenderEngine<AnyPlatformBackend>,
    js_runtime: &mut JsRuntime,
    win_id: NodeId,
) {
    for (cid, msg) in server.poll_messages() {
        match msg {
            HmrMessage::Reload { script } => {
                if let Err(err) = js_runtime.hot_reload(engine, &script) {
                    eprintln!("[HMR IPC Error] Failed to reload bundle: {err:?}");
                    server.send_response(cid, &HmrResponse::Error(format!("{err:?}")));
                } else {
                    println!("\n[HMR IPC] Applied live update (signals preserved):");
                    println!("{}", engine.render_window_preview(win_id).unwrap());
                    server.send_response(cid, &HmrResponse::Ok { reloaded: true });
                }
            }
            HmrMessage::Eval { code } => match js_runtime.eval_js(engine, &code) {
                Ok(res) => server.send_response(cid, &HmrResponse::EvalResult(res)),
                Err(err) => {
                    server.send_response(cid, &HmrResponse::Error(format!("{err:?}")));
                }
            },
            HmrMessage::Ping => {
                server.send_response(cid, &HmrResponse::Pong);
            }
        }
    }
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

struct PokedexUiNodes {
    hello: Option<NodeId>,
    lbl: Option<NodeId>,
    num: Option<NodeId>,
    sprite: Option<NodeId>,
    type_primary: Option<NodeId>,
    type_secondary: Option<NodeId>,
    height: Option<NodeId>,
    weight: Option<NodeId>,
    hp_bar: Option<NodeId>,
    atk_bar: Option<NodeId>,
    def_bar: Option<NodeId>,
    spd_bar: Option<NodeId>,
}

impl PokedexUiNodes {
    fn resolve(runtime: &JsRuntime) -> Self {
        Self {
            hello: runtime.resolve_var("hello").ok(),
            lbl: runtime.resolve_var("lbl").ok(),
            num: runtime.resolve_var("poke_num").ok(),
            sprite: runtime.resolve_var("poke_sprite").ok(),
            type_primary: runtime.resolve_var("poke_type_primary").ok(),
            type_secondary: runtime.resolve_var("poke_type_secondary").ok(),
            height: runtime.resolve_var("lbl_height").ok(),
            weight: runtime.resolve_var("lbl_weight").ok(),
            hp_bar: runtime.resolve_var("hp_bar").ok(),
            atk_bar: runtime.resolve_var("atk_bar").ok(),
            def_bar: runtime.resolve_var("def_bar").ok(),
            spd_bar: runtime.resolve_var("spd_bar").ok(),
        }
    }
}

fn update_pokedex_ui(
    engine: &mut RenderEngine<AnyPlatformBackend>,
    nodes: &PokedexUiNodes,
    poke: &ng_native_engine::desktop::PokemonInfo,
) {
    if let Some(h) = nodes.hello {
        let _ = engine.set_text(h, poke.display_name().to_uppercase());
    }
    if let Some(n) = nodes.num {
        let _ = engine.set_text(n, poke.format_id());
    }
    if let Some(s) = nodes.sprite {
        let _ = engine.set_text(s, &poke.sprite_path);
    }
    if let Some(tp) = nodes.type_primary {
        let _ = engine.set_text(tp, poke.primary_type.to_uppercase());
    }
    if let Some(ts) = nodes.type_secondary {
        let sec = poke.secondary_type.as_deref().unwrap_or("").to_uppercase();
        let _ = engine.set_text(ts, sec);
    }
    if let Some(h) = nodes.height {
        let _ = engine.set_text(h, poke.format_height());
    }
    if let Some(w) = nodes.weight {
        let _ = engine.set_text(w, poke.format_weight());
    }
    if let Some(l) = nodes.lbl {
        let _ = engine.set_text(
            l,
            format!(
                "HP: {} | ATK: {} | DEF: {} | SPD: {}",
                poke.hp, poke.attack, poke.defense, poke.speed
            ),
        );
    }
    if let Some(bar) = nodes.hp_bar {
        let _ = engine.set_attribute(bar, "value", poke.hp.to_string());
    }
    if let Some(bar) = nodes.atk_bar {
        let _ = engine.set_attribute(bar, "value", poke.attack.to_string());
    }
    if let Some(bar) = nodes.def_bar {
        let _ = engine.set_attribute(bar, "value", poke.defense.to_string());
    }
    if let Some(bar) = nodes.spd_bar {
        let _ = engine.set_attribute(bar, "value", poke.speed.to_string());
    }
}

struct InteractiveContext<'a> {
    cfg: &'a AngularNativeConfig,
    bundle_path: &'a str,
    last_mtime: Option<SystemTime>,
    win_id: NodeId,
}

fn run_interactive_loop(
    mut ctx: InteractiveContext<'_>,
    mut engine: RenderEngine<AnyPlatformBackend>,
    mut js_runtime: JsRuntime,
    mut hmr_server: Option<HmrIpcServer>,
) {
    println!(
        "\n[GTK4 Native Window Active] Opened native Linux window '{}'. Watching '{}' for Hot Reload...",
        ctx.cfg.project_name, ctx.bundle_path
    );
    let mut last_pokedex_id: Option<i64> = None;
    let mut nodes = PokedexUiNodes::resolve(&js_runtime);

    if let Ok(poke) = js_runtime.desktop_mut().fetch_pokemon("1") {
        update_pokedex_ui(&mut engine, &nodes, &poke);
        last_pokedex_id = Some(1);
    }

    while engine.backend().open_window_count() > 0 {
        let native_events = engine.backend().step_native_events(false);
        for ev in native_events {
            if let ng_native_engine::engine::EventPayload::Text(ref query) = ev.payload {
                let clean = query.trim();
                if !clean.is_empty()
                    && clean != "ng-native-dev"
                    && let Ok(poke) = js_runtime.desktop_mut().fetch_pokemon(clean)
                {
                    println!(
                        "\n[PokéAPI] Searched '{}' -> {} {} ({})",
                        clean,
                        poke.format_id(),
                        poke.display_name(),
                        poke.primary_type
                    );
                    update_pokedex_ui(&mut engine, &nodes, &poke);
                    last_pokedex_id = Some(i64::from(poke.id));
                }
            }

            js_runtime
                .trigger_node_event_with_payload(&mut engine, ev.node, &ev.event_name, &ev.payload)
                .unwrap();
        }

        let current_count = js_runtime.signal_value("count").unwrap_or(0);
        if Some(current_count) != last_pokedex_id && current_count > 0 {
            if let Ok(poke) = js_runtime
                .desktop_mut()
                .fetch_pokemon(&current_count.to_string())
            {
                println!(
                    "\n[PokéAPI] Navigated -> {} {} ({})",
                    poke.format_id(),
                    poke.display_name(),
                    poke.primary_type
                );
                update_pokedex_ui(&mut engine, &nodes, &poke);
            }
            last_pokedex_id = Some(current_count);
        }

        if let Some(ref mut server) = hmr_server {
            handle_hmr_messages(server, &mut engine, &mut js_runtime, ctx.win_id);
            nodes = PokedexUiNodes::resolve(&js_runtime);
        }

        let current_mtime = bundle_modified_time(ctx.bundle_path);
        if current_mtime.is_some()
            && current_mtime != ctx.last_mtime
            && let Ok(updated_script) = fs::read_to_string(ctx.bundle_path)
            && !updated_script.trim().is_empty()
        {
            ctx.last_mtime = current_mtime;
            if let Err(err) = js_runtime.hot_reload(&mut engine, &updated_script) {
                eprintln!("[Hot Reload Error] Failed to apply bundle update: {err:?}");
            } else {
                println!(
                    "\n[Hot Reload] Applied live update from '{}' (signals preserved):",
                    ctx.bundle_path
                );
                println!("{}", engine.render_window_preview(ctx.win_id).unwrap());
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    println!("\n--- Scene Graph Snapshot ---");
    engine.print_tree(ctx.win_id, 0).unwrap();
}

fn main() {
    // 1. Read `angular.json` to configure the project & target platform
    let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
    let bundle_path = format!("{}/main.ngb", cfg.output_path);
    let bootstrap_script = ensure_bundle_compiled(&bundle_path);
    let last_mtime = bundle_modified_time(&bundle_path);

    println!(
        "Loaded angular.json -> project='{}', platform='{}', bundle='{}'\n",
        cfg.project_name, cfg.target_platform, bundle_path
    );

    if env::args().any(|a| a == "--package") {
        let artifacts = stage_linux_distribution(&cfg, "0.1.0").unwrap();
        println!("[NgNative Packager] Staged Linux distribution artifacts:");
        for path in artifacts {
            println!("  -> {path}");
        }
        return;
    }

    // 2. Resolve the OS Backend & Pluggable JS Engine (`jsc` or `quickjs`) via Factory Method
    let backend = AnyPlatformBackend::from_target(&cfg.target_platform).unwrap();
    let mut engine = RenderEngine::new(backend);
    let mut js_runtime = JsRuntime::from_engine_config(&cfg.js_engine).unwrap();
    println!(
        "Initialized JS Engine -> '{}' (JIT={})\n",
        js_runtime.js_engine().engine_name(),
        js_runtime.js_engine().is_jit_enabled()
    );

    // 3. Tick #1: JS Runtime evaluates the compiled Angular bundle (styles + template + bindings)
    js_runtime
        .eval_tick(&mut engine, &bootstrap_script)
        .unwrap();

    // 4. Render visual preview of the mounted native window
    let win_id = js_runtime.resolve_var("win").unwrap();
    println!("\n--- Rendered Window Preview ---");
    println!("{}", engine.render_window_preview(win_id).unwrap());

    let hmr_socket = format!("{}/hmr.sock", cfg.output_path);
    let hmr_server = HmrIpcServer::bind(&hmr_socket).ok();
    if hmr_server.is_some() {
        println!("[HMR IPC] Socket server active -> '{hmr_socket}'");
    }

    if engine.backend().is_native_display_active() && env::var("NG_NATIVE_BATCH").is_err() {
        run_interactive_loop(
            InteractiveContext {
                cfg: &cfg,
                bundle_path: &bundle_path,
                last_mtime,
                win_id,
            },
            engine,
            js_runtime,
            hmr_server,
        );
    } else {
        // Pump non-blocking GTK4 events if available and run declarative click simulation
        let _ = engine.backend().step_event_loop(false);
        println!("\n--- Simulating declarative (click)=\"increment()\" on 'btn' ---");
        js_runtime
            .trigger_event(&mut engine, "btn", "clicked")
            .unwrap();
        println!("\n--- Scene Graph Snapshot ---");
        engine.print_tree(win_id, 0).unwrap();
    }
}
