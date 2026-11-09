use super::gtk4_sys::{Gtk4Sys, GtkHandle};
use super::linux_gtk::{GtkLinuxBackend, GtkWidgetType};
use crate::engine::{NodeId, StyleRule};
use std::fmt::Write as _;

/// Translates a single engine `StyleRule` into a GTK4 CSS class block (pure, unit-testable).
pub(super) fn render_gtk_css_rule(rule: &StyleRule) -> String {
    let mut css = String::new();
    let _ = writeln!(css, ".{} {{", rule.class_name);
    if let Some(bg) = &rule.background {
        let _ = writeln!(css, "  background: {bg};");
    }
    if let Some(fg) = &rule.foreground {
        let _ = writeln!(css, "  color: {fg};");
    }
    if rule.padding > 0 {
        let _ = writeln!(css, "  padding: {}px;", rule.padding);
    }
    if let Some(radius) = rule.border_radius {
        let _ = writeln!(css, "  border-radius: {radius}px;");
    }
    if let Some(size) = rule.font_size {
        let _ = writeln!(css, "  font-size: {size}px;");
    }
    if let Some(weight) = rule.font_weight {
        let _ = writeln!(css, "  font-weight: {weight};");
    }
    let _ = writeln!(css, "}}");
    css
}

impl GtkLinuxBackend {
    pub(super) fn rebuild_css_stylesheet(&self) -> String {
        let mut css = String::from(
            "window { background: #1e1e1e; color: #ffffff; }\n\
             headerbar, windowhandle { min-height: 46px; }\n",
        );
        for rule in self.styles.values() {
            css.push_str(&render_gtk_css_rule(rule));
        }
        css
    }

    pub(super) fn parse_px_i32(raw: &str) -> Option<i32> {
        raw.trim().trim_end_matches("px").trim().parse::<i32>().ok()
    }

    fn apply_container_attribute(
        gtk: &Gtk4Sys,
        kind: GtkWidgetType,
        handle: GtkHandle,
        key: &str,
        value: &str,
    ) -> bool {
        match key {
            "spacing" | "gap" if matches!(kind, GtkWidgetType::Grid | GtkWidgetType::FlowBox) => {
                if let Some(px) = Self::parse_px_i32(value)
                    && let Ok(u) = u32::try_from(px)
                {
                    if kind == GtkWidgetType::Grid {
                        gtk.set_grid_row_spacing(handle, u);
                        gtk.set_grid_column_spacing(handle, u);
                    } else {
                        gtk.set_flow_box_row_spacing(handle, u);
                        gtk.set_flow_box_column_spacing(handle, u);
                    }
                }
                true
            }
            "row-spacing" if matches!(kind, GtkWidgetType::Grid | GtkWidgetType::FlowBox) => {
                if let Some(px) = Self::parse_px_i32(value)
                    && let Ok(u) = u32::try_from(px)
                {
                    if kind == GtkWidgetType::Grid {
                        gtk.set_grid_row_spacing(handle, u);
                    } else {
                        gtk.set_flow_box_row_spacing(handle, u);
                    }
                }
                true
            }
            "column-spacing" if matches!(kind, GtkWidgetType::Grid | GtkWidgetType::FlowBox) => {
                if let Some(px) = Self::parse_px_i32(value)
                    && let Ok(u) = u32::try_from(px)
                {
                    if kind == GtkWidgetType::Grid {
                        gtk.set_grid_column_spacing(handle, u);
                    } else {
                        gtk.set_flow_box_column_spacing(handle, u);
                    }
                }
                true
            }
            "min-children" | "min-per-row" if kind == GtkWidgetType::FlowBox => {
                if let Ok(u) = value.trim().parse::<u32>() {
                    gtk.set_flow_box_min_children_per_line(handle, u);
                }
                true
            }
            "max-children" | "max-per-row" if kind == GtkWidgetType::FlowBox => {
                if let Ok(u) = value.trim().parse::<u32>() {
                    gtk.set_flow_box_max_children_per_line(handle, u);
                }
                true
            }
            "homogeneous" if kind == GtkWidgetType::FlowBox => {
                gtk.set_flow_box_homogeneous(handle, value != "false" && value != "0");
                true
            }
            _ => false,
        }
    }

    pub(super) fn apply_native_attribute(&mut self, node: NodeId, key: &str, value: &str) {
        let (Some(gtk), Some(widget)) = (&self.gtk, self.widgets.get(&node)) else {
            return;
        };
        let Some(handle) = widget.handle else { return };
        if Self::apply_container_attribute(gtk, widget.kind, handle, key, value) {
            return;
        }
        match key {
            "title" if widget.kind == GtkWidgetType::Window => {
                gtk.set_window_title(handle, value);
            }
            "title" if widget.kind == GtkWidgetType::HeaderBar => {
                let entry = self.header_titles.entry(node).or_default();
                entry.0 = value.to_string();
                let (t, s) = (entry.0.clone(), entry.1.clone());
                gtk.set_header_bar_title(handle, &t, &s);
            }
            "subtitle" if widget.kind == GtkWidgetType::HeaderBar => {
                let entry = self.header_titles.entry(node).or_default();
                entry.1 = value.to_string();
                let (t, s) = (entry.0.clone(), entry.1.clone());
                gtk.set_header_bar_title(handle, &t, &s);
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
            "hexpand" => gtk.set_hexpand(handle, value == "true" || value == "1"),
            "vexpand" => gtk.set_vexpand(handle, value == "true" || value == "1"),
            "visible" => gtk.set_widget_visible(handle, value != "false" && value != "0"),
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
            (GtkWidgetType::Image, "src") => gtk.set_image_file(handle, value),
            (GtkWidgetType::Image, "width") => {
                if let Some(w) = Self::parse_px_i32(value) {
                    gtk.set_size_request(handle, w, -1);
                }
            }
            (GtkWidgetType::Image, "height") => {
                if let Some(h) = Self::parse_px_i32(value) {
                    gtk.set_size_request(handle, -1, h);
                }
            }
            (GtkWidgetType::LevelBar, "value") => {
                if let Ok(v) = value.trim().parse::<f64>() {
                    gtk.set_level_bar_value(handle, v);
                }
            }
            (GtkWidgetType::LevelBar, "max") => {
                if let Ok(m) = value.trim().parse::<f64>() {
                    gtk.set_level_bar_max(handle, m);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_typography_and_radius_only_when_declared() {
        let plain = StyleRule {
            class_name: String::from("plain"),
            padding: 8,
            ..StyleRule::default()
        };
        let plain_css = render_gtk_css_rule(&plain);
        assert!(plain_css.contains("padding: 8px;"));
        assert!(!plain_css.contains("border-radius"));

        let hero = StyleRule {
            class_name: String::from("hero"),
            foreground: Some(String::from("#78aeed")),
            border_radius: Some(12),
            font_size: Some(22),
            font_weight: Some(800),
            ..StyleRule::default()
        };
        let hero_css = render_gtk_css_rule(&hero);
        assert!(hero_css.starts_with(".hero {"));
        assert!(hero_css.contains("color: #78aeed;"));
        assert!(hero_css.contains("border-radius: 12px;"));
        assert!(hero_css.contains("font-size: 22px;"));
        assert!(hero_css.contains("font-weight: 800;"));
        assert!(!hero_css.contains("padding"));
    }
}
