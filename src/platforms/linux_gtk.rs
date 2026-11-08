use super::PlatformBackend;
use crate::engine::RenderCommand;

/// Concrete Linux GTK4 + Libadwaita platform adapter.
#[derive(Default)]
pub struct GtkLinuxBackend {
    pub flushed_frames: u64,
}

impl GtkLinuxBackend {
    pub fn new() -> Self {
        Self { flushed_frames: 0 }
    }
}

impl PlatformBackend for GtkLinuxBackend {
    fn backend_name(&self) -> &str {
        "Linux GTK4 + Libadwaita"
    }

    fn map_native_widget(&self, tag: &str) -> &'static str {
        match tag {
            "ng-window" | "adw-window" => "AdwApplicationWindow",
            "ng-box" | "gtk-box" => "GtkBox",
            "ng-label" | "gtk-label" => "GtkLabel",
            "ng-button" | "gtk-button" => "GtkButton",
            _ => "GtkWidget",
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
