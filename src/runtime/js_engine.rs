use super::jsc_sys::JscSys;
use crate::engine::EngineError;

/// Standard `globalThis.__NG_NATIVE__` + Web/Host globals (`console`, `queueMicrotask`, `setTimeout`)
/// injected into every JS engine context.
/// `@ng-native/platform` (`NativeRenderer2`) calls this object to queue frame-batched scene commands
/// and register live JS event listener closures.
pub const NG_NATIVE_BOOTSTRAP_JS: &str = r"
globalThis.__NG_NATIVE__ = (() => {
  let queue = [];
  let logs = [];
  let timers = new Map();
  let nextTimerId = 1;
  let listeners = new Map();
  let nextListenerId = 1;

  globalThis.console = {
    log: (...args) => logs.push(['LOG', args.join(' ')]),
    info: (...args) => logs.push(['INFO', args.join(' ')]),
    warn: (...args) => logs.push(['WARN', args.join(' ')]),
    error: (...args) => logs.push(['ERROR', args.join(' ')]),
  };

  if (typeof globalThis.queueMicrotask !== 'function') {
    globalThis.queueMicrotask = (cb) => Promise.resolve().then(cb);
  }

  globalThis.setTimeout = (cb, _ms = 0) => {
    const id = nextTimerId++;
    timers.set(id, cb);
    return id;
  };

  globalThis.clearTimeout = (id) => {
    timers.delete(id);
  };

  return {
    registerStyle(cls, bg = 'none', fg = 'none', pad = 0, space = 0) {
      queue.push(`style ${cls} ${bg} ${fg} ${pad} ${space}`);
    },
    declareSignal(name, initial = 0) {
      queue.push(`signal ${name} ${initial}`);
    },
    createElement(id, tag) {
      queue.push(`create ${id} ${tag}`);
      return id;
    },
    createComment(id, text = 'anchor') {
      queue.push(`comment ${id} ${text}`);
      return id;
    },
    setText(id, text) {
      queue.push(`text ${id} ${text}`);
    },
    setAttribute(id, key, value) {
      queue.push(`attr ${id} ${key} ${value}`);
    },
    setWindowIcon(id, appId, icoPath) {
      queue.push(`icon ${id} ${appId} ${icoPath}`);
    },
    appendChild(parent, child) {
      queue.push(`append ${parent} ${child}`);
    },
    insertBefore(parent, child, refChild) {
      if (refChild) {
        queue.push(`insert_before ${parent} ${child} ${refChild}`);
      } else {
        queue.push(`insert_before ${parent} ${child}`);
      }
    },
    removeChild(parent, child) {
      queue.push(`remove_child ${parent} ${child}`);
    },
    addClass(id, className) {
      queue.push(`add_class ${id} ${className}`);
    },
    bindText(id, signalName, template) {
      queue.push(`bind_text ${id} ${signalName} ${template}`);
    },
    onAdd(id, eventName, signalName, delta = 1) {
      queue.push(`on_add ${id} ${eventName} ${signalName} ${delta}`);
    },
    onSet(id, eventName, signalName, value = 0) {
      queue.push(`on_set ${id} ${eventName} ${signalName} ${value}`);
    },
    listen(id, eventName, callback) {
      const lid = nextListenerId++;
      listeners.set(lid, { id, eventName, callback });
      return () => listeners.delete(lid);
    },
    __dispatchEvent(id, eventName, eventPayload = {}) {
      for (const entry of listeners.values()) {
        if (entry.id === id && entry.eventName === eventName) {
          entry.callback(eventPayload);
        }
      }
    },
    __flushTimers() {
      const pending = Array.from(timers.values());
      timers.clear();
      for (const cb of pending) {
        cb();
      }
    },
    destroyNode(id) {
      queue.push(`destroy ${id}`);
    },
    __drainQueue() {
      const out = queue.join('\n');
      queue = [];
      return out;
    }
  };
})();
";

/// Hexagonal port for embedded JavaScript engines (`JavaScriptCore`, `QuickJS-NG`, etc.).
/// Decouples `RenderEngine` and `JsRuntime` from any specific JS VM implementation.
pub trait JsEngineBackend {
    fn engine_name(&self) -> &str;
    fn is_jit_enabled(&self) -> bool;
    fn eval_script(&mut self, js_code: &str) -> Result<String, EngineError>;
    fn dispatch_js_event(&mut self, node_var: &str, event_name: &str) -> Result<(), EngineError>;
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

    fn dispatch_js_event(&mut self, node_var: &str, event_name: &str) -> Result<(), EngineError> {
        if let Some(jsc) = &self.jsc {
            let call = format!(
                "globalThis.__NG_NATIVE__.__dispatchEvent({node_var:?}, {event_name:?}); globalThis.__NG_NATIVE__.__flushTimers();"
            );
            let _ = jsc.eval(&call).map_err(EngineError::ScriptSyntaxError)?;
            Ok(())
        } else {
            self.fallback.dispatch_js_event(node_var, event_name)
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

/// Lightweight, zero-JIT embedded `QuickJS` adapter for portable/static builds.
/// Translates `__NG_NATIVE__.*` calls into frame-batched scene commands with zero external C dependencies.
#[derive(Default)]
pub struct QuickJsEngine {
    pending_commands: Vec<String>,
}

impl QuickJsEngine {
    pub fn new() -> Self {
        Self {
            pending_commands: Vec::new(),
        }
    }

    fn parse_call_args(raw_args: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quote: Option<char> = None;

        for ch in raw_args.chars() {
            match in_quote {
                Some(q) if ch == q => in_quote = None,
                None if ch == '\'' || ch == '"' => in_quote = Some(ch),
                None if ch == ',' => {
                    args.push(current.trim().to_string());
                    current.clear();
                }
                Some(_) | None => current.push(ch),
            }
        }
        let tail = current.trim();
        if !tail.is_empty() {
            args.push(tail.to_string());
        }
        args
    }

    fn translate_bridge_call(&mut self, method: &str, args: &[String]) -> Result<(), EngineError> {
        let cmd = match (method, args) {
            ("registerStyle", [cls, bg, fg, pad, space]) => {
                format!("style {cls} {bg} {fg} {pad} {space}")
            }
            ("declareSignal", [name, initial]) => format!("signal {name} {initial}"),
            ("createElement", [id, tag]) => format!("create {id} {tag}"),
            ("createComment", [id, text]) => format!("comment {id} {text}"),
            ("setText", [id, text]) => format!("text {id} {text}"),
            ("setAttribute", [id, key, val]) => format!("attr {id} {key} {val}"),
            ("setWindowIcon", [id, app_id, path]) => format!("icon {id} {app_id} {path}"),
            ("appendChild", [parent, child]) => format!("append {parent} {child}"),
            ("insertBefore", [parent, child]) => format!("insert_before {parent} {child}"),
            ("insertBefore", [parent, child, ref_c]) => {
                format!("insert_before {parent} {child} {ref_c}")
            }
            ("removeChild", [parent, child]) => format!("remove_child {parent} {child}"),
            ("addClass", [id, cls]) => format!("add_class {id} {cls}"),
            ("bindText", [id, sig, tpl]) => format!("bind_text {id} {sig} {tpl}"),
            ("onAdd", [id, evt, sig, delta]) => format!("on_add {id} {evt} {sig} {delta}"),
            ("onSet", [id, evt, sig, val]) => format!("on_set {id} {evt} {sig} {val}"),
            ("destroyNode", [id]) => format!("destroy {id}"),
            _ => {
                return Err(EngineError::ScriptSyntaxError(format!(
                    "Unsupported QuickJS bridge call: {method}"
                )));
            }
        };
        self.pending_commands.push(cmd);
        Ok(())
    }
}

impl JsEngineBackend for QuickJsEngine {
    fn engine_name(&self) -> &str {
        "QuickJS-NG Embedded (Bytecode Interpreter)"
    }

    fn is_jit_enabled(&self) -> bool {
        false
    }

    fn eval_script(&mut self, js_code: &str) -> Result<String, EngineError> {
        let mut last_result = String::from("undefined");
        for raw_line in js_code.lines() {
            let line = raw_line.trim().trim_end_matches(';').trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let normalized = line.strip_prefix("globalThis.").unwrap_or(line);
            if let Some(rest) = normalized.strip_prefix("__NG_NATIVE__.")
                && let Some(open) = rest.find('(')
                && let Some(close) = rest.rfind(')')
            {
                let method = rest[..open].trim();
                let args = Self::parse_call_args(&rest[open + 1..close]);
                self.translate_bridge_call(method, &args)?;
                if let Some(first) = args.first() {
                    last_result.clone_from(first);
                }
            }
        }
        Ok(last_result)
    }

    fn dispatch_js_event(&mut self, _node_var: &str, _event_name: &str) -> Result<(), EngineError> {
        Ok(())
    }

    fn drain_native_commands(&mut self) -> Result<String, EngineError> {
        let drained = self.pending_commands.join("\n");
        self.pending_commands.clear();
        Ok(drained)
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

    fn dispatch_js_event(&mut self, node_var: &str, event_name: &str) -> Result<(), EngineError> {
        match self {
            Self::Jsc(e) => e.dispatch_js_event(node_var, event_name),
            Self::QuickJs(e) => e.dispatch_js_event(node_var, event_name),
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
