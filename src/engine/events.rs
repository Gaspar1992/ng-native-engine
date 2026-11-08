use super::node::NodeId;

#[derive(Debug, Clone)]
pub struct EventListener {
    pub listener_id: u64,
    pub node: NodeId,
    pub event_name: String,
}

/// Manages event subscriptions between native widget signals and Angular callbacks.
#[derive(Default)]
pub struct EventRegistry {
    listeners: Vec<EventListener>,
}

impl EventRegistry {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn register(&mut self, node: NodeId, event_name: impl Into<String>, listener_id: u64) {
        self.listeners.push(EventListener {
            listener_id,
            node,
            event_name: event_name.into(),
        });
    }

    pub fn unregister(&mut self, listener_id: u64) -> bool {
        let initial_len = self.listeners.len();
        self.listeners.retain(|l| l.listener_id != listener_id);
        self.listeners.len() < initial_len
    }

    pub fn remove_for_node(&mut self, node: NodeId) {
        self.listeners.retain(|l| l.node != node);
    }

    pub fn matching_listeners(&self, node: NodeId, event_name: &str) -> Vec<u64> {
        self.listeners
            .iter()
            .filter(|l| l.node == node && l.event_name == event_name)
            .map(|l| l.listener_id)
            .collect()
    }
}
