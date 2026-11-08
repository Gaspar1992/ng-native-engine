use crate::engine::{EngineError, NodeId, RenderEngine, StyleRule};
use crate::platforms::PlatformBackend;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TextBinding {
    node_var: String,
    signal_name: String,
    template: String,
}

#[derive(Debug, Clone)]
enum HandlerAction {
    AddSignal { signal_name: String, delta: i64 },
    SetSignal { signal_name: String, value: i64 },
    SetText { node_var: String, text: String },
}

/// Embedded JS Bridge Runtime that executes compiled Angular instructions (`.ngb`),
/// maintains reactive Signal state, and dispatches native OS events back into Angular handlers.
#[derive(Default)]
pub struct JsRuntime {
    variables: HashMap<String, NodeId>,
    signals: HashMap<String, i64>,
    bindings: Vec<TextBinding>,
    handlers: HashMap<u64, HandlerAction>,
    next_listener_id: u64,
}

impl JsRuntime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            signals: HashMap::new(),
            bindings: Vec::new(),
            handlers: HashMap::new(),
            next_listener_id: 1,
        }
    }

    pub fn eval_tick<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        script: &str,
    ) -> Result<(), EngineError> {
        for raw_line in script.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            self.execute_statement(engine, line)?;
        }

        engine.flush_frame();
        Ok(())
    }

    /// Dispatches a native OS event on a resolved element variable and runs registered Angular handlers.
    pub fn trigger_event<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        var_name: &str,
        event_name: &str,
    ) -> Result<(), EngineError> {
        let node_id = self.resolve_var(var_name)?;
        self.trigger_node_event(engine, node_id, event_name)
    }

    /// Dispatches a native OS event by `NodeId` and runs registered Angular handlers.
    pub fn trigger_node_event<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        node_id: NodeId,
        event_name: &str,
    ) -> Result<(), EngineError> {
        let mut runtime_err: Option<EngineError> = None;

        engine.emit_event(node_id, event_name, |eng, listener_id| {
            if let Some(action) = self.handlers.get(&listener_id).cloned()
                && let Err(e) = self.execute_handler_action(eng, action)
            {
                runtime_err = Some(e);
            }
        })?;

        if let Some(err) = runtime_err {
            return Err(err);
        }
        Ok(())
    }

    fn execute_handler_action<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        action: HandlerAction,
    ) -> Result<(), EngineError> {
        match action {
            HandlerAction::AddSignal { signal_name, delta } => {
                let current = self.signals.entry(signal_name.clone()).or_insert(0);
                *current += delta;
                let updated_val = *current;
                self.propagate_signal(engine, &signal_name, updated_val)
            }
            HandlerAction::SetSignal { signal_name, value } => {
                self.signals.insert(signal_name.clone(), value);
                self.propagate_signal(engine, &signal_name, value)
            }
            HandlerAction::SetText { node_var, text } => {
                let target_id = self.resolve_var(&node_var)?;
                engine.set_text(target_id, text)
            }
        }
    }

    fn propagate_signal<B: PlatformBackend>(
        &self,
        engine: &mut RenderEngine<B>,
        signal_name: &str,
        updated_val: i64,
    ) -> Result<(), EngineError> {
        let matching_bindings: Vec<TextBinding> = self
            .bindings
            .iter()
            .filter(|b| b.signal_name == signal_name)
            .cloned()
            .collect();

        for binding in matching_bindings {
            let target_id = self.resolve_var(&binding.node_var)?;
            let rendered = binding.template.replace("{}", &updated_val.to_string());
            engine.set_text(target_id, rendered)?;
        }
        Ok(())
    }

    fn execute_statement<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        line: &str,
    ) -> Result<(), EngineError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["style", class_name, bg, fg, padding_str, spacing_str] => {
                let rule =
                    Self::parse_style_rule(line, class_name, bg, fg, padding_str, spacing_str)?;
                engine.register_style(rule);
                Ok(())
            }
            ["signal", signal_name, initial_str] => {
                let initial = initial_str
                    .parse::<i64>()
                    .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
                self.signals.insert((*signal_name).to_string(), initial);
                Ok(())
            }
            ["bind_text", var_name, signal_name, rest @ ..] => {
                let _ = self.resolve_var(var_name)?;
                self.bindings.push(TextBinding {
                    node_var: (*var_name).to_string(),
                    signal_name: (*signal_name).to_string(),
                    template: rest.join(" "),
                });
                Ok(())
            }
            ["on_add", var_name, event_name, signal_name, delta_str] => {
                let delta = delta_str
                    .parse::<i64>()
                    .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
                self.attach_handler(
                    engine,
                    var_name,
                    event_name,
                    HandlerAction::AddSignal {
                        signal_name: (*signal_name).to_string(),
                        delta,
                    },
                )
            }
            ["on_set", var_name, event_name, signal_name, value_str] => {
                let value = value_str
                    .parse::<i64>()
                    .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
                self.attach_handler(
                    engine,
                    var_name,
                    event_name,
                    HandlerAction::SetSignal {
                        signal_name: (*signal_name).to_string(),
                        value,
                    },
                )
            }
            ["on_text", var_name, event_name, target_var, rest @ ..] => self.attach_handler(
                engine,
                var_name,
                event_name,
                HandlerAction::SetText {
                    node_var: (*target_var).to_string(),
                    text: rest.join(" "),
                },
            ),
            ["create", var_name, tag] => {
                let id = engine.create_element(tag)?;
                self.variables.insert((*var_name).to_string(), id);
                Ok(())
            }
            ["text", var_name, rest @ ..] => {
                let id = self.resolve_var(var_name)?;
                engine.set_text(id, rest.join(" "))
            }
            ["append", parent_var, child_var] => {
                let parent = self.resolve_var(parent_var)?;
                let child = self.resolve_var(child_var)?;
                engine.append_child(parent, child)
            }
            ["add_class", var_name, class_name] => {
                let id = self.resolve_var(var_name)?;
                engine.add_class(id, class_name)
            }
            ["destroy", var_name] => {
                let id = self.resolve_var(var_name)?;
                engine.destroy_node(id)
            }
            _ => Err(EngineError::ScriptSyntaxError(line.to_string())),
        }
    }

    fn parse_style_rule(
        line: &str,
        class_name: &str,
        bg: &str,
        fg: &str,
        padding_str: &str,
        spacing_str: &str,
    ) -> Result<StyleRule, EngineError> {
        let padding = padding_str
            .parse::<u16>()
            .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
        let spacing = spacing_str
            .parse::<u16>()
            .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
        Ok(StyleRule {
            class_name: class_name.to_string(),
            background: (bg != "none").then(|| bg.to_string()),
            foreground: (fg != "none").then(|| fg.to_string()),
            padding,
            spacing,
        })
    }

    fn attach_handler<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        var_name: &str,
        event_name: &str,
        action: HandlerAction,
    ) -> Result<(), EngineError> {
        let node_id = self.resolve_var(var_name)?;
        let listener_id = self.next_listener_id;
        self.next_listener_id += 1;
        engine.listen(node_id, event_name, listener_id)?;
        self.handlers.insert(listener_id, action);
        Ok(())
    }

    pub fn resolve_var(&self, name: &str) -> Result<NodeId, EngineError> {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| EngineError::ScriptSyntaxError(format!("Unknown JS handle: {name}")))
    }
}
