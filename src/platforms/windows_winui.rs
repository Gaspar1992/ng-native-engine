use super::PlatformBackend;
use super::scene_state::{NativeSceneNode, NativeSceneState};
use crate::engine::{NodeId, RenderCommand};

/// Concrete Windows WinUI 3 (`Microsoft.UI.Xaml`) platform adapter.
#[derive(Default)]
pub struct WinUiBackend {
    pub flushed_frames: u64,
    state: NativeSceneState,
}

impl WinUiBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scene_state(&self) -> &NativeSceneState {
        &self.state
    }

    pub fn control_count(&self) -> usize {
        self.state.node_count()
    }

    pub fn get_control(&self, id: NodeId) -> Option<&NativeSceneNode> {
        self.state.get_node(id)
    }

    fn map_winui_tag(tag: &str) -> &'static str {
        match tag {
            "ng-window" | "adw-window" => "Microsoft.UI.Xaml.Window",
            "ng-header-bar" | "adw-header-bar" | "gtk-header-bar" => {
                "Microsoft.UI.Xaml.Controls.TitleBar"
            }
            "ng-scroll" | "gtk-scrolled-window" => "Microsoft.UI.Xaml.Controls.ScrollViewer",
            "ng-box" | "gtk-box" => "Microsoft.UI.Xaml.Controls.StackPanel",
            "ng-grid" | "gtk-grid" => "Microsoft.UI.Xaml.Controls.Grid",
            "ng-separator" | "gtk-separator" => "Microsoft.UI.Xaml.Controls.AppBarSeparator",
            "ng-label" | "gtk-label" => "Microsoft.UI.Xaml.Controls.TextBlock",
            "ng-button" | "gtk-button" => "Microsoft.UI.Xaml.Controls.Button",
            "ng-entry" | "gtk-entry" => "Microsoft.UI.Xaml.Controls.TextBox",
            "ng-switch" | "gtk-switch" => "Microsoft.UI.Xaml.Controls.ToggleSwitch",
            "ng-checkbox" | "ng-check" | "gtk-check-button" => {
                "Microsoft.UI.Xaml.Controls.CheckBox"
            }
            "ng-slider" | "ng-scale" | "gtk-scale" => "Microsoft.UI.Xaml.Controls.Slider",
            "ng-select" | "ng-dropdown" | "gtk-drop-down" => "Microsoft.UI.Xaml.Controls.ComboBox",
            "ng-image" | "gtk-picture" | "ng-picture" => "Microsoft.UI.Xaml.Controls.Image",
            "ng-level-bar" | "gtk-level-bar" | "ng-progress-bar" => {
                "Microsoft.UI.Xaml.Controls.ProgressBar"
            }
            _ => "Microsoft.UI.Xaml.UIElement",
        }
    }
}

impl PlatformBackend for WinUiBackend {
    fn backend_name(&self) -> &str {
        "Windows WinUI 3"
    }

    fn map_native_widget(&self, tag: &str) -> &'static str {
        Self::map_winui_tag(tag)
    }

    fn flush_commands(&mut self, commands: &[RenderCommand]) {
        if commands.is_empty() {
            return;
        }
        self.flushed_frames += 1;
        println!(
            "[{} | Frame #{}] Flushing {} render commands:",
            self.backend_name(),
            self.flushed_frames,
            commands.len()
        );
        for cmd in commands {
            if let RenderCommand::CreateWidget { id, tag } = cmd {
                println!(
                    "  -> CreateWidget {{ id: {id:?}, tag: {tag:?} => native: {} }}",
                    Self::map_winui_tag(tag)
                );
            } else {
                println!("  -> {cmd:?}");
            }
            self.state
                .apply_command(cmd, "Microsoft.UI.Xaml.AnchorComment", Self::map_winui_tag);
        }
    }
}
