pub mod js_engine;
pub mod jsc_sys;
pub mod parser;
pub mod quickjs_engine;

use crate::engine::{EngineError, EventPayload, NodeId, RenderEngine};
use crate::platforms::PlatformBackend;
pub use js_engine::{AnyJsEngine, JsEngineBackend, JscEngine, QuickJsEngine};
pub use parser::{Instruction, parse_instruction, parse_script};
use std::collections::{HashMap, HashSet};

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
/// hosts a pluggable `JsEngineBackend` (`JavaScriptCore` JIT or `QuickJS`),
/// maintains reactive Signal state, supports state-preserving Hot Reload (HMR),
/// and dispatches native OS events back into Angular handlers.
#[derive(Default)]
pub struct JsRuntime {
    js_engine: AnyJsEngine,
    variables: HashMap<String, NodeId>,
    signals: HashMap<String, i64>,
    bindings: Vec<TextBinding>,
    handlers: HashMap<u64, HandlerAction>,
    next_listener_id: u64,
}

impl JsRuntime {
    pub fn new() -> Self {
        Self::with_engine(AnyJsEngine::default())
    }

    pub fn with_engine(js_engine: AnyJsEngine) -> Self {
        Self {
            js_engine,
            variables: HashMap::new(),
            signals: HashMap::new(),
            bindings: Vec::new(),
            handlers: HashMap::new(),
            next_listener_id: 1,
        }
    }

    pub fn from_engine_config(engine_kind: &str) -> Result<Self, EngineError> {
        let js_engine = AnyJsEngine::from_config(engine_kind)?;
        Ok(Self::with_engine(js_engine))
    }

    pub fn js_engine(&self) -> &AnyJsEngine {
        &self.js_engine
    }

    /// Evaluates JavaScript code inside the active `JsEngineBackend` (`JscEngine` or `QuickJsEngine`)
    /// and flushes any `globalThis.__NG_NATIVE__` scene mutations to `RenderEngine`.
    pub fn eval_js<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        js_code: &str,
    ) -> Result<String, EngineError> {
        let eval_result = self.js_engine.eval_script(js_code)?;
        let drained = self.js_engine.drain_native_commands()?;
        if !drained.trim().is_empty() {
            self.eval_tick(engine, &drained)?;
        }
        Ok(eval_result)
    }

    pub fn eval_tick<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        script: &str,
    ) -> Result<(), EngineError> {
        let instructions = parse_script(script)?;
        let mut newly_created = HashSet::new();
        for inst in instructions {
            self.execute_instruction(engine, inst, false, &mut newly_created)?;
        }

        engine.flush_frame();
        Ok(())
    }

    /// Hot-reloads an updated `.ngb` bundle in-place against the live scene graph,
    /// preserving active Signal values while updating styles, text, and new widgets.
    pub fn hot_reload<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        script: &str,
    ) -> Result<(), EngineError> {
        let instructions = parse_script(script)?;
        self.bindings.clear();
        self.handlers.clear();
        engine.clear_listeners();

        let mut newly_created = HashSet::new();
        for inst in instructions {
            self.execute_instruction(engine, inst, true, &mut newly_created)?;
        }

        let active_signals: Vec<(String, i64)> =
            self.signals.iter().map(|(k, v)| (k.clone(), *v)).collect();
        for (sig_name, val) in active_signals {
            self.propagate_signal(engine, &sig_name, val)?;
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
        self.trigger_event_with_payload(engine, var_name, event_name, &EventPayload::None)
    }

    /// Dispatches a native OS event carrying a typed `EventPayload` (`Text`, `Bool`, `Int`).
    pub fn trigger_event_with_payload<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        var_name: &str,
        event_name: &str,
        payload: &EventPayload,
    ) -> Result<(), EngineError> {
        let node_id = self.resolve_var(var_name)?;
        self.js_engine
            .dispatch_js_event_with_payload(var_name, event_name, payload)?;
        let drained = self.js_engine.drain_native_commands()?;
        if !drained.trim().is_empty() {
            self.eval_tick(engine, &drained)?;
        }
        self.trigger_node_event(engine, node_id, event_name)
    }

    /// Dispatches a native OS event by `NodeId` with a typed `EventPayload`, notifying both
    /// the embedded JS engine (`__dispatchEvent`) and declarative scene handlers.
    pub fn trigger_node_event_with_payload<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        node_id: NodeId,
        event_name: &str,
        payload: &EventPayload,
    ) -> Result<(), EngineError> {
        let var_name: Option<String> = self
            .variables
            .iter()
            .find_map(|(k, id)| (*id == node_id).then(|| k.clone()));
        if let Some(var) = var_name {
            self.js_engine
                .dispatch_js_event_with_payload(&var, event_name, payload)?;
            let drained = self.js_engine.drain_native_commands()?;
            if !drained.trim().is_empty() {
                self.eval_tick(engine, &drained)?;
            }
        }
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

    fn execute_instruction<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        instruction: Instruction,
        is_hmr: bool,
        newly_created: &mut HashSet<String>,
    ) -> Result<(), EngineError> {
        match instruction {
            Instruction::RegisterStyle(rule) => {
                engine.register_style(rule);
                Ok(())
            }
            Instruction::DeclareSignal { name, initial } => {
                if is_hmr {
                    self.signals.entry(name).or_insert(initial);
                } else {
                    self.signals.insert(name, initial);
                }
                Ok(())
            }
            Instruction::BindText {
                var_name,
                signal_name,
                template,
            } => {
                let _ = self.resolve_var(&var_name)?;
                self.bindings.push(TextBinding {
                    node_var: var_name,
                    signal_name,
                    template,
                });
                Ok(())
            }
            Instruction::OnAdd {
                var_name,
                event_name,
                signal_name,
                delta,
            } => self.attach_handler(
                engine,
                &var_name,
                &event_name,
                HandlerAction::AddSignal { signal_name, delta },
            ),
            Instruction::OnSet {
                var_name,
                event_name,
                signal_name,
                value,
            } => self.attach_handler(
                engine,
                &var_name,
                &event_name,
                HandlerAction::SetSignal { signal_name, value },
            ),
            Instruction::OnText {
                var_name,
                event_name,
                target_var,
                text,
            } => self.attach_handler(
                engine,
                &var_name,
                &event_name,
                HandlerAction::SetText {
                    node_var: target_var,
                    text,
                },
            ),
            other => self.execute_scene_instruction(engine, other, is_hmr, newly_created),
        }
    }

    fn execute_scene_instruction<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        instruction: Instruction,
        is_hmr: bool,
        newly_created: &mut HashSet<String>,
    ) -> Result<(), EngineError> {
        match instruction {
            Instruction::CreateElement { var_name, tag } => {
                if !is_hmr || !self.variables.contains_key(&var_name) {
                    let id = engine.create_element(&tag)?;
                    self.variables.insert(var_name.clone(), id);
                    newly_created.insert(var_name);
                }
                Ok(())
            }
            Instruction::CreateComment { var_name, text } => {
                if !is_hmr || !self.variables.contains_key(&var_name) {
                    let id = engine.create_comment(text);
                    self.variables.insert(var_name.clone(), id);
                    newly_created.insert(var_name);
                }
                Ok(())
            }
            Instruction::SetText { var_name, text } => {
                let id = self.resolve_var(&var_name)?;
                engine.set_text(id, text)
            }
            Instruction::SetAttribute {
                var_name,
                key,
                value,
            } => {
                let id = self.resolve_var(&var_name)?;
                engine.set_attribute(id, key, value)
            }
            Instruction::SetIcon {
                var_name,
                app_id,
                ico_path,
            } => {
                let id = self.resolve_var(&var_name)?;
                engine.set_window_icon(id, app_id, &ico_path)
            }
            Instruction::AppendChild {
                parent_var,
                child_var,
            } => {
                if !is_hmr || newly_created.contains(&child_var) {
                    let parent = self.resolve_var(&parent_var)?;
                    let child = self.resolve_var(&child_var)?;
                    engine.append_child(parent, child)?;
                }
                Ok(())
            }
            Instruction::InsertBefore {
                parent_var,
                child_var,
                ref_var,
            } => {
                if !is_hmr || newly_created.contains(&child_var) {
                    let parent = self.resolve_var(&parent_var)?;
                    let child = self.resolve_var(&child_var)?;
                    let ref_child = match ref_var {
                        Some(r) => Some(self.resolve_var(&r)?),
                        None => None,
                    };
                    engine.insert_before(parent, child, ref_child)?;
                }
                Ok(())
            }
            Instruction::RemoveChild {
                parent_var,
                child_var,
            } => {
                let parent = self.resolve_var(&parent_var)?;
                let child = self.resolve_var(&child_var)?;
                engine.remove_child(parent, child)
            }
            Instruction::AddClass {
                var_name,
                class_name,
            } => {
                let id = self.resolve_var(&var_name)?;
                engine.add_class(id, &class_name)
            }
            Instruction::Destroy { var_name } => {
                let id = self.resolve_var(&var_name)?;
                self.variables.remove(&var_name);
                engine.destroy_node(id)
            }
            _ => Ok(()),
        }
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
