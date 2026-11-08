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

    pub fn clear(&mut self) {
        self.listeners.clear();
    }

    pub fn matching_listeners(&self, node: NodeId, event_name: &str) -> Vec<u64> {
        self.listeners
            .iter()
            .filter(|l| l.node == node && l.event_name == event_name)
            .map(|l| l.listener_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_filters_and_cleans_up_node_listeners() {
        let mut events = EventRegistry::new();
        let n1 = NodeId {
            index: 1,
            generation: 1,
        };
        let n2 = NodeId {
            index: 2,
            generation: 1,
        };

        events.register(n1, "clicked", 10);
        events.register(n1, "clicked", 11);
        events.register(n2, "clicked", 20);

        assert_eq!(events.matching_listeners(n1, "clicked"), vec![10, 11]);
        assert!(events.unregister(10));
        assert_eq!(events.matching_listeners(n1, "clicked"), vec![11]);

        events.remove_for_node(n1);
        assert_eq!(events.matching_listeners(n1, "clicked"), Vec::<u64>::new());
        assert_eq!(events.matching_listeners(n2, "clicked"), vec![20]);
    }
}
