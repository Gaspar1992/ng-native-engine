use super::PlatformBackend;
use crate::engine::{NodeId, RenderCommand, StyleId, StyleRule};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAppKitView {
    pub cocoa_class: &'static str,
    pub text: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<NodeId>,
}

/// Concrete macOS AppKit (Cocoa) platform adapter.
#[derive(Default)]
pub struct MacOsAppKitBackend {
    pub flushed_frames: u64,
    views: HashMap<NodeId, NativeAppKitView>,
    styles: HashMap<StyleId, StyleRule>,
}

impl MacOsAppKitBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view_count(&self) -> usize {
        self.views.len()
    }

    pub fn get_view(&self, id: NodeId) -> Option<&NativeAppKitView> {
        self.views.get(&id)
    }

    fn apply_command(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::RegisterStyle { style_id, rule } => {
                self.styles.insert(*style_id, rule.clone());
            }
            RenderCommand::CreateWidget { id, tag } => {
                let cocoa_class = self.map_native_widget(tag);
                self.views.insert(
                    *id,
                    NativeAppKitView {
                        cocoa_class,
                        text: String::new(),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::CreateComment { id, text } => {
                self.views.insert(
                    *id,
                    NativeAppKitView {
                        cocoa_class: "NSAnchorComment",
                        text: text.clone(),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::SetText { id, text } => {
                if let Some(view) = self.views.get_mut(id) {
                    text.clone_into(&mut view.text);
                }
            }
            RenderCommand::SetAttribute { node, key, value } => {
                if let Some(view) = self.views.get_mut(node) {
                    view.attributes.insert(key.clone(), value.clone());
                }
            }
            RenderCommand::AppendChild { parent, child } => {
                if let Some(p) = self.views.get_mut(parent) {
                    p.children.push(*child);
                }
            }
            RenderCommand::InsertBefore {
                parent,
                child,
                ref_child,
            } => {
                if let Some(p) = self.views.get_mut(parent) {
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
                if let Some(p) = self.views.get_mut(parent) {
                    p.children.retain(|c| *c != *child);
                }
            }
            RenderCommand::DestroyWidget { id } => {
                self.views.remove(id);
            }
            RenderCommand::SetWindowIcon { .. } | RenderCommand::ApplyStyleClass { .. } => {}
        }
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
            self.apply_command(cmd);
        }
    }
}
