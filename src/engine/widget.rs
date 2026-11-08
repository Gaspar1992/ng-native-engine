use super::node::{EngineError, NodeId};

#[derive(Debug)]
pub enum WidgetKind {
    Window {
        title: String,
        child: Option<NodeId>,
    },
    Box {
        vertical: bool,
        spacing: u32,
        children: Vec<NodeId>,
    },
    Button {
        label: String,
    },
    Label {
        text: String,
    },
}

impl WidgetKind {
    pub fn from_tag(tag: &str) -> Result<Self, EngineError> {
        match tag {
            "ng-window" | "adw-window" => Ok(Self::Window {
                title: String::from("Untitled"),
                child: None,
            }),
            "ng-box" | "gtk-box" => Ok(Self::Box {
                vertical: true,
                spacing: 0,
                children: Vec::new(),
            }),
            "ng-button" | "gtk-button" => Ok(Self::Button {
                label: String::new(),
            }),
            "ng-label" | "gtk-label" => Ok(Self::Label {
                text: String::new(),
            }),
            other => Err(EngineError::UnknownTag(other.to_string())),
        }
    }

    pub fn set_text(&mut self, id: NodeId, text: impl Into<String>) -> Result<(), EngineError> {
        match self {
            Self::Window { title, .. } => *title = text.into(),
            Self::Button { label } => *label = text.into(),
            Self::Label { text: label_text } => *label_text = text.into(),
            Self::Box { .. } => return Err(EngineError::InvalidParent(id)),
        }
        Ok(())
    }

    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), EngineError> {
        match self {
            Self::Window { child, .. } => {
                *child = Some(child_id);
                Ok(())
            }
            Self::Box { children, .. } => {
                children.push(child_id);
                Ok(())
            }
            Self::Button { .. } | Self::Label { .. } => Err(EngineError::InvalidParent(parent_id)),
        }
    }
}
