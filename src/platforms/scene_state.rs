use crate::engine::{NodeId, RenderCommand, StyleId, StyleRule, WindowIconAsset};
use std::collections::HashMap;

/// Platform-agnostic native control/view state node shared by WinUI 3 and macOS AppKit backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSceneNode {
    pub native_class: &'static str,
    pub text: String,
    pub attributes: HashMap<String, String>,
    pub style_classes: Vec<StyleId>,
    pub children: Vec<NodeId>,
}

/// Shared retaining state machine for cross-platform native scene trees (WinUI 3, AppKit).
#[derive(Debug, Default, Clone)]
pub struct NativeSceneState {
    nodes: HashMap<NodeId, NativeSceneNode>,
    styles: HashMap<StyleId, StyleRule>,
    window_icons: HashMap<NodeId, WindowIconAsset>,
}

impl NativeSceneState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn style_count(&self) -> usize {
        self.styles.len()
    }

    pub fn get_node(&self, id: NodeId) -> Option<&NativeSceneNode> {
        self.nodes.get(&id)
    }

    pub fn get_style(&self, id: StyleId) -> Option<&StyleRule> {
        self.styles.get(&id)
    }

    pub fn get_window_icon(&self, window: NodeId) -> Option<&WindowIconAsset> {
        self.window_icons.get(&window)
    }

    pub fn apply_command<F>(&mut self, cmd: &RenderCommand, comment_class: &'static str, map_tag: F)
    where
        F: Fn(&str) -> &'static str,
    {
        match cmd {
            RenderCommand::RegisterStyle { style_id, rule } => {
                self.styles.insert(*style_id, rule.clone());
            }
            RenderCommand::CreateWidget { id, tag } => {
                self.nodes.insert(
                    *id,
                    NativeSceneNode {
                        native_class: map_tag(tag),
                        text: String::new(),
                        attributes: HashMap::new(),
                        style_classes: Vec::new(),
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::CreateComment { id, text } => {
                self.nodes.insert(
                    *id,
                    NativeSceneNode {
                        native_class: comment_class,
                        text: text.clone(),
                        attributes: HashMap::new(),
                        style_classes: Vec::new(),
                        children: Vec::new(),
                    },
                );
            }
            RenderCommand::SetText { id, text } => {
                if let Some(node) = self.nodes.get_mut(id) {
                    text.clone_into(&mut node.text);
                }
            }
            RenderCommand::SetAttribute { node, key, value } => {
                if let Some(n) = self.nodes.get_mut(node) {
                    n.attributes.insert(key.clone(), value.clone());
                }
            }
            RenderCommand::SetWindowIcon { node, icon } => {
                self.window_icons.insert(*node, icon.clone());
            }
            RenderCommand::ApplyStyleClass { node, style_id } => {
                if let Some(n) = self.nodes.get_mut(node)
                    && !n.style_classes.contains(style_id)
                {
                    n.style_classes.push(*style_id);
                }
            }
            RenderCommand::AppendChild { parent, child } => {
                if let Some(p) = self.nodes.get_mut(parent) {
                    p.children.push(*child);
                }
            }
            RenderCommand::InsertBefore {
                parent,
                child,
                ref_child,
            } => {
                if let Some(p) = self.nodes.get_mut(parent) {
                    p.children.retain(|c| *c != *child);
                    if let Some(r) = ref_child
                        && let Some(pos) = p.children.iter().position(|c| *c == *r)
                    {
                        p.children.insert(pos, *child);
                    } else {
                        p.children.push(*child);
                    }
                }
            }
            RenderCommand::RemoveChild { parent, child } => {
                if let Some(p) = self.nodes.get_mut(parent) {
                    p.children.retain(|c| *c != *child);
                }
            }
            RenderCommand::DestroyWidget { id } => {
                self.nodes.remove(id);
                self.window_icons.remove(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_hierarchy_styles_and_comment_anchor_insertions() {
        let mut state = NativeSceneState::new();
        let win = NodeId {
            index: 0,
            generation: 1,
        };
        let anchor = NodeId {
            index: 1,
            generation: 1,
        };
        let lbl = NodeId {
            index: 2,
            generation: 1,
        };
        let style = StyleId(1);

        state.apply_command(
            &RenderCommand::RegisterStyle {
                style_id: style,
                rule: StyleRule {
                    class_name: String::from("hero"),
                    background: None,
                    foreground: Some(String::from("#ffffff")),
                    padding: 8,
                    spacing: 4,
                },
            },
            "AnchorComment",
            |_| "NativeWindow",
        );
        state.apply_command(
            &RenderCommand::CreateWidget {
                id: win,
                tag: String::from("ng-window"),
            },
            "AnchorComment",
            |_| "NativeWindow",
        );
        state.apply_command(
            &RenderCommand::CreateComment {
                id: anchor,
                text: String::from("@if"),
            },
            "AnchorComment",
            |_| "NativeWidget",
        );
        state.apply_command(
            &RenderCommand::AppendChild {
                parent: win,
                child: anchor,
            },
            "AnchorComment",
            |_| "NativeWidget",
        );
        state.apply_command(
            &RenderCommand::CreateWidget {
                id: lbl,
                tag: String::from("ng-label"),
            },
            "AnchorComment",
            |_| "NativeLabel",
        );
        state.apply_command(
            &RenderCommand::ApplyStyleClass {
                node: lbl,
                style_id: style,
            },
            "AnchorComment",
            |_| "NativeLabel",
        );
        state.apply_command(
            &RenderCommand::InsertBefore {
                parent: win,
                child: lbl,
                ref_child: Some(anchor),
            },
            "AnchorComment",
            |_| "NativeLabel",
        );

        assert_eq!(state.node_count(), 3);
        assert_eq!(state.style_count(), 1);
        assert_eq!(state.get_node(win).unwrap().children, vec![lbl, anchor]);
        assert_eq!(state.get_node(lbl).unwrap().style_classes, vec![style]);
    }
}
