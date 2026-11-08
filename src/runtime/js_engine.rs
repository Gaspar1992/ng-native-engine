pub use super::bootstrap_js::NG_NATIVE_BOOTSTRAP_JS;
use super::jsc_sys::JscSys;
pub use super::quickjs_engine::QuickJsEngine;
use crate::engine::{EngineError, EventPayload};

/// Hexagonal port for embedded JavaScript engines (`JavaScriptCore`, `QuickJS-NG`, etc.).
/// Decouples `RenderEngine` and `JsRuntime` from any specific JS VM implementation.
pub trait JsEngineBackend {
    fn engine_name(&self) -> &str;
    fn is_jit_enabled(&self) -> bool;
    fn eval_script(&mut self, js_code: &str) -> Result<String, EngineError>;
    fn dispatch_js_event(&mut self, node_var: &str, event_name: &str) -> Result<(), EngineError> {
        self.dispatch_js_event_with_payload(node_var, event_name, &EventPayload::None)
    }
    fn dispatch_js_event_with_payload(
        &mut self,
        node_var: &str,
        event_name: &str,
        payload: &EventPayload,
    ) -> Result<(), EngineError>;
    fn drain_native_commands(&mut self) -> Result<String, EngineError>;
}

/// High-performance Apple/WebKit `JavaScriptCore` (JSC) adapter with 4-tier JIT (`libjavascriptcoregtk-6.0.so.1`).
pub struct JscEngine {
    jsc: Option<JscSys>,
    fallback: QuickJsEngine,
}

impl Default for JscEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl JscEngine {
    pub fn new() -> Self {
        let jsc = JscSys::try_load();
        if let Some(sys) = &jsc {
            let _ = sys.eval(NG_NATIVE_BOOTSTRAP_JS);
        }
        Self {
            jsc,
            fallback: QuickJsEngine::new(),
        }
    }

    pub fn is_native_jsc_loaded(&self) -> bool {
        self.jsc.is_some()
    }
}

impl JsEngineBackend for JscEngine {
    fn engine_name(&self) -> &str {
        if self.jsc.is_some() {
            "JavaScriptCore JIT (libjavascriptcoregtk-6.0.so.1)"
        } else {
            "JavaScriptCore (Portable Fallback)"
        }
    }

    fn is_jit_enabled(&self) -> bool {
        self.jsc.is_some()
    }

    fn eval_script(&mut self, js_code: &str) -> Result<String, EngineError> {
        if let Some(jsc) = &self.jsc {
            jsc.eval(js_code).map_err(EngineError::ScriptSyntaxError)
        } else {
            self.fallback.eval_script(js_code)
        }
    }

    fn dispatch_js_event_with_payload(
        &mut self,
        node_var: &str,
        event_name: &str,
        payload: &EventPayload,
    ) -> Result<(), EngineError> {
        if let Some(jsc) = &self.jsc {
            let payload_js = match payload {
                EventPayload::None => String::from("{}"),
                EventPayload::Text(t) => format!("{{ value: {t:?} }}"),
                EventPayload::Bool(b) => format!("{{ value: {b}, active: {b}, checked: {b} }}"),
                EventPayload::Int(n) => format!("{{ value: {n} }}"),
            };
            let call = format!(
                "globalThis.__NG_NATIVE__.__dispatchEvent({node_var:?}, {event_name:?}, {payload_js}); globalThis.__NG_NATIVE__.__flushTimers();"
            );
            let _ = jsc.eval(&call).map_err(EngineError::ScriptSyntaxError)?;
            Ok(())
        } else {
            self.fallback
                .dispatch_js_event_with_payload(node_var, event_name, payload)
        }
    }

    fn drain_native_commands(&mut self) -> Result<String, EngineError> {
        if let Some(jsc) = &self.jsc {
            jsc.eval("globalThis.__NG_NATIVE__.__drainQueue()")
                .map_err(EngineError::ScriptSyntaxError)
        } else {
            self.fallback.drain_native_commands()
        }
    }
}

/// Polymorphic JS engine wrapper resolved via Factory Method from `angular.json` `jsEngine`.
pub enum AnyJsEngine {
    Jsc(JscEngine),
    QuickJs(QuickJsEngine),
}

impl Default for AnyJsEngine {
    fn default() -> Self {
        Self::Jsc(JscEngine::new())
    }
}

impl AnyJsEngine {
    pub fn from_config(engine_kind: &str) -> Result<Self, EngineError> {
        match engine_kind {
            "jsc" | "javascriptcore" => Ok(Self::Jsc(JscEngine::new())),
            "quickjs" | "rquickjs" => Ok(Self::QuickJs(QuickJsEngine::new())),
            other => Err(EngineError::ConfigError(format!(
                "Unsupported jsEngine '{other}'. Expected 'jsc' or 'quickjs'."
            ))),
        }
    }
}

impl JsEngineBackend for AnyJsEngine {
    fn engine_name(&self) -> &str {
        match self {
            Self::Jsc(e) => e.engine_name(),
            Self::QuickJs(e) => e.engine_name(),
        }
    }

    fn is_jit_enabled(&self) -> bool {
        match self {
            Self::Jsc(e) => e.is_jit_enabled(),
            Self::QuickJs(e) => e.is_jit_enabled(),
        }
    }

    fn eval_script(&mut self, js_code: &str) -> Result<String, EngineError> {
        match self {
            Self::Jsc(e) => e.eval_script(js_code),
            Self::QuickJs(e) => e.eval_script(js_code),
        }
    }

    fn dispatch_js_event_with_payload(
        &mut self,
        node_var: &str,
        event_name: &str,
        payload: &EventPayload,
    ) -> Result<(), EngineError> {
        match self {
            Self::Jsc(e) => e.dispatch_js_event_with_payload(node_var, event_name, payload),
            Self::QuickJs(e) => e.dispatch_js_event_with_payload(node_var, event_name, payload),
        }
    }

    fn drain_native_commands(&mut self) -> Result<String, EngineError> {
        match self {
            Self::Jsc(e) => e.drain_native_commands(),
            Self::QuickJs(e) => e.drain_native_commands(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_jsc_and_quickjs_engines_queue_and_drain_ng_native_bridge_calls() {
        let bridge_script = r#"
            __NG_NATIVE__.createElement("win", "ng-window");
            __NG_NATIVE__.setText("win", "Dual Engine Window");
            __NG_NATIVE__.createElement("btn", "ng-button");
            __NG_NATIVE__.setAttribute("btn", "halign", "center");
            __NG_NATIVE__.appendChild("win", "btn");
        "#;

        for kind in ["jsc", "quickjs"] {
            let mut engine = AnyJsEngine::from_config(kind).unwrap();
            engine.eval_script(bridge_script).unwrap();
            let batch = engine.drain_native_commands().unwrap();
            assert!(batch.contains("create win ng-window"));
            assert!(batch.contains("text win Dual Engine Window"));
            assert!(batch.contains("attr btn halign center"));
            assert!(batch.contains("append win btn"));
        }
    }

    #[test]
    fn jsc_engine_executes_es2024_and_live_event_closures() {
        let mut jsc = JscEngine::new();
        if jsc.is_native_jsc_loaded() {
            assert!(jsc.is_jit_enabled());
            let script = r#"
                let counter = 10;
                __NG_NATIVE__.createElement("lbl", "ng-label");
                __NG_NATIVE__.createElement("btn", "ng-button");
                __NG_NATIVE__.listen("btn", "clicked", () => {
                    counter += 5;
                    __NG_NATIVE__.setText("lbl", `Counter: ${counter}`);
                });
            "#;
            jsc.eval_script(script).unwrap();
            let _ = jsc.drain_native_commands().unwrap();

            jsc.dispatch_js_event("btn", "clicked").unwrap();
            let after_click = jsc.drain_native_commands().unwrap();
            assert_eq!(after_click.trim(), "text lbl Counter: 15");
        }
    }
}
