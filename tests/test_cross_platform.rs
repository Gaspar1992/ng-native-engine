use ng_native_engine::config::AngularNativeConfig;
use ng_native_engine::engine::RenderEngine;
use ng_native_engine::packager::stage_linux_distribution;
use ng_native_engine::platforms::{
    AnyPlatformBackend, MacOsAppKitBackend, PlatformBackend, WinUiBackend,
};
use ng_native_engine::runtime::JsRuntime;
use std::fs;

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

#[test]
fn winui_and_appkit_retain_nested_grid_and_form_hierarchies() {
    let script = r"
        create win ng-window
        create grid ng-grid
        create sw ng-switch
        attr sw active true
        create chk ng-checkbox
        text chk Enable Feature
        create sel ng-select
        attr sel options Alpha,Beta,Gamma
        append grid sw
        append grid chk
        append grid sel
        append win grid
    ";

    let mut winui_engine = RenderEngine::new(WinUiBackend::new());
    let mut winui_rt = JsRuntime::new();
    winui_rt.eval_tick(&mut winui_engine, script).unwrap();
    let grid_id = winui_rt.resolve_var("grid").unwrap();
    let sw_id = winui_rt.resolve_var("sw").unwrap();

    let grid_node = winui_engine.backend().get_control(grid_id).unwrap();
    assert_eq!(grid_node.native_class, "Microsoft.UI.Xaml.Controls.Grid");
    assert_eq!(grid_node.children.len(), 3);

    let sw_node = winui_engine.backend().get_control(sw_id).unwrap();
    assert_eq!(
        sw_node.native_class,
        "Microsoft.UI.Xaml.Controls.ToggleSwitch"
    );
    assert_eq!(sw_node.attributes.get("active"), Some(&"true".to_string()));
}
