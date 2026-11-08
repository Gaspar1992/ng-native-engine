use super::node::{EngineError, NodeId, StyleId};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleRule {
    pub class_name: String,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub padding: u16,
    pub spacing: u16,
    pub border_radius: Option<u16>,
    pub font_size: Option<u16>,
    pub font_weight: Option<u16>,
}

impl StyleRule {
    /// Applies an optional extended `key=value` declaration (`radius`, `font-size`, `font-weight`).
    /// Returns `false` when the key is unknown or the value is not a valid unsigned integer.
    pub fn apply_extension(&mut self, key: &str, raw_value: &str) -> bool {
        let Ok(value) = raw_value.trim().trim_end_matches("px").parse::<u16>() else {
            return false;
        };
        match key {
            "radius" | "border-radius" => self.border_radius = Some(value),
            "font-size" => self.font_size = Some(value),
            "font-weight" => self.font_weight = Some(value),
            _ => return false,
        }
        true
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_styles_by_class_name_and_tracks_node_classes() {
        let mut reg = StyleRegistry::new();
        let node = NodeId {
            index: 0,
            generation: 1,
        };

        let id1 = reg.register_rule(StyleRule {
            class_name: String::from("card"),
            background: Some(String::from("#111111")),
            foreground: None,
            padding: 8,
            spacing: 4,
            ..StyleRule::default()
        });
        let id2 = reg.register_rule(StyleRule {
            class_name: String::from("card"),
            background: Some(String::from("#222222")),
            foreground: None,
            padding: 16,
            spacing: 8,
            ..StyleRule::default()
        });
        assert_eq!(id1, id2);
        assert_eq!(reg.get_rule(id1).unwrap().padding, 16);

        assert_eq!(
            reg.attach_class(node, "missing"),
            Err(EngineError::StyleNotFound(String::from("missing")))
        );
        assert_eq!(reg.attach_class(node, "card").unwrap(), id1);
        assert_eq!(reg.node_classes(node), vec!["card"]);

        reg.remove_node(node);
        assert_eq!(reg.node_classes(node), Vec::<&str>::new());
    }

    #[test]
    fn applies_typography_and_radius_extensions_and_rejects_invalid_ones() {
        let mut rule = StyleRule {
            class_name: String::from("card"),
            ..StyleRule::default()
        };
        assert!(rule.apply_extension("radius", "12px"));
        assert!(rule.apply_extension("font-size", "18"));
        assert!(rule.apply_extension("font-weight", "700"));
        assert!(!rule.apply_extension("font-weight", "bold"));
        assert!(!rule.apply_extension("letter-spacing", "2"));
        assert_eq!(rule.border_radius, Some(12));
        assert_eq!(rule.font_size, Some(18));
        assert_eq!(rule.font_weight, Some(700));
    }
}
