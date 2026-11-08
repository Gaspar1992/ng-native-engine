use super::{Instruction, JsRuntime};
use crate::engine::{EngineError, NodeId, RenderEngine};
use crate::platforms::PlatformBackend;
use std::collections::HashSet;

/// Scene Executor (SRP): applies structural `.ngb` instructions to the `RenderEngine`,
/// keeping the `var <-> NodeId` handle tables in sync and honoring HMR reuse rules.
impl JsRuntime {
    pub(super) fn execute_scene_instruction<B: PlatformBackend>(
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
                    self.register_handle(&var_name, id);
                    newly_created.insert(var_name);
                }
                Ok(())
            }
            Instruction::CreateComment { var_name, text } => {
                if !is_hmr || !self.variables.contains_key(&var_name) {
                    let id = engine.create_comment(text);
                    self.register_handle(&var_name, id);
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
                    let ref_child = ref_var.map(|r| self.resolve_var(&r)).transpose()?;
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
                self.node_vars.remove(&id);
                engine.destroy_node(id)
            }
            _ => Ok(()),
        }
    }

    fn register_handle(&mut self, var_name: &str, id: NodeId) {
        if let Some(previous) = self.variables.insert(var_name.to_string(), id) {
            self.node_vars.remove(&previous);
        }
        self.node_vars.insert(id, var_name.to_string());
    }
}
