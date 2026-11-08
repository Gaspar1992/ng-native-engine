use super::PlatformBackend;
use crate::engine::RenderCommand;

/// Concrete Windows WinUI 3 platform adapter.
#[derive(Default)]
pub struct WinUiBackend {
    pub flushed_frames: u64,
}

impl WinUiBackend {
    pub fn new() -> Self {
        Self { flushed_frames: 0 }
    }
}

impl PlatformBackend for WinUiBackend {
    fn backend_name(&self) -> &str {
        "Windows WinUI 3"
    }

    fn map_native_widget(&self, tag: &str) -> &'static str {
        match tag {
            "ng-window" | "adw-window" => "Microsoft.UI.Xaml.Window",
            "ng-box" | "gtk-box" => "Microsoft.UI.Xaml.Controls.StackPanel",
            "ng-label" | "gtk-label" => "Microsoft.UI.Xaml.Controls.TextBlock",
            "ng-button" | "gtk-button" => "Microsoft.UI.Xaml.Controls.Button",
            _ => "Microsoft.UI.Xaml.UIElement",
        }
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
                    self.map_native_widget(tag)
                );
            } else {
                println!("  -> {cmd:?}");
            }
        }
    }
}
