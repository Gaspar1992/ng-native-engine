#![allow(unsafe_code)]

use super::PlatformBackend;
use super::gtk4_sys::{Gtk4Sys, GtkHandle};
use crate::engine::{EventPayload, NativeEvent, NodeId, RenderCommand, StyleId, StyleRule};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Write as _;

thread_local! {
    static PENDING_EVENTS: RefCell<Vec<NativeEvent>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "C" fn on_gtk_button_clicked(_btn: *mut c_void, user_data: *mut c_void) {
    let packed = user_data as usize;
    let index = packed >> 32;
    #[allow(clippy::cast_possible_truncation)]
    let generation = (packed & 0xFFFF_FFFF) as u32;
    PENDING_EVENTS.with(|queue| {
        queue.borrow_mut().push(NativeEvent {
            node: NodeId { index, generation },
            event_name: String::from("clicked"),
            payload: EventPayload::None,
        });
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GtkWidgetType {
    Window,
    HeaderBar,
    Scroll,
    Box,
    Grid,
    Separator,
    Label,
    Button,
    Anchor,
}

struct NativeGtkWidget {
    handle: Option<GtkHandle>,
    kind: GtkWidgetType,
    children: Vec<NodeId>,
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
        self.step_native_events(may_block)
            .into_iter()
            .map(|ev| ev.node)
            .collect()
    }

    pub fn step_native_events(&self, may_block: bool) -> Vec<NativeEvent> {
        if let Some(gtk) = &self.gtk {
            gtk.step_main_context(may_block);
        }
        PENDING_EVENTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
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

    fn parse_px_i32(raw: &str) -> Option<i32> {
        raw.trim().trim_end_matches("px").trim().parse::<i32>().ok()
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
                    GtkWidgetType::Label => gtk.set_label_text(handle, text),
                    GtkWidgetType::Button => gtk.set_button_label(handle, text),
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
                let packed = (id.index << 32) | (id.generation as usize);
                gtk.connect_clicked(btn, on_gtk_button_clicked, packed);
                (Some(btn), GtkWidgetType::Button)
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

    fn apply_native_attribute(&self, node: NodeId, key: &str, value: &str) {
        let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.get(&node)) else {
            return;
        };
        let Some(handle) = widget.handle else { return };
        match key {
            "title" if widget.kind == GtkWidgetType::Window => {
                gtk.set_window_title(handle, value);
            }
            "show-title-buttons" if widget.kind == GtkWidgetType::HeaderBar => {
                gtk.set_header_bar_show_title_buttons(handle, value != "false" && value != "0");
            }
            "min-height" if widget.kind == GtkWidgetType::Scroll => {
                if let Some(px) = Self::parse_px_i32(value) {
                    gtk.set_scrolled_min_height(handle, px);
                }
            }
            "orientation"
                if matches!(widget.kind, GtkWidgetType::Box | GtkWidgetType::Separator) =>
            {
                gtk.set_orientation(handle, value != "horizontal");
            }
            "spacing" | "gap" if widget.kind == GtkWidgetType::Box => {
                if let Some(px) = Self::parse_px_i32(value) {
                    gtk.set_box_spacing(handle, px);
                }
            }
            "spacing" | "gap" if widget.kind == GtkWidgetType::Grid => {
                if let Some(px) = Self::parse_px_i32(value)
                    && let Ok(u) = u32::try_from(px)
                {
                    gtk.set_grid_row_spacing(handle, u);
                    gtk.set_grid_column_spacing(handle, u);
                }
            }
            "row-spacing" if widget.kind == GtkWidgetType::Grid => {
                if let Some(px) = Self::parse_px_i32(value)
                    && let Ok(u) = u32::try_from(px)
                {
                    gtk.set_grid_row_spacing(handle, u);
                }
            }
            "column-spacing" if widget.kind == GtkWidgetType::Grid => {
                if let Some(px) = Self::parse_px_i32(value)
                    && let Ok(u) = u32::try_from(px)
                {
                    gtk.set_grid_column_spacing(handle, u);
                }
            }
            "hexpand" => gtk.set_hexpand(handle, value == "true" || value == "1"),
            "vexpand" => gtk.set_vexpand(handle, value == "true" || value == "1"),
            "halign" => gtk.set_halign(handle, value),
            "valign" => gtk.set_valign(handle, value),
            "margin" => {
                if let Some(px) = Self::parse_px_i32(value) {
                    gtk.set_uniform_margin(handle, px);
                }
            }
            "margin-top" | "margin-bottom" | "margin-start" | "margin-end" => {
                if let Some(px) = Self::parse_px_i32(value) {
                    gtk.set_margin_side(handle, key, px);
                }
            }
            _ => {}
        }
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
