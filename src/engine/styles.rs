use super::node::{EngineError, NodeId, StyleId};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleRule {
    pub class_name: String,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub padding: u16,
    pub spacing: u16,
}

/// Game-engine style Resource Manager for UI styles.
pub struct StyleRegistry {
    rules_by_id: HashMap<StyleId, StyleRule>,
    ids_by_name: HashMap<String, StyleId>,
    node_styles: HashMap<NodeId, Vec<StyleId>>,
    next_id: u32,
}

impl Default for StyleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleRegistry {
    pub fn new() -> Self {
        Self {
            rules_by_id: HashMap::new(),
            ids_by_name: HashMap::new(),
            node_styles: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register_rule(&mut self, rule: StyleRule) -> StyleId {
        if let Some(&existing_id) = self.ids_by_name.get(&rule.class_name) {
            self.rules_by_id.insert(existing_id, rule);
            return existing_id;
        }

        let id = StyleId(self.next_id);
        self.next_id += 1;
        self.ids_by_name.insert(rule.class_name.clone(), id);
        self.rules_by_id.insert(id, rule);
        id
    }

    pub fn attach_class(&mut self, node: NodeId, class_name: &str) -> Result<StyleId, EngineError> {
        let style_id = *self
            .ids_by_name
            .get(class_name)
            .ok_or_else(|| EngineError::StyleNotFound(class_name.to_string()))?;

        let list = self.node_styles.entry(node).or_default();
        if !list.contains(&style_id) {
            list.push(style_id);
        }
        Ok(style_id)
    }

    pub fn get_rule(&self, id: StyleId) -> Option<&StyleRule> {
        self.rules_by_id.get(&id)
    }

    pub fn get_rule_by_name(&self, class_name: &str) -> Option<&StyleRule> {
        let id = self.ids_by_name.get(class_name)?;
        self.rules_by_id.get(id)
    }

    pub fn node_classes(&self, node: NodeId) -> Vec<&str> {
        self.node_styles
            .get(&node)
            .into_iter()
            .flatten()
            .filter_map(|id| self.get_rule(*id).map(|r| r.class_name.as_str()))
            .collect()
    }

    pub fn remove_node(&mut self, node: NodeId) {
        self.node_styles.remove(&node);
    }
}
