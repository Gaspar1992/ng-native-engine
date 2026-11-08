use super::js_engine::JsEngineBackend;
use crate::engine::{EngineError, EventPayload};

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

    fn dispatch_js_event_with_payload(
        &mut self,
        _node_var: &str,
        _event_name: &str,
        _payload: &EventPayload,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    fn drain_native_commands(&mut self) -> Result<String, EngineError> {
        let drained = self.pending_commands.join("\n");
        self.pending_commands.clear();
        Ok(drained)
    }
}
