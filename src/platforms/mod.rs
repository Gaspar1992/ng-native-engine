pub mod gtk4_callbacks;
pub mod gtk4_sys;
pub mod gtk4_widgets;
pub mod headless;
pub mod linux_gtk;
pub mod macos_appkit;
pub mod windows_winui;

use crate::engine::{NativeEvent, NodeId, RenderCommand};
pub use headless::HeadlessBackend;
pub use linux_gtk::GtkLinuxBackend;
pub use macos_appkit::MacOsAppKitBackend;
pub use windows_winui::WinUiBackend;

/// Generic platform interface (Render Hardware Interface / OS Bridge).
/// Applies Dependency Inversion (DIP) and Open/Closed Principle (OCP):
/// `RenderEngine` depends only on this abstraction, never on a concrete OS toolkit.
pub trait PlatformBackend {
    fn backend_name(&self) -> &str;
    fn map_native_widget(&self, tag: &str) -> &'static str {
        match tag {
            "ng-window" | "adw-window" => "NativeWindow",
            "ng-header-bar" | "adw-header-bar" | "gtk-header-bar" => "NativeHeaderBar",
            "ng-scroll" | "gtk-scrolled-window" => "NativeScrollView",
            "ng-box" | "gtk-box" => "NativeBox",
            "ng-grid" | "gtk-grid" => "NativeGrid",
            "ng-separator" | "gtk-separator" => "NativeSeparator",
            "ng-label" | "gtk-label" => "NativeLabel",
            "ng-button" | "gtk-button" => "NativeButton",
            "ng-entry" | "gtk-entry" => "NativeEntry",
            "ng-switch" | "gtk-switch" => "NativeSwitch",
            "ng-checkbox" | "ng-check" | "gtk-check-button" => "NativeCheckButton",
            "ng-slider" | "ng-scale" | "gtk-scale" => "NativeSlider",
            "ng-select" | "ng-dropdown" | "gtk-drop-down" => "NativeDropDown",
            _ => "NativeWidget",
        }
    }
    fn flush_commands(&mut self, commands: &[RenderCommand]);
}

/// Polymorphic backend wrapper produced by the Platform Factory from `angular.json`.
pub enum AnyPlatformBackend {
    LinuxGtk(Box<GtkLinuxBackend>),
    WindowsWinUi(WinUiBackend),
    MacOsAppKit(MacOsAppKitBackend),
}

impl AnyPlatformBackend {
    /// Factory Method resolving the concrete OS adapter from `angular.json` `targetPlatform`.
    pub fn from_target(target_platform: &str) -> Result<Self, String> {
        match target_platform {
            "linux-gtk4" => Ok(Self::LinuxGtk(Box::default())),
            "windows-winui" => Ok(Self::WindowsWinUi(WinUiBackend::new())),
            "macos-appkit" => Ok(Self::MacOsAppKit(MacOsAppKitBackend::new())),
            other => Err(format!("Unsupported targetPlatform '{other}'")),
        }
    }

    pub fn is_native_display_active(&self) -> bool {
        match self {
            Self::LinuxGtk(b) => b.is_native_display_active(),
            Self::WindowsWinUi(_) | Self::MacOsAppKit(_) => false,
        }
    }

    pub fn open_window_count(&self) -> u32 {
        match self {
            Self::LinuxGtk(b) => b.open_window_count(),
            Self::WindowsWinUi(_) | Self::MacOsAppKit(_) => 0,
        }
    }

    pub fn step_event_loop(&self, may_block: bool) -> Vec<NodeId> {
        match self {
            Self::LinuxGtk(b) => b.step_event_loop(may_block),
            Self::WindowsWinUi(_) | Self::MacOsAppKit(_) => Vec::new(),
        }
    }

    pub fn step_native_events(&self, may_block: bool) -> Vec<NativeEvent> {
        match self {
            Self::LinuxGtk(b) => b.step_native_events(may_block),
            Self::WindowsWinUi(_) | Self::MacOsAppKit(_) => Vec::new(),
        }
    }
}

impl PlatformBackend for AnyPlatformBackend {
    fn backend_name(&self) -> &str {
        match self {
            Self::LinuxGtk(b) => b.backend_name(),
            Self::WindowsWinUi(b) => b.backend_name(),
            Self::MacOsAppKit(b) => b.backend_name(),
        }
    }

    fn map_native_widget(&self, tag: &str) -> &'static str {
        match self {
            Self::LinuxGtk(b) => b.map_native_widget(tag),
            Self::WindowsWinUi(b) => b.map_native_widget(tag),
            Self::MacOsAppKit(b) => b.map_native_widget(tag),
        }
    }

    fn flush_commands(&mut self, commands: &[RenderCommand]) {
        match self {
            Self::LinuxGtk(b) => b.flush_commands(commands),
            Self::WindowsWinUi(b) => b.flush_commands(commands),
            Self::MacOsAppKit(b) => b.flush_commands(commands),
        }
    }
}
