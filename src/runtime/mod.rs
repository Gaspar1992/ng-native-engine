use crate::engine::{EngineError, NodeId, RenderEngine, StyleRule};
use crate::platforms::PlatformBackend;
use std::collections::HashMap;

/// Embedded JS Bridge Runtime that executes compiled Angular Renderer2 instructions
/// against the generic `RenderEngine<B>` and flushes frames at the end of each tick.
#[derive(Default)]
pub struct JsRuntime {
    variables: HashMap<String, NodeId>,
}

impl JsRuntime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
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

    fn execute_statement<B: PlatformBackend>(
        &mut self,
        engine: &mut RenderEngine<B>,
        line: &str,
    ) -> Result<(), EngineError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["style", class_name, bg, fg, padding_str, spacing_str] => {
                let padding = padding_str
                    .parse::<u16>()
                    .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
                let spacing = spacing_str
                    .parse::<u16>()
                    .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))?;
                let background = if *bg == "none" {
                    None
                } else {
                    Some((*bg).to_string())
                };
                let foreground = if *fg == "none" {
                    None
                } else {
                    Some((*fg).to_string())
                };
                engine.register_style(StyleRule {
                    class_name: (*class_name).to_string(),
                    background,
                    foreground,
                    padding,
                    spacing,
                });
                Ok(())
            }
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

    pub fn resolve_var(&self, name: &str) -> Result<NodeId, EngineError> {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| EngineError::ScriptSyntaxError(format!("Unknown JS handle: {name}")))
    }
}
