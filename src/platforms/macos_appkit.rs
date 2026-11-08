use super::PlatformBackend;
use crate::engine::RenderCommand;

/// Concrete macOS AppKit (Cocoa) platform adapter.
#[derive(Default)]
pub struct MacOsAppKitBackend {
    pub flushed_frames: u64,
}

impl MacOsAppKitBackend {
    pub fn new() -> Self {
        Self { flushed_frames: 0 }
    }
}

impl PlatformBackend for MacOsAppKitBackend {
    fn backend_name(&self) -> &str {
        "macOS AppKit (Cocoa)"
    }

    fn map_native_widget(&self, tag: &str) -> &'static str {
        match tag {
            "ng-window" | "adw-window" => "NSWindow",
            "ng-header-bar" | "adw-header-bar" | "gtk-header-bar" => "NSToolbar",
            "ng-scroll" | "gtk-scrolled-window" => "NSScrollView",
            "ng-box" | "gtk-box" => "NSStackView",
            "ng-grid" | "gtk-grid" => "NSGridView",
            "ng-separator" | "gtk-separator" => "NSBoxSeparator",
            "ng-label" | "gtk-label" => "NSTextField",
            "ng-button" | "gtk-button" => "NSButton",
            "ng-entry" | "gtk-entry" => "NSTextFieldEditable",
            "ng-switch" | "gtk-switch" => "NSSwitch",
            "ng-checkbox" | "ng-check" | "gtk-check-button" => "NSButtonCheckbox",
            "ng-slider" | "ng-scale" | "gtk-scale" => "NSSlider",
            "ng-select" | "ng-dropdown" | "gtk-drop-down" => "NSPopUpButton",
            _ => "NSView",
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
