use super::RenderEngine;
use super::node::{EngineError, NodeId};
use super::widget::WidgetKind;
use crate::platforms::PlatformBackend;
use std::fmt::Write;

impl<B: PlatformBackend> RenderEngine<B> {
    pub fn print_tree(&self, id: NodeId, indent: usize) -> Result<(), EngineError> {
        let pad = "  ".repeat(indent);
        let classes = self.styles.node_classes(id);
        let class_str = if classes.is_empty() {
            String::new()
        } else {
            format!(" classes={classes:?}")
        };

        let widget = self.arena.get(id)?;
        println!(
            "{pad}{}",
            Self::format_tree_node_header(id, widget, &class_str)
        );

        for child_id in Self::collect_widget_children(widget) {
            if self.arena.get(child_id).is_ok() {
                self.print_tree(child_id, indent + 1)?;
            }
        }
        Ok(())
    }

    fn format_tree_node_header(id: NodeId, widget: &WidgetKind, class_str: &str) -> String {
        let idx = id.index;
        let generation = id.generation;
        match widget {
            WidgetKind::Window { title, icon, .. } => {
                let icon_str = icon.as_ref().map_or_else(String::new, |ic| {
                    format!(" icon=\"{}\" [{}]", ic.source_path, ic.format_resolutions())
                });
                format!("[Window #{idx}:{generation} title=\"{title}\"{icon_str}{class_str}]")
            }
            WidgetKind::HeaderBar {
                title,
                subtitle,
                show_title_buttons,
                ..
            } => {
                let sub_str = subtitle
                    .as_ref()
                    .map_or_else(String::new, |s| format!(" subtitle=\"{s}\""));
                format!(
                    "[HeaderBar #{idx}:{generation} title=\"{title}\"{sub_str} buttons={show_title_buttons}{class_str}]"
                )
            }
            WidgetKind::Scroll { min_height, .. } => {
                format!("[Scroll #{idx}:{generation} min_height={min_height}{class_str}]")
            }
            WidgetKind::Box {
                vertical, spacing, ..
            } => {
                let dir = if *vertical { "vertical" } else { "horizontal" };
                format!("[Box #{idx}:{generation} dir={dir} spacing={spacing}{class_str}]")
            }
            WidgetKind::Grid {
                row_spacing,
                column_spacing,
                ..
            } => format!(
                "[Grid #{idx}:{generation} row_spacing={row_spacing} column_spacing={column_spacing}{class_str}]"
            ),
            WidgetKind::Separator { vertical } => {
                let dir = if *vertical { "vertical" } else { "horizontal" };
                format!("[Separator #{idx}:{generation} dir={dir}{class_str}]")
            }
            WidgetKind::Button { label } => {
                format!("[Button #{idx}:{generation} label=\"{label}\"{class_str}]")
            }
            WidgetKind::Label { text } => {
                format!("[Label #{idx}:{generation} text=\"{text}\"{class_str}]")
            }
            WidgetKind::Entry { text, placeholder } => {
                format!(
                    "[Entry #{idx}:{generation} text=\"{text}\" placeholder=\"{placeholder}\"{class_str}]"
                )
            }
            WidgetKind::Switch { active } => {
                format!("[Switch #{idx}:{generation} active={active}{class_str}]")
            }
            WidgetKind::CheckButton { label, active } => {
                format!(
                    "[CheckButton #{idx}:{generation} label=\"{label}\" active={active}{class_str}]"
                )
            }
            WidgetKind::Slider { value, min, max } => {
                format!("[Slider #{idx}:{generation} value={value} range={min}..{max}{class_str}]")
            }
            WidgetKind::DropDown { options, selected } => {
                format!(
                    "[DropDown #{idx}:{generation} selected={selected} options={options:?}{class_str}]"
                )
            }
            WidgetKind::Anchor { comment } => {
                format!("[Anchor #{idx}:{generation} comment=\"{comment}\"]")
            }
        }
    }

    fn collect_widget_children(widget: &WidgetKind) -> Vec<NodeId> {
        match widget {
            WidgetKind::Window {
                header_bar, child, ..
            } => header_bar.iter().chain(child.iter()).copied().collect(),
            WidgetKind::Scroll { child, .. } => child.iter().copied().collect(),
            WidgetKind::HeaderBar { children, .. }
            | WidgetKind::Box { children, .. }
            | WidgetKind::Grid { children, .. } => children.clone(),
            WidgetKind::Separator { .. }
            | WidgetKind::Button { .. }
            | WidgetKind::Label { .. }
            | WidgetKind::Entry { .. }
            | WidgetKind::Switch { .. }
            | WidgetKind::CheckButton { .. }
            | WidgetKind::Slider { .. }
            | WidgetKind::DropDown { .. }
            | WidgetKind::Anchor { .. } => Vec::new(),
        }
    }

    pub fn render_window_preview(&self, root_id: NodeId) -> Result<String, EngineError> {
        let root = self.arena.get(root_id)?;
        let (title, icon_opt, header_opt, child_opt) = match root {
            WidgetKind::Window {
                title,
                icon,
                header_bar,
                child,
            } => (title.as_str(), icon.as_ref(), *header_bar, *child),
            _ => return Err(EngineError::InvalidParent(root_id)),
        };

        let width: usize = 76;
        let border = "═".repeat(width);
        let mut out = String::new();
        let _ = writeln!(out, "╔{border}╗");
        let icon_badge = icon_opt.map_or_else(
            || String::from("● ● ●"),
            |ic| format!("[ICO:{}]", ic.format_resolutions()),
        );
        let header = format!(" {icon_badge}  {title} ({})", self.backend.backend_name());
        let _ = writeln!(out, "║{header:<width$}║");
        let _ = writeln!(out, "╠{border}╣");

        if let Some(hb_id) = header_opt
            && self.arena.get(hb_id).is_ok()
        {
            self.format_preview_node(hb_id, width, &mut out)?;
        }

        if let Some(child_id) = child_opt {
            self.format_preview_node(child_id, width, &mut out)?;
        }

        let _ = write!(out, "╚{border}╝");
        Ok(out)
    }

    fn format_preview_node(
        &self,
        id: NodeId,
        width: usize,
        out: &mut String,
    ) -> Result<(), EngineError> {
        let widget = self.arena.get(id)?;
        let style_summary = self.describe_node_styles(id);

        if let Some(line) = Self::format_preview_line(widget, width, &style_summary) {
            let _ = writeln!(out, "║{line:<width$}║");
        }

        for child_id in Self::collect_widget_children(widget) {
            if self.arena.get(child_id).is_ok() {
                self.format_preview_node(child_id, width, out)?;
            }
        }
        Ok(())
    }

    fn format_preview_line(
        widget: &WidgetKind,
        width: usize,
        style_summary: &str,
    ) -> Option<String> {
        match widget {
            WidgetKind::HeaderBar {
                title, subtitle, ..
            } => {
                let sub = subtitle
                    .as_deref()
                    .map_or_else(String::new, |s| format!(" — {s}"));
                Some(format!("  [HeaderBar: {title}{sub}]"))
            }
            WidgetKind::Box {
                vertical, spacing, ..
            } => {
                if style_summary.is_empty() && *vertical && *spacing == 0 {
                    return None;
                }
                let dir = if *vertical { "vertical" } else { "horizontal" };
                let meta = if style_summary.is_empty() {
                    format!("dir:{dir}, gap:{spacing}px")
                } else {
                    format!("{style_summary} dir:{dir}, gap:{spacing}px")
                };
                Some(format!("  [Box {meta}]"))
            }
            WidgetKind::Grid {
                row_spacing,
                column_spacing,
                ..
            } => Some(format!(
                "  [Grid row-gap:{row_spacing}px col-gap:{column_spacing}px]"
            )),
            WidgetKind::Separator { .. } => {
                let sep = "─".repeat(width.saturating_sub(4));
                Some(format!("  {sep}"))
            }
            WidgetKind::Label { text } => Some(if style_summary.is_empty() {
                format!("  {text}")
            } else {
                format!("  {text}  {style_summary}")
            }),
            WidgetKind::Button { label } => Some(if style_summary.is_empty() {
                format!("  [ {label} ]")
            } else {
                format!("  [ {label} ]  {style_summary}")
            }),
            WidgetKind::Entry { text, placeholder } => {
                let display = if text.is_empty() { placeholder } else { text };
                Some(format!("  [Entry: \"{display}\"]"))
            }
            WidgetKind::Switch { active } => {
                let state = if *active { "ON" } else { "OFF" };
                Some(format!("  [Switch: {state}]"))
            }
            WidgetKind::CheckButton { label, active } => {
                let mark = if *active { "x" } else { " " };
                Some(format!("  [{mark}] {label}"))
            }
            WidgetKind::Slider { value, min, max } => {
                Some(format!("  [Slider: {value} ({min}..{max})]"))
            }
            WidgetKind::DropDown { options, selected } => {
                let current = options.get(*selected).map_or("", String::as_str);
                Some(format!("  [Select: \"{current}\" ({selected})]"))
            }
            WidgetKind::Window { .. } | WidgetKind::Scroll { .. } | WidgetKind::Anchor { .. } => {
                None
            }
        }
    }

    fn describe_node_styles(&self, id: NodeId) -> String {
        let classes = self.styles.node_classes(id);
        if classes.is_empty() {
            return String::new();
        }
        let mut details = Vec::new();
        for class_name in classes {
            if let Some(rule) = self.styles.get_rule_by_name(class_name) {
                let bg = rule.background.as_deref().unwrap_or("transparent");
                let fg = rule.foreground.as_deref().unwrap_or("inherit");
                let mut extras = String::new();
                if let Some(radius) = rule.border_radius {
                    let _ = write!(extras, ", radius:{radius}px");
                }
                if let Some(size) = rule.font_size {
                    let _ = write!(extras, ", font-size:{size}px");
                }
                if let Some(weight) = rule.font_weight {
                    let _ = write!(extras, ", font-weight:{weight}");
                }
                details.push(format!(
                    ".{class_name}(bg:{bg}, fg:{fg}, pad:{}px{extras})",
                    rule.padding
                ));
            } else {
                details.push(format!(".{class_name}"));
            }
        }
        format!("{{{}}}", details.join(" "))
    }
}
