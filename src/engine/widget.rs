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
    Image {
        src: String,
        width: u32,
        height: u32,
    },
    LevelBar {
        value: f64,
        min: f64,
        max: f64,
    },
    FlowBox {
        row_spacing: u32,
        column_spacing: u32,
        min_children_per_line: u32,
        max_children_per_line: u32,
        homogeneous: bool,
        children: Vec<NodeId>,
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
            "ng-image" | "gtk-picture" | "ng-picture" => Ok(Self::Image {
                src: String::new(),
                width: 0,
                height: 0,
            }),
            "ng-level-bar" | "gtk-level-bar" | "ng-progress-bar" => Ok(Self::LevelBar {
                value: 0.0,
                min: 0.0,
                max: 255.0,
            }),
            "ng-flow" | "ng-flow-box" | "gtk-flow-box" => Ok(Self::FlowBox {
                row_spacing: 0,
                column_spacing: 0,
                min_children_per_line: 0,
                max_children_per_line: 0,
                homogeneous: false,
                children: Vec::new(),
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
            }
            | Self::Image {
                src: label_text, ..
            } => {
                *label_text = text.into();
            }
            Self::Anchor { comment } => *comment = text.into(),
            Self::Box { .. }
            | Self::Scroll { .. }
            | Self::Grid { .. }
            | Self::FlowBox { .. }
            | Self::Separator { .. }
            | Self::Switch { .. }
            | Self::Slider { .. }
            | Self::DropDown { .. }
            | Self::LevelBar { .. } => {
                return Err(EngineError::InvalidParent(id));
            }
        }
        Ok(())
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
            | Self::FlowBox { children, .. }
            | Self::HeaderBar { children, .. } => {
                if !children.contains(&child_id) {
                    children.push(child_id);
                }
                Ok(())
            }
            Self::Button { .. }
            | Self::Label { .. }
            | Self::Image { .. }
            | Self::LevelBar { .. }
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
            | Self::FlowBox { children, .. }
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
            | Self::Image { .. }
            | Self::LevelBar { .. }
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
            | Self::FlowBox { children, .. }
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
            | Self::Image { .. }
            | Self::LevelBar { .. }
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
            "ng-image",
            "ng-level-bar",
            "ng-flow",
            "ng-flow-box",
            "gtk-flow-box",
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
