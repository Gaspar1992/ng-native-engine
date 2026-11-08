use super::icons::WindowIconAsset;
use super::node::{EngineError, NodeId};

#[derive(Debug)]
pub enum WidgetKind {
    Window {
        title: String,
        icon: Option<WindowIconAsset>,
        header_bar: Option<NodeId>,
        child: Option<NodeId>,
    },
    HeaderBar {
        title: String,
        subtitle: Option<String>,
        show_title_buttons: bool,
        children: Vec<NodeId>,
    },
    Scroll {
        min_height: u32,
        child: Option<NodeId>,
    },
    Box {
        vertical: bool,
        spacing: u32,
        children: Vec<NodeId>,
    },
    Grid {
        row_spacing: u32,
        column_spacing: u32,
        children: Vec<NodeId>,
    },
    Separator {
        vertical: bool,
    },
    Button {
        label: String,
    },
    Label {
        text: String,
    },
    Entry {
        text: String,
        placeholder: String,
    },
    Switch {
        active: bool,
    },
    CheckButton {
        label: String,
        active: bool,
    },
    Slider {
        value: i64,
        min: i64,
        max: i64,
    },
    DropDown {
        options: Vec<String>,
        selected: usize,
    },
    Anchor {
        comment: String,
    },
}

impl WidgetKind {
    pub fn from_tag(tag: &str) -> Result<Self, EngineError> {
        match tag {
            "ng-window" | "adw-window" => Ok(Self::Window {
                title: String::from("Untitled"),
                icon: None,
                header_bar: None,
                child: None,
            }),
            "ng-header-bar" | "adw-header-bar" | "gtk-header-bar" => Ok(Self::HeaderBar {
                title: String::new(),
                subtitle: None,
                show_title_buttons: true,
                children: Vec::new(),
            }),
            "ng-scroll" | "gtk-scrolled-window" => Ok(Self::Scroll {
                min_height: 0,
                child: None,
            }),
            "ng-box" | "gtk-box" => Ok(Self::Box {
                vertical: true,
                spacing: 0,
                children: Vec::new(),
            }),
            "ng-grid" | "gtk-grid" => Ok(Self::Grid {
                row_spacing: 0,
                column_spacing: 0,
                children: Vec::new(),
            }),
            "ng-separator" | "gtk-separator" => Ok(Self::Separator { vertical: false }),
            "ng-button" | "gtk-button" => Ok(Self::Button {
                label: String::new(),
            }),
            "ng-label" | "gtk-label" => Ok(Self::Label {
                text: String::new(),
            }),
            "ng-entry" | "gtk-entry" => Ok(Self::Entry {
                text: String::new(),
                placeholder: String::new(),
            }),
            "ng-switch" | "gtk-switch" => Ok(Self::Switch { active: false }),
            "ng-checkbox" | "ng-check" | "gtk-check-button" => Ok(Self::CheckButton {
                label: String::new(),
                active: false,
            }),
            "ng-slider" | "ng-scale" | "gtk-scale" => Ok(Self::Slider {
                value: 0,
                min: 0,
                max: 100,
            }),
            "ng-select" | "ng-dropdown" | "gtk-drop-down" => Ok(Self::DropDown {
                options: Vec::new(),
                selected: 0,
            }),
            "ng-anchor" | "#comment" => Ok(Self::Anchor {
                comment: String::new(),
            }),
            other => Err(EngineError::UnknownTag(other.to_string())),
        }
    }

    pub fn new_comment(comment: impl Into<String>) -> Self {
        Self::Anchor {
            comment: comment.into(),
        }
    }

    pub fn is_header_bar(&self) -> bool {
        matches!(self, Self::HeaderBar { .. })
    }

    pub fn set_text(&mut self, id: NodeId, text: impl Into<String>) -> Result<(), EngineError> {
        match self {
            Self::Window { title, .. } | Self::HeaderBar { title, .. } => *title = text.into(),
            Self::Button { label } | Self::CheckButton { label, .. } => *label = text.into(),
            Self::Label { text: label_text }
            | Self::Entry {
                text: label_text, ..
            } => {
                *label_text = text.into();
            }
            Self::Anchor { comment } => *comment = text.into(),
            Self::Box { .. }
            | Self::Scroll { .. }
            | Self::Grid { .. }
            | Self::Separator { .. }
            | Self::Switch { .. }
            | Self::Slider { .. }
            | Self::DropDown { .. } => {
                return Err(EngineError::InvalidParent(id));
            }
        }
        Ok(())
    }

    pub fn set_attribute(&mut self, key: &str, value: &str) {
        match self {
            Self::Window { title, .. } if key == "title" => {
                *title = value.to_string();
            }
            Self::HeaderBar {
                title,
                subtitle,
                show_title_buttons,
                ..
            } => match key {
                "title" => *title = value.to_string(),
                "subtitle" => *subtitle = (!value.is_empty()).then(|| value.to_string()),
                "show-title-buttons" => *show_title_buttons = value != "false" && value != "0",
                _ => {}
            },
            Self::Scroll { min_height, .. } if key == "min-height" => {
                if let Ok(parsed) = Self::parse_px(value) {
                    *min_height = parsed;
                }
            }
            Self::Box {
                vertical, spacing, ..
            } => match key {
                "orientation" => *vertical = value != "horizontal",
                "spacing" | "gap" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *spacing = parsed;
                    }
                }
                _ => {}
            },
            Self::Grid {
                row_spacing,
                column_spacing,
                ..
            } => match key {
                "spacing" | "gap" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *row_spacing = parsed;
                        *column_spacing = parsed;
                    }
                }
                "row-spacing" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *row_spacing = parsed;
                    }
                }
                "column-spacing" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *column_spacing = parsed;
                    }
                }
                _ => {}
            },
            Self::Separator { vertical } if key == "orientation" => {
                *vertical = value == "vertical";
            }
            _ => self.set_interactive_attribute(key, value),
        }
    }

    fn set_interactive_attribute(&mut self, key: &str, value: &str) {
        match self {
            Self::Entry {
                text, placeholder, ..
            } => match key {
                "value" | "text" => value.clone_into(text),
                "placeholder" => value.clone_into(placeholder),
                _ => {}
            },
            Self::Switch { active } => {
                if key == "active" || key == "checked" || key == "value" {
                    *active = value == "true" || value == "1";
                }
            }
            Self::CheckButton { label, active } => match key {
                "label" => value.clone_into(label),
                "active" | "checked" | "value" => *active = value == "true" || value == "1",
                _ => {}
            },
            Self::Slider {
                value: val,
                min,
                max,
            } => match key {
                "value" => {
                    if let Ok(v) = value.trim().parse::<i64>() {
                        *val = v.clamp(*min, *max);
                    }
                }
                "min" => {
                    if let Ok(m) = value.trim().parse::<i64>() {
                        *min = m;
                    }
                }
                "max" => {
                    if let Ok(m) = value.trim().parse::<i64>() {
                        *max = m;
                    }
                }
                _ => {}
            },
            Self::DropDown { options, selected } => match key {
                "options" => {
                    *options = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "selected" | "value" => {
                    if let Ok(idx) = value.trim().parse::<usize>() {
                        *selected = idx;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn parse_px(raw: &str) -> Result<u32, std::num::ParseIntError> {
        let trimmed = raw.trim().trim_end_matches("px").trim();
        trimmed.parse::<u32>()
    }

    pub fn set_icon(&mut self, id: NodeId, icon_asset: WindowIconAsset) -> Result<(), EngineError> {
        match self {
            Self::Window { icon, .. } => {
                *icon = Some(icon_asset);
                Ok(())
            }
            _ => Err(EngineError::InvalidParent(id)),
        }
    }

    pub fn add_child(
        &mut self,
        parent_id: NodeId,
        child_id: NodeId,
        is_header_bar: bool,
    ) -> Result<(), EngineError> {
        match self {
            Self::Window {
                header_bar, child, ..
            } => {
                if is_header_bar {
                    *header_bar = Some(child_id);
                } else {
                    *child = Some(child_id);
                }
                Ok(())
            }
            Self::Scroll { child, .. } => {
                *child = Some(child_id);
                Ok(())
            }
            Self::Box { children, .. }
            | Self::Grid { children, .. }
            | Self::HeaderBar { children, .. } => {
                if !children.contains(&child_id) {
                    children.push(child_id);
                }
                Ok(())
            }
            Self::Button { .. }
            | Self::Label { .. }
            | Self::Separator { .. }
            | Self::Entry { .. }
            | Self::Switch { .. }
            | Self::CheckButton { .. }
            | Self::Slider { .. }
            | Self::DropDown { .. }
            | Self::Anchor { .. } => Err(EngineError::InvalidParent(parent_id)),
        }
    }

    pub fn insert_before(
        &mut self,
        parent_id: NodeId,
        new_child: NodeId,
        ref_child: Option<NodeId>,
    ) -> Result<(), EngineError> {
        match self {
            Self::Box { children, .. }
            | Self::Grid { children, .. }
            | Self::HeaderBar { children, .. } => {
                children.retain(|c| *c != new_child);
                if let Some(ref_id) = ref_child {
                    let pos = children
                        .iter()
                        .position(|c| *c == ref_id)
                        .ok_or(EngineError::InvalidNode(ref_id))?;
                    children.insert(pos, new_child);
                } else {
                    children.push(new_child);
                }
                Ok(())
            }
            Self::Window { child, .. } | Self::Scroll { child, .. } => {
                *child = Some(new_child);
                Ok(())
            }
            Self::Button { .. }
            | Self::Label { .. }
            | Self::Separator { .. }
            | Self::Entry { .. }
            | Self::Switch { .. }
            | Self::CheckButton { .. }
            | Self::Slider { .. }
            | Self::DropDown { .. }
            | Self::Anchor { .. } => Err(EngineError::InvalidParent(parent_id)),
        }
    }

    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), EngineError> {
        match self {
            Self::Window {
                header_bar, child, ..
            } => {
                if *header_bar == Some(child_id) {
                    *header_bar = None;
                    Ok(())
                } else if *child == Some(child_id) {
                    *child = None;
                    Ok(())
                } else {
                    Err(EngineError::InvalidNode(child_id))
                }
            }
            Self::Scroll { child, .. } => {
                if *child == Some(child_id) {
                    *child = None;
                    Ok(())
                } else {
                    Err(EngineError::InvalidNode(child_id))
                }
            }
            Self::Box { children, .. }
            | Self::Grid { children, .. }
            | Self::HeaderBar { children, .. } => {
                let before = children.len();
                children.retain(|c| *c != child_id);
                if children.len() == before {
                    Err(EngineError::InvalidNode(child_id))
                } else {
                    Ok(())
                }
            }
            Self::Button { .. }
            | Self::Label { .. }
            | Self::Separator { .. }
            | Self::Entry { .. }
            | Self::Switch { .. }
            | Self::CheckButton { .. }
            | Self::Slider { .. }
            | Self::DropDown { .. }
            | Self::Anchor { .. } => Err(EngineError::InvalidParent(parent_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_supported_widget_tags_and_rejects_unknown() {
        for tag in [
            "ng-window",
            "ng-header-bar",
            "ng-scroll",
            "ng-box",
            "ng-grid",
            "ng-separator",
            "ng-button",
            "ng-label",
            "ng-entry",
            "ng-switch",
            "ng-checkbox",
            "ng-slider",
            "ng-select",
            "ng-anchor",
        ] {
            assert!(WidgetKind::from_tag(tag).is_ok(), "Failed to parse {tag}");
        }
        assert_eq!(
            WidgetKind::from_tag("ng-unknown").unwrap_err(),
            EngineError::UnknownTag(String::from("ng-unknown"))
        );
    }

    #[test]
    fn updates_layout_and_interactive_form_attributes() {
        let mut bx = WidgetKind::from_tag("ng-box").unwrap();
        bx.set_attribute("orientation", "horizontal");
        bx.set_attribute("spacing", "24px");
        assert!(matches!(
            bx,
            WidgetKind::Box {
                vertical: false,
                spacing: 24,
                ..
            }
        ));

        let mut entry = WidgetKind::from_tag("ng-entry").unwrap();
        entry.set_attribute("placeholder", "Email");
        entry.set_attribute("value", "dev@angular.io");
        assert!(matches!(
            entry,
            WidgetKind::Entry { ref text, ref placeholder }
            if text == "dev@angular.io" && placeholder == "Email"
        ));

        let mut sw = WidgetKind::from_tag("ng-switch").unwrap();
        sw.set_attribute("active", "true");
        assert!(matches!(sw, WidgetKind::Switch { active: true }));
    }

    #[test]
    fn enforces_parent_child_hierarchy_invariants() {
        let parent_id = NodeId {
            index: 0,
            generation: 1,
        };
        let c1 = NodeId {
            index: 1,
            generation: 1,
        };
        let c2 = NodeId {
            index: 2,
            generation: 1,
        };

        let mut btn = WidgetKind::from_tag("ng-button").unwrap();
        assert_eq!(
            btn.add_child(parent_id, c1, false).unwrap_err(),
            EngineError::InvalidParent(parent_id)
        );

        let mut bx = WidgetKind::from_tag("ng-box").unwrap();
        bx.add_child(parent_id, c1, false).unwrap();
        bx.insert_before(parent_id, c2, Some(c1)).unwrap();
        if let WidgetKind::Box { children, .. } = &bx {
            assert_eq!(children, &vec![c2, c1]);
        }
        bx.remove_child(parent_id, c2).unwrap();
        if let WidgetKind::Box { children, .. } = &bx {
            assert_eq!(children, &vec![c1]);
        }
    }
}
