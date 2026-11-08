pub mod arena;
pub mod commands;
pub mod events;
pub mod node;
pub mod styles;
pub mod widget;

use crate::platforms::PlatformBackend;
use arena::Arena;
pub use commands::{CommandQueue, RenderCommand};
use events::EventRegistry;
pub use node::{EngineError, NodeId, StyleId};
use std::fmt::Write;
pub use styles::{StyleRegistry, StyleRule};
pub use widget::WidgetKind;

/// Generic Render Engine (OS-agnostic).
/// Composes the scene graph (`Arena`), asset manager (`StyleRegistry`),
/// event bus (`EventRegistry`), and frame command buffer (`CommandQueue`).
pub struct RenderEngine<B: PlatformBackend> {
    arena: Arena,
    styles: StyleRegistry,
    events: EventRegistry,
    commands: CommandQueue,
    backend: B,
}

impl<B: PlatformBackend> RenderEngine<B> {
    pub fn new(backend: B) -> Self {
        Self {
            arena: Arena::new(),
            styles: StyleRegistry::new(),
            events: EventRegistry::new(),
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

    pub fn set_text(&mut self, id: NodeId, text: impl Into<String>) -> Result<(), EngineError> {
        let text_str = text.into();
        self.arena
            .with_exclusive_mut(id, |w| w.set_text(id, text_str.clone()))?;
        self.commands
            .push(RenderCommand::SetText { id, text: text_str });
        Ok(())
    }

    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), EngineError> {
        let _ = self.arena.get(child)?;
        self.arena
            .with_exclusive_mut(parent, |w| w.add_child(parent, child))?;
        self.commands
            .push(RenderCommand::AppendChild { parent, child });
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

    pub fn print_tree(&self, id: NodeId, indent: usize) -> Result<(), EngineError> {
        let pad = "  ".repeat(indent);
        let classes = self.styles.node_classes(id);
        let class_str = if classes.is_empty() {
            String::new()
        } else {
            format!(" classes={classes:?}")
        };

        let widget = self.arena.get(id)?;
        match widget {
            WidgetKind::Window { title, child } => {
                println!(
                    "{pad}[Window #{}:{} title=\"{title}\"{class_str}]",
                    id.index, id.generation
                );
                if let Some(child_id) = child {
                    self.print_tree(*child_id, indent + 1)?;
                }
            }
            WidgetKind::Box {
                vertical,
                spacing,
                children,
            } => {
                let dir = if *vertical { "vertical" } else { "horizontal" };
                println!(
                    "{pad}[Box #{}:{} dir={dir} spacing={spacing}{class_str}]",
                    id.index, id.generation
                );
                for child_id in children {
                    if self.arena.get(*child_id).is_ok() {
                        self.print_tree(*child_id, indent + 1)?;
                    }
                }
            }
            WidgetKind::Button { label } => {
                println!(
                    "{pad}[Button #{}:{} label=\"{label}\"{class_str}]",
                    id.index, id.generation
                );
            }
            WidgetKind::Label { text } => {
                println!(
                    "{pad}[Label #{}:{} text=\"{text}\"{class_str}]",
                    id.index, id.generation
                );
            }
        }
        Ok(())
    }

    pub fn render_window_preview(&self, root_id: NodeId) -> Result<String, EngineError> {
        let root = self.arena.get(root_id)?;
        let (title, child_opt) = match root {
            WidgetKind::Window { title, child } => (title.as_str(), *child),
            _ => return Err(EngineError::InvalidParent(root_id)),
        };

        let width: usize = 76;
        let border = "═".repeat(width);
        let mut out = String::new();
        let _ = writeln!(out, "╔{border}╗");
        let header = format!(" ● ● ●  {title} ({})", self.backend.backend_name());
        let _ = writeln!(out, "║{header:<width$}║");
        let _ = writeln!(out, "╠{border}╣");

        if let Some(child_id) = child_opt {
            self.format_preview_node(child_id, width, &mut out)?;
        }

        let _ = write!(out, "╚{border}╝");
        Ok(out)
    }

    fn format_preview_node(
        &self,
        id: NodeId,
        width: usize,
        out: &mut String,
    ) -> Result<(), EngineError> {
        let widget = self.arena.get(id)?;
        let style_summary = self.describe_node_styles(id);

        match widget {
            WidgetKind::Box { children, .. } => {
                if !style_summary.is_empty() {
                    let line = format!("  [Box {style_summary}]");
                    let _ = writeln!(out, "║{line:<width$}║");
                }
                for child_id in children {
                    if self.arena.get(*child_id).is_ok() {
                        self.format_preview_node(*child_id, width, out)?;
                    }
                }
            }
            WidgetKind::Label { text } => {
                let line = if style_summary.is_empty() {
                    format!("  {text}")
                } else {
                    format!("  {text}  {style_summary}")
                };
                let _ = writeln!(out, "║{line:<width$}║");
            }
            WidgetKind::Button { label } => {
                let line = if style_summary.is_empty() {
                    format!("  [ {label} ]")
                } else {
                    format!("  [ {label} ]  {style_summary}")
                };
                let _ = writeln!(out, "║{line:<width$}║");
            }
            WidgetKind::Window { .. } => {}
        }
        Ok(())
    }

    fn describe_node_styles(&self, id: NodeId) -> String {
        let classes = self.styles.node_classes(id);
        if classes.is_empty() {
            return String::new();
        }
        let mut details = Vec::new();
        for class_name in classes {
            if let Some(rule) = self.styles.get_rule_by_name(class_name) {
                let bg = rule.background.as_deref().unwrap_or("transparent");
                let fg = rule.foreground.as_deref().unwrap_or("inherit");
                details.push(format!(
                    ".{class_name}(bg:{bg}, fg:{fg}, pad:{}px)",
                    rule.padding
                ));
            } else {
                details.push(format!(".{class_name}"));
            }
        }
        format!("{{{}}}", details.join(" "))
    }
}
