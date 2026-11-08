use super::gtk4_sys::{Gtk4Sys, GtkHandle};
use super::linux_gtk::{GtkLinuxBackend, GtkWidgetType};
use crate::engine::NodeId;
use std::fmt::Write as _;

impl GtkLinuxBackend {
    pub(super) fn rebuild_css_stylesheet(&self) -> String {
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

    pub(super) fn parse_px_i32(raw: &str) -> Option<i32> {
        raw.trim().trim_end_matches("px").trim().parse::<i32>().ok()
    }

    pub(super) fn apply_native_attribute(&self, node: NodeId, key: &str, value: &str) {
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
            _ => Self::apply_interactive_attribute(gtk, widget.kind, handle, key, value),
        }
    }

    fn apply_interactive_attribute(
        gtk: &Gtk4Sys,
        kind: GtkWidgetType,
        handle: GtkHandle,
        key: &str,
        value: &str,
    ) {
        match (kind, key) {
            (GtkWidgetType::Entry, "value" | "text") => gtk.set_entry_text(handle, value),
            (GtkWidgetType::Entry, "placeholder") => gtk.set_entry_placeholder(handle, value),
            (GtkWidgetType::Switch, "active" | "checked" | "value") => {
                gtk.set_switch_active(handle, value == "true" || value == "1");
            }
            (GtkWidgetType::CheckButton, "active" | "checked" | "value") => {
                gtk.set_check_button_active(handle, value == "true" || value == "1");
            }
            (GtkWidgetType::CheckButton, "label") => gtk.set_check_button_label(handle, value),
            (GtkWidgetType::Slider, "value") => {
                if let Ok(v) = value.trim().parse::<f64>() {
                    gtk.set_slider_value(handle, v);
                }
            }
            (GtkWidgetType::DropDown, "options") => {
                gtk.set_drop_down_options(handle, value);
            }
            (GtkWidgetType::DropDown, "selected" | "value") => {
                if let Ok(idx) = value.trim().parse::<u32>() {
                    gtk.set_drop_down_selected(handle, idx);
                }
            }
            _ => {}
        }
    }
}
