#![allow(unsafe_code)]

use super::PlatformBackend;
use super::gtk4_callbacks::{
    drain_pending_events, on_gtk_button_clicked, on_gtk_check_toggled,
    on_gtk_drop_down_selected_notify, on_gtk_entry_changed, on_gtk_scale_value_changed,
    on_gtk_switch_active_notify, pack_node_id,
};
use super::gtk4_sys::{Gtk4Sys, GtkHandle};
use crate::engine::{EventPayload, NativeEvent, NodeId, RenderCommand, StyleId, StyleRule};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GtkWidgetType {
    Window,
    HeaderBar,
    Scroll,
    Box,
    Grid,
    Separator,
    Label,
    Button,
    Entry,
    Switch,
    CheckButton,
    Slider,
    DropDown,
    Image,
    LevelBar,
    FlowBox,
    Anchor,
}

pub(super) struct NativeGtkWidget {
    pub(super) handle: Option<GtkHandle>,
    pub(super) kind: GtkWidgetType,
    pub(super) children: Vec<NodeId>,
}

/// Concrete Linux GTK4 + Libadwaita platform adapter backed by `libgtk-4.so.1`.
pub struct GtkLinuxBackend {
    pub flushed_frames: u64,
    pub(super) gtk: Option<Gtk4Sys>,
    pub(super) widgets: HashMap<NodeId, NativeGtkWidget>,
    pub(super) styles: HashMap<StyleId, StyleRule>,
    pub(super) header_titles: HashMap<NodeId, (String, String)>,
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
            header_titles: HashMap::new(),
        }
    }

    pub fn is_native_display_active(&self) -> bool {
        self.gtk.is_some()
    }

    pub fn open_window_count(&self) -> u32 {
        self.gtk.as_ref().map_or(0, Gtk4Sys::open_window_count)
    }

    pub fn step_event_loop(&self, may_block: bool) -> Vec<NodeId> {
        self.step_native_events(may_block)
            .into_iter()
            .map(|ev| ev.node)
            .collect()
    }

    pub fn step_native_events(&self, may_block: bool) -> Vec<NativeEvent> {
        if let Some(gtk) = &self.gtk {
            gtk.step_main_context(may_block);
        }
        let mut events = drain_pending_events();
        if let Some(gtk) = &self.gtk {
            for ev in &mut events {
                if let Some(widget) = self.widgets.get(&ev.node)
                    && let Some(handle) = widget.handle
                {
                    ev.payload = match widget.kind {
                        GtkWidgetType::Entry => EventPayload::Text(gtk.get_entry_text(handle)),
                        GtkWidgetType::Switch => EventPayload::Bool(gtk.get_switch_active(handle)),
                        GtkWidgetType::CheckButton => {
                            EventPayload::Bool(gtk.get_check_button_active(handle))
                        }
                        GtkWidgetType::Slider => EventPayload::Int(gtk.get_slider_value(handle)),
                        GtkWidgetType::DropDown => {
                            EventPayload::Int(gtk.get_drop_down_selected(handle))
                        }
                        _ => EventPayload::None,
                    };
                }
            }
        }
        events
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
            RenderCommand::CreateWidget { id, tag } => self.create_native_widget(*id, tag),
            RenderCommand::CreateComment { id, .. } => {
                self.widgets.insert(
                    *id,
                    NativeGtkWidget {
                        handle: None,
                        kind: GtkWidgetType::Anchor,
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::SetText { id, text } => {
                let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.get(id)) else {
                    return;
                };
                let Some(handle) = widget.handle else { return };
                match widget.kind {
                    GtkWidgetType::Window => gtk.set_window_title(handle, text),
                    GtkWidgetType::HeaderBar => {
                        let entry = self.header_titles.entry(*id).or_default();
                        entry.0.clone_from(text);
                        let (t, s) = (entry.0.clone(), entry.1.clone());
                        gtk.set_header_bar_title(handle, &t, &s);
                    }
                    GtkWidgetType::Label => gtk.set_label_text(handle, text),
                    GtkWidgetType::Button => gtk.set_button_label(handle, text),
                    GtkWidgetType::Entry => gtk.set_entry_text(handle, text),
                    GtkWidgetType::CheckButton => gtk.set_check_button_label(handle, text),
                    GtkWidgetType::Image => gtk.set_image_file(handle, text),
                    _ => {}
                }
            }
            RenderCommand::SetAttribute { node, key, value } => {
                self.apply_native_attribute(*node, key, value);
            }
            RenderCommand::SetWindowIcon { node, icon } => {
                let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.get(node)) else {
                    return;
                };
                if widget.kind == GtkWidgetType::Window
                    && let Some(handle) = widget.handle
                {
                    gtk.configure_window_icon(handle, &icon.app_id, "dist/ng-linux-counter/icons");
                }
            }
            RenderCommand::ApplyStyleClass { node, style_id } => {
                let (Some(gtk), Some(widget), Some(rule)) =
                    (&self.gtk, self.widgets.get(node), self.styles.get(style_id))
                else {
                    return;
                };
                if let Some(handle) = widget.handle {
                    gtk.add_css_class(handle, &rule.class_name);
                }
            }
            RenderCommand::AppendChild { parent, child } => {
                self.apply_append_child(*parent, *child);
            }
            RenderCommand::InsertBefore {
                parent,
                child,
                ref_child,
            } => {
                self.apply_insert_before(*parent, *child, *ref_child);
            }
            RenderCommand::RemoveChild { parent, child } => {
                self.apply_remove_child(*parent, *child);
            }
            RenderCommand::DestroyWidget { id } => {
                if let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.remove(id))
                    && widget.kind == GtkWidgetType::Window
                    && let Some(handle) = widget.handle
                {
                    gtk.destroy_window(handle);
                }
            }
        }
    }

    fn create_native_widget(&mut self, id: NodeId, tag: &str) {
        let Some(gtk) = &self.gtk else { return };
        let (handle, kind) = match tag {
            "ng-window" | "adw-window" => (Some(gtk.create_window()), GtkWidgetType::Window),
            "ng-header-bar" | "adw-header-bar" | "gtk-header-bar" => {
                (Some(gtk.create_header_bar()), GtkWidgetType::HeaderBar)
            }
            "ng-scroll" | "gtk-scrolled-window" => {
                (Some(gtk.create_scrolled_window()), GtkWidgetType::Scroll)
            }
            "ng-box" | "gtk-box" => (Some(gtk.create_box(12)), GtkWidgetType::Box),
            "ng-grid" | "gtk-grid" => (Some(gtk.create_grid()), GtkWidgetType::Grid),
            "ng-separator" | "gtk-separator" => {
                (Some(gtk.create_separator()), GtkWidgetType::Separator)
            }
            "ng-label" | "gtk-label" => (Some(gtk.create_label()), GtkWidgetType::Label),
            "ng-button" | "gtk-button" => {
                let btn = gtk.create_button();
                gtk.connect_clicked(btn, on_gtk_button_clicked, pack_node_id(id));
                (Some(btn), GtkWidgetType::Button)
            }
            "ng-entry" | "gtk-entry" => {
                let entry = gtk.create_entry();
                gtk.connect_signal(entry, c"changed", on_gtk_entry_changed, pack_node_id(id));
                (Some(entry), GtkWidgetType::Entry)
            }
            "ng-switch" | "gtk-switch" => {
                let sw = gtk.create_switch();
                gtk.connect_notify_signal(
                    sw,
                    c"notify::active",
                    on_gtk_switch_active_notify,
                    pack_node_id(id),
                );
                (Some(sw), GtkWidgetType::Switch)
            }
            "ng-checkbox" | "ng-check" | "gtk-check-button" => {
                let chk = gtk.create_check_button();
                gtk.connect_signal(chk, c"toggled", on_gtk_check_toggled, pack_node_id(id));
                (Some(chk), GtkWidgetType::CheckButton)
            }
            "ng-slider" | "ng-scale" | "gtk-scale" => {
                let scale = gtk.create_slider();
                gtk.connect_signal(
                    scale,
                    c"value-changed",
                    on_gtk_scale_value_changed,
                    pack_node_id(id),
                );
                (Some(scale), GtkWidgetType::Slider)
            }
            "ng-select" | "ng-dropdown" | "gtk-drop-down" => {
                let dd = gtk.create_drop_down();
                gtk.connect_notify_signal(
                    dd,
                    c"notify::selected",
                    on_gtk_drop_down_selected_notify,
                    pack_node_id(id),
                );
                (Some(dd), GtkWidgetType::DropDown)
            }
            "ng-image" | "gtk-picture" | "ng-picture" => {
                (Some(gtk.create_image()), GtkWidgetType::Image)
            }
            "ng-level-bar" | "gtk-level-bar" | "ng-progress-bar" => {
                (Some(gtk.create_level_bar()), GtkWidgetType::LevelBar)
            }
            "ng-flow" | "ng-flow-box" | "gtk-flow-box" => {
                (Some(gtk.create_flow_box()), GtkWidgetType::FlowBox)
            }
            "ng-anchor" | "#comment" => (None, GtkWidgetType::Anchor),
            _ => return,
        };
        self.widgets.insert(
            id,
            NativeGtkWidget {
                handle,
                kind,
                children: Vec::new(),
            },
        );
    }

    fn apply_append_child(&mut self, parent: NodeId, child: NodeId) {
        let (Some(gtk), Some(parent_w), Some(child_w)) = (
            &self.gtk,
            self.widgets.get(&parent),
            self.widgets.get(&child),
        ) else {
            return;
        };
        let child_index = i32::try_from(parent_w.children.len()).unwrap_or(0);
        if let (Some(p_h), Some(c_h)) = (parent_w.handle, child_w.handle) {
            match parent_w.kind {
                GtkWidgetType::Window => {
                    if child_w.kind == GtkWidgetType::HeaderBar {
                        gtk.set_window_titlebar(p_h, c_h);
                    } else {
                        gtk.set_window_child(p_h, c_h);
                    }
                }
                GtkWidgetType::HeaderBar => gtk.header_bar_pack_end(p_h, c_h),
                GtkWidgetType::Scroll => gtk.set_scrolled_child(p_h, c_h),
                GtkWidgetType::Box => gtk.box_append(p_h, c_h),
                GtkWidgetType::FlowBox => gtk.flow_box_append(p_h, c_h),
                GtkWidgetType::Grid => {
                    gtk.grid_attach(p_h, c_h, child_index % 2, child_index / 2);
                }
                _ => {}
            }
        }
        if let Some(pw) = self.widgets.get_mut(&parent) {
            pw.children.push(child);
        }
    }

    fn apply_insert_before(&mut self, parent: NodeId, child: NodeId, ref_child: Option<NodeId>) {
        let (Some(gtk), Some(parent_w), Some(child_w)) = (
            &self.gtk,
            self.widgets.get(&parent),
            self.widgets.get(&child),
        ) else {
            return;
        };

        let mut prev_real_sibling: Option<GtkHandle> = None;
        if let Some(ref_id) = ref_child
            && let Some(pos) = parent_w.children.iter().position(|c| *c == ref_id)
        {
            for prev_id in parent_w.children[..pos].iter().rev() {
                if let Some(prev_w) = self.widgets.get(prev_id)
                    && let Some(h) = prev_w.handle
                {
                    prev_real_sibling = Some(h);
                    break;
                }
            }
        }

        if parent_w.kind == GtkWidgetType::Box
            && let (Some(p_h), Some(c_h)) = (parent_w.handle, child_w.handle)
        {
            if ref_child.is_some() {
                gtk.box_insert_after(p_h, c_h, prev_real_sibling);
            } else {
                gtk.box_append(p_h, c_h);
            }
        }

        if let Some(pw) = self.widgets.get_mut(&parent) {
            pw.children.retain(|c| *c != child);
            if let Some(ref_id) = ref_child
                && let Some(pos) = pw.children.iter().position(|c| *c == ref_id)
            {
                pw.children.insert(pos, child);
            } else {
                pw.children.push(child);
            }
        }
    }

    fn apply_remove_child(&mut self, parent: NodeId, child: NodeId) {
        let (Some(gtk), Some(parent_w), Some(child_w)) = (
            &self.gtk,
            self.widgets.get(&parent),
            self.widgets.get(&child),
        ) else {
            return;
        };
        if parent_w.kind == GtkWidgetType::Box
            && let (Some(p_h), Some(c_h)) = (parent_w.handle, child_w.handle)
        {
            gtk.box_remove(p_h, c_h);
        }
        if parent_w.kind == GtkWidgetType::FlowBox
            && let (Some(p_h), Some(c_h)) = (parent_w.handle, child_w.handle)
        {
            gtk.flow_box_remove(p_h, c_h);
        }
        if let Some(pw) = self.widgets.get_mut(&parent) {
            pw.children.retain(|c| *c != child);
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
            "ng-header-bar" | "adw-header-bar" | "gtk-header-bar" => "AdwHeaderBar",
            "ng-scroll" | "gtk-scrolled-window" => "GtkScrolledWindow",
            "ng-box" | "gtk-box" => "GtkBox",
            "ng-grid" | "gtk-grid" => "GtkGrid",
            "ng-separator" | "gtk-separator" => "GtkSeparator",
            "ng-label" | "gtk-label" => "GtkLabel",
            "ng-button" | "gtk-button" => "GtkButton",
            "ng-entry" | "gtk-entry" => "GtkEntry",
            "ng-switch" | "gtk-switch" => "GtkSwitch",
            "ng-checkbox" | "ng-check" | "gtk-check-button" => "GtkCheckButton",
            "ng-slider" | "ng-scale" | "gtk-scale" => "GtkScale",
            "ng-select" | "ng-dropdown" | "gtk-drop-down" => "GtkDropDown",
            "ng-image" | "gtk-picture" | "ng-picture" => "GtkPicture",
            "ng-level-bar" | "gtk-level-bar" | "ng-progress-bar" => "GtkLevelBar",
            "ng-flow" | "ng-flow-box" | "gtk-flow-box" => "GtkFlowBox",
            "ng-anchor" | "#comment" => "GtkAnchorComment",
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
