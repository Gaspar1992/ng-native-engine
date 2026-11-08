#![allow(unsafe_code)]

use super::PlatformBackend;
use super::gtk4_sys::{Gtk4Sys, GtkHandle};
use crate::engine::{NodeId, RenderCommand, StyleId, StyleRule};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Write as _;

thread_local! {
    static CLICKED_NODES: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "C" fn on_gtk_button_clicked(_btn: *mut c_void, user_data: *mut c_void) {
    let packed = user_data as usize;
    let index = packed >> 32;
    #[allow(clippy::cast_possible_truncation)]
    let generation = (packed & 0xFFFF_FFFF) as u32;
    CLICKED_NODES.with(|queue| {
        queue.borrow_mut().push(NodeId { index, generation });
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GtkWidgetType {
    Window,
    Box,
    Label,
    Button,
}

struct NativeGtkWidget {
    handle: GtkHandle,
    kind: GtkWidgetType,
}

/// Concrete Linux GTK4 + Libadwaita platform adapter backed by `libgtk-4.so.1`.
pub struct GtkLinuxBackend {
    pub flushed_frames: u64,
    gtk: Option<Gtk4Sys>,
    widgets: HashMap<NodeId, NativeGtkWidget>,
    styles: HashMap<StyleId, StyleRule>,
}

impl Default for GtkLinuxBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl GtkLinuxBackend {
    pub fn new() -> Self {
        Self {
            flushed_frames: 0,
            gtk: Gtk4Sys::try_load(),
            widgets: HashMap::new(),
            styles: HashMap::new(),
        }
    }

    pub fn is_native_display_active(&self) -> bool {
        self.gtk.is_some()
    }

    pub fn open_window_count(&self) -> u32 {
        self.gtk.as_ref().map_or(0, Gtk4Sys::open_window_count)
    }

    pub fn step_event_loop(&self, may_block: bool) -> Vec<NodeId> {
        if let Some(gtk) = &self.gtk {
            gtk.step_main_context(may_block);
        }
        CLICKED_NODES.with(|q| std::mem::take(&mut *q.borrow_mut()))
    }

    fn rebuild_css_stylesheet(&self) -> String {
        let mut css = String::from("window { background: #1e1e1e; color: #ffffff; }\n");
        for rule in self.styles.values() {
            let _ = writeln!(css, ".{} {{", rule.class_name);
            if let Some(bg) = &rule.background {
                let _ = writeln!(css, "  background: {bg};");
            }
            if let Some(fg) = &rule.foreground {
                let _ = writeln!(css, "  color: {fg};");
            }
            if rule.padding > 0 {
                let _ = writeln!(css, "  padding: {}px;", rule.padding);
                let _ = writeln!(css, "  border-radius: 10px;");
            }
            let _ = writeln!(css, "}}");
        }
        css
    }

    fn apply_command_to_gtk(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::RegisterStyle { style_id, rule } => {
                self.styles.insert(*style_id, rule.clone());
                let stylesheet = self.rebuild_css_stylesheet();
                if let Some(gtk) = &self.gtk {
                    gtk.load_css(&stylesheet);
                }
            }
            RenderCommand::CreateWidget { id, tag } => {
                let Some(gtk) = &self.gtk else { return };
                let (handle, kind) = match tag.as_str() {
                    "ng-window" | "adw-window" => (gtk.create_window(), GtkWidgetType::Window),
                    "ng-box" | "gtk-box" => (gtk.create_box(12), GtkWidgetType::Box),
                    "ng-label" | "gtk-label" => (gtk.create_label(), GtkWidgetType::Label),
                    "ng-button" | "gtk-button" => {
                        let btn = gtk.create_button();
                        let packed = (id.index << 32) | (id.generation as usize);
                        gtk.connect_clicked(btn, on_gtk_button_clicked, packed);
                        (btn, GtkWidgetType::Button)
                    }
                    _ => return,
                };
                self.widgets.insert(*id, NativeGtkWidget { handle, kind });
            }
            RenderCommand::SetText { id, text } => {
                let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.get(id)) else {
                    return;
                };
                match widget.kind {
                    GtkWidgetType::Window => gtk.set_window_title(widget.handle, text),
                    GtkWidgetType::Label => gtk.set_label_text(widget.handle, text),
                    GtkWidgetType::Button => gtk.set_button_label(widget.handle, text),
                    GtkWidgetType::Box => {}
                }
            }
            RenderCommand::SetWindowIcon { node, icon } => {
                let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.get(node)) else {
                    return;
                };
                if widget.kind == GtkWidgetType::Window {
                    gtk.configure_window_icon(
                        widget.handle,
                        &icon.app_id,
                        "dist/ng-linux-counter/icons",
                    );
                }
            }
            RenderCommand::ApplyStyleClass { node, style_id } => {
                let (Some(gtk), Some(widget), Some(rule)) =
                    (&self.gtk, self.widgets.get(node), self.styles.get(style_id))
                else {
                    return;
                };
                gtk.add_css_class(widget.handle, &rule.class_name);
            }
            RenderCommand::AppendChild { parent, child } => {
                let (Some(gtk), Some(parent_w), Some(child_w)) =
                    (&self.gtk, self.widgets.get(parent), self.widgets.get(child))
                else {
                    return;
                };
                match parent_w.kind {
                    GtkWidgetType::Window => gtk.set_window_child(parent_w.handle, child_w.handle),
                    GtkWidgetType::Box => gtk.box_append(parent_w.handle, child_w.handle),
                    GtkWidgetType::Label | GtkWidgetType::Button => {}
                }
            }
            RenderCommand::DestroyWidget { id } => {
                if let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.remove(id))
                    && widget.kind == GtkWidgetType::Window
                {
                    gtk.destroy_window(widget.handle);
                }
            }
        }
    }
}

impl PlatformBackend for GtkLinuxBackend {
    fn backend_name(&self) -> &str {
        if self.gtk.is_some() {
            "Linux GTK4 Native (libgtk-4.so.1)"
        } else {
            "Linux GTK4 + Libadwaita"
        }
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
            self.apply_command_to_gtk(cmd);
        }
    }
}
