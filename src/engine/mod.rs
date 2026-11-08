pub mod arena;
pub mod commands;
pub mod events;
pub mod icons;
pub mod node;
pub mod preview;
pub mod styles;
pub mod widget;
pub mod widget_attrs;

use crate::platforms::PlatformBackend;
use arena::Arena;
pub use commands::{CommandQueue, RenderCommand};
pub use events::{EventPayload, EventRegistry, NativeEvent};
pub use icons::{IconResolution, WindowIconAsset};
pub use node::{EngineError, NodeId, StyleId};
use std::collections::HashMap;
pub use styles::{StyleRegistry, StyleRule};
pub use widget::WidgetKind;

/// Generic Render Engine (OS-agnostic).
/// Composes the scene graph (`Arena`), asset manager (`StyleRegistry`),
/// event bus (`EventRegistry`), attribute map, and frame command buffer (`CommandQueue`).
pub struct RenderEngine<B: PlatformBackend> {
    pub(crate) arena: Arena,
    pub(crate) styles: StyleRegistry,
    events: EventRegistry,
    attributes: HashMap<NodeId, HashMap<String, String>>,
    commands: CommandQueue,
    pub(crate) backend: B,
}

impl<B: PlatformBackend> RenderEngine<B> {
    pub fn new(backend: B) -> Self {
        Self {
            arena: Arena::new(),
            styles: StyleRegistry::new(),
            events: EventRegistry::new(),
            attributes: HashMap::new(),
            commands: CommandQueue::new(),
            backend,
        }
    }

    pub fn register_style(&mut self, rule: StyleRule) -> StyleId {
        let style_id = self.styles.register_rule(rule.clone());
        self.commands
            .push(RenderCommand::RegisterStyle { style_id, rule });
        style_id
    }

    pub fn create_element(&mut self, tag: &str) -> Result<NodeId, EngineError> {
        let widget = WidgetKind::from_tag(tag)?;
        let id = self.arena.insert(widget);
        self.commands.push(RenderCommand::CreateWidget {
            id,
            tag: tag.to_string(),
        });
        Ok(id)
    }

    pub fn create_comment(&mut self, text: impl Into<String>) -> NodeId {
        let comment_str = text.into();
        let id = self
            .arena
            .insert(WidgetKind::new_comment(comment_str.clone()));
        self.commands.push(RenderCommand::CreateComment {
            id,
            text: comment_str,
        });
        id
    }

    pub fn set_text(&mut self, id: NodeId, text: impl Into<String>) -> Result<(), EngineError> {
        let text_str = text.into();
        self.arena
            .with_exclusive_mut(id, |w| w.set_text(id, text_str.clone()))?;
        self.commands
            .push(RenderCommand::SetText { id, text: text_str });
        Ok(())
    }

    pub fn set_attribute(
        &mut self,
        node: NodeId,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), EngineError> {
        let key_str = key.into();
        let val_str = value.into();
        self.arena.with_exclusive_mut(node, |w| {
            w.set_attribute(&key_str, &val_str);
            Ok(())
        })?;
        self.attributes
            .entry(node)
            .or_default()
            .insert(key_str.clone(), val_str.clone());
        self.commands.push(RenderCommand::SetAttribute {
            node,
            key: key_str,
            value: val_str,
        });
        Ok(())
    }

    pub fn get_attribute(&self, node: NodeId, key: &str) -> Option<&str> {
        self.attributes
            .get(&node)
            .and_then(|map| map.get(key))
            .map(String::as_str)
    }

    pub fn set_window_icon(
        &mut self,
        node: NodeId,
        app_id: impl Into<String>,
        ico_path: &str,
    ) -> Result<(), EngineError> {
        let icon = WindowIconAsset::from_ico_file(app_id, ico_path)?;
        self.arena
            .with_exclusive_mut(node, |w| w.set_icon(node, icon.clone()))?;
        self.commands
            .push(RenderCommand::SetWindowIcon { node, icon });
        Ok(())
    }

    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), EngineError> {
        let is_header_bar = self.arena.get(child)?.is_header_bar();
        self.arena
            .with_exclusive_mut(parent, |w| w.add_child(parent, child, is_header_bar))?;
        self.commands
            .push(RenderCommand::AppendChild { parent, child });
        Ok(())
    }

    pub fn insert_before(
        &mut self,
        parent: NodeId,
        new_child: NodeId,
        ref_child: Option<NodeId>,
    ) -> Result<(), EngineError> {
        let _ = self.arena.get(new_child)?;
        if let Some(ref_id) = ref_child {
            let _ = self.arena.get(ref_id)?;
        }
        self.arena
            .with_exclusive_mut(parent, |w| w.insert_before(parent, new_child, ref_child))?;
        self.commands.push(RenderCommand::InsertBefore {
            parent,
            child: new_child,
            ref_child,
        });
        Ok(())
    }

    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), EngineError> {
        let _ = self.arena.get(child)?;
        self.arena
            .with_exclusive_mut(parent, |w| w.remove_child(parent, child))?;
        self.commands
            .push(RenderCommand::RemoveChild { parent, child });
        Ok(())
    }

    pub fn add_class(&mut self, node: NodeId, class_name: &str) -> Result<(), EngineError> {
        let _ = self.arena.get(node)?;
        let style_id = self.styles.attach_class(node, class_name)?;
        self.commands
            .push(RenderCommand::ApplyStyleClass { node, style_id });
        Ok(())
    }

    pub fn listen(
        &mut self,
        node: NodeId,
        event_name: impl Into<String>,
        listener_id: u64,
    ) -> Result<(), EngineError> {
        let _ = self.arena.get(node)?;
        self.events.register(node, event_name, listener_id);
        Ok(())
    }

    pub fn unlisten(&mut self, listener_id: u64) -> bool {
        self.events.unregister(listener_id)
    }

    pub fn clear_listeners(&mut self) {
        self.events.clear();
    }

    pub fn emit_event<F>(
        &mut self,
        source: NodeId,
        event_name: &str,
        mut js_dispatcher: F,
    ) -> Result<(), EngineError>
    where
        F: FnMut(&mut Self, u64),
    {
        self.arena.borrow_shared(source)?;
        let matching_ids = self.events.matching_listeners(source, event_name);

        for lid in matching_ids {
            js_dispatcher(self, lid);
        }

        self.arena.release_shared(source)?;
        self.flush_frame();
        Ok(())
    }

    pub fn destroy_node(&mut self, id: NodeId) -> Result<(), EngineError> {
        self.arena.remove(id)?;
        self.events.remove_for_node(id);
        self.styles.remove_node(id);
        self.attributes.remove(&id);
        self.commands.push(RenderCommand::DestroyWidget { id });
        Ok(())
    }

    pub fn flush_frame(&mut self) {
        let batch = self.commands.drain();
        self.backend.flush_commands(&batch);
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }
}
