use super::PlatformBackend;
use crate::engine::{NodeId, RenderCommand, StyleId, StyleRule};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWinUiControl {
    pub xaml_class: &'static str,
    pub text: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<NodeId>,
}

/// Concrete Windows WinUI 3 (`Microsoft.UI.Xaml`) platform adapter.
#[derive(Default)]
pub struct WinUiBackend {
    pub flushed_frames: u64,
    controls: HashMap<NodeId, NativeWinUiControl>,
    styles: HashMap<StyleId, StyleRule>,
}

impl WinUiBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn control_count(&self) -> usize {
        self.controls.len()
    }

    pub fn get_control(&self, id: NodeId) -> Option<&NativeWinUiControl> {
        self.controls.get(&id)
    }

    fn apply_command(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::RegisterStyle { style_id, rule } => {
                self.styles.insert(*style_id, rule.clone());
            }
            RenderCommand::CreateWidget { id, tag } => {
                let xaml_class = self.map_native_widget(tag);
                self.controls.insert(
                    *id,
                    NativeWinUiControl {
                        xaml_class,
                        text: String::new(),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::CreateComment { id, text } => {
                self.controls.insert(
                    *id,
                    NativeWinUiControl {
                        xaml_class: "Microsoft.UI.Xaml.AnchorComment",
                        text: text.clone(),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::SetText { id, text } => {
                if let Some(ctrl) = self.controls.get_mut(id) {
                    text.clone_into(&mut ctrl.text);
                }
            }
            RenderCommand::SetAttribute { node, key, value } => {
                if let Some(ctrl) = self.controls.get_mut(node) {
                    ctrl.attributes.insert(key.clone(), value.clone());
                }
            }
            RenderCommand::AppendChild { parent, child } => {
                if let Some(p) = self.controls.get_mut(parent) {
                    p.children.push(*child);
                }
            }
            RenderCommand::InsertBefore {
                parent,
                child,
                ref_child,
            } => {
                if let Some(p) = self.controls.get_mut(parent) {
                    p.children.retain(|c| *c != *child);
                    if let Some(r) = ref_child
                        && let Some(pos) = p.children.iter().position(|c| *c == *r)
                    {
                        p.children.insert(pos, *child);
                    } else {
                        p.children.push(*child);
                    }
                }
            }
            RenderCommand::RemoveChild { parent, child } => {
                if let Some(p) = self.controls.get_mut(parent) {
                    p.children.retain(|c| *c != *child);
                }
            }
            RenderCommand::DestroyWidget { id } => {
                self.controls.remove(id);
            }
            RenderCommand::SetWindowIcon { .. } | RenderCommand::ApplyStyleClass { .. } => {}
        }
    }
}

impl PlatformBackend for WinUiBackend {
    fn backend_name(&self) -> &str {
        "Windows WinUI 3"
    }

    fn map_native_widget(&self, tag: &str) -> &'static str {
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
            self.apply_command(cmd);
        }
    }
}
