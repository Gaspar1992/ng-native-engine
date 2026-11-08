use super::icons::WindowIconAsset;
use super::node::{NodeId, StyleId};
use super::styles::StyleRule;

/// Game-engine Render Command Buffer entry.
/// Angular mutations are queued during a tick and flushed in batch to the target OS backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderCommand {
    CreateWidget {
        id: NodeId,
        tag: String,
    },
    CreateComment {
        id: NodeId,
        text: String,
    },
    SetText {
        id: NodeId,
        text: String,
    },
    SetAttribute {
        node: NodeId,
        key: String,
        value: String,
    },
    SetWindowIcon {
        node: NodeId,
        icon: WindowIconAsset,
    },
    AppendChild {
        parent: NodeId,
        child: NodeId,
    },
    InsertBefore {
        parent: NodeId,
        child: NodeId,
        ref_child: Option<NodeId>,
    },
    RemoveChild {
        parent: NodeId,
        child: NodeId,
    },
    RegisterStyle {
        style_id: StyleId,
        rule: StyleRule,
    },
    ApplyStyleClass {
        node: NodeId,
        style_id: StyleId,
    },
    DestroyWidget {
        id: NodeId,
    },
}

#[derive(Default)]
pub struct CommandQueue {
    pending: Vec<RenderCommand>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, cmd: RenderCommand) {
        self.pending.push(cmd);
    }

    pub fn drain(&mut self) -> Vec<RenderCommand> {
        std::mem::take(&mut self.pending)
    }
}
