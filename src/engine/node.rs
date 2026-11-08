#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub index: usize,
    pub generation: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StyleId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowState {
    Unborrowed,
    Shared(u32),
    Exclusive,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    UnknownTag(String),
    InvalidNode(NodeId),
    StaleGeneration {
        id: NodeId,
        current_generation: u32,
    },
    InvalidParent(NodeId),
    BorrowConflict {
        id: NodeId,
        current_state: BorrowState,
    },
    StyleNotFound(String),
    ScriptSyntaxError(String),
    ConfigError(String),
}
