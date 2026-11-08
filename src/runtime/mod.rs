pub mod bootstrap_js;
mod js_bridge;
pub mod js_engine;
pub mod jsc_sys;
pub mod parser;
pub mod quickjs_engine;
mod reactivity;
mod scene_exec;

use crate::desktop::{DesktopServices, DesktopTheme};
use crate::engine::{EngineError, NodeId, RenderEngine};
use crate::platforms::PlatformBackend;
pub use js_engine::{AnyJsEngine, JsEngineBackend, JscEngine, QuickJsEngine};
pub use parser::{Instruction, parse_instruction, parse_script};
use reactivity::{HandlerAction, TextBinding};
use std::collections::{HashMap, HashSet};

/// Embedded JS Bridge Runtime that executes compiled Angular instructions (`.ngb`),
/// hosts a pluggable `JsEngineBackend` (`JavaScriptCore` JIT or `QuickJS`),
/// maintains reactive Signal state, supports state-preserving Hot Reload (HMR),
/// and dispatches native OS events back into Angular handlers.
///
/// Responsibilities are split across submodules:
/// - `js_bridge`: JS evaluation and native event routing.
/// - `reactivity`: Signals, text bindings, and declarative handlers.
/// - `scene_exec`: structural scene graph instructions and handle tables.
pub struct JsRuntime {
    js_engine: AnyJsEngine,
    desktop: DesktopServices,
    variables: HashMap<String, NodeId>,
    node_vars: HashMap<NodeId, String>,
    signals: HashMap<String, i64>,
    bindings: Vec<TextBinding>,
    handlers: HashMap<u64, HandlerAction>,
    next_listener_id: u64,
}

impl Default for JsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl JsRuntime {
    pub fn new() -> Self {
        Self::with_engine(AnyJsEngine::default())
    }

    pub fn with_engine(js_engine: AnyJsEngine) -> Self {
        Self {
            js_engine,
            desktop: DesktopServices::new(),
            variables: HashMap::new(),
            node_vars: HashMap::new(),
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

    pub const fn js_engine(&self) -> &AnyJsEngine {
        &self.js_engine
    }

    pub const fn desktop(&self) -> &DesktopServices {
        &self.desktop
    }

    pub const fn desktop_mut(&mut self) -> &mut DesktopServices {
        &mut self.desktop
    }

    pub fn signal_value(&self, name: &str) -> Option<i64> {
        self.signals.get(name).copied()
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
            Instruction::SetTheme { mode } => {
                self.desktop.set_theme(DesktopTheme::from_str_lossy(&mode));
                Ok(())
            }
            Instruction::SetClipboard { text } => {
                self.desktop.set_clipboard(&text);
                Ok(())
            }
            other => self.execute_scene_instruction(engine, other, is_hmr, newly_created),
        }
    }

    pub fn resolve_var(&self, name: &str) -> Result<NodeId, EngineError> {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| EngineError::ScriptSyntaxError(format!("Unknown JS handle: {name}")))
    }
}
