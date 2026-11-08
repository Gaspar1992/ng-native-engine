use crate::engine::{EngineError, StyleRule};

/// Intermediate Representation (IR) of a single `.ngb` bytecode instruction.
/// Decouples text syntax parsing from reactive execution against `RenderEngine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    RegisterStyle(StyleRule),
    DeclareSignal {
        name: String,
        initial: i64,
    },
    BindText {
        var_name: String,
        signal_name: String,
        template: String,
    },
    OnAdd {
        var_name: String,
        event_name: String,
        signal_name: String,
        delta: i64,
    },
    OnSet {
        var_name: String,
        event_name: String,
        signal_name: String,
        value: i64,
    },
    OnText {
        var_name: String,
        event_name: String,
        target_var: String,
        text: String,
    },
    CreateElement {
        var_name: String,
        tag: String,
    },
    CreateComment {
        var_name: String,
        text: String,
    },
    SetText {
        var_name: String,
        text: String,
    },
    SetAttribute {
        var_name: String,
        key: String,
        value: String,
    },
    SetIcon {
        var_name: String,
        app_id: String,
        ico_path: String,
    },
    AppendChild {
        parent_var: String,
        child_var: String,
    },
    InsertBefore {
        parent_var: String,
        child_var: String,
        ref_var: Option<String>,
    },
    RemoveChild {
        parent_var: String,
        child_var: String,
    },
    AddClass {
        var_name: String,
        class_name: String,
    },
    Destroy {
        var_name: String,
    },
    SetTheme {
        mode: String,
    },
    SetClipboard {
        text: String,
    },
}

pub fn parse_script(script: &str) -> Result<Vec<Instruction>, EngineError> {
    let mut instructions = Vec::new();
    for raw_line in script.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        instructions.push(parse_instruction(line)?);
    }
    Ok(instructions)
}

pub fn parse_instruction(line: &str) -> Result<Instruction, EngineError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.as_slice() {
        ["style", class_name, bg, fg, padding_str, spacing_str] => {
            let rule = parse_style_rule(line, class_name, bg, fg, padding_str, spacing_str)?;
            Ok(Instruction::RegisterStyle(rule))
        }
        ["signal", signal_name, initial_str] => {
            let initial = parse_i64(line, initial_str)?;
            Ok(Instruction::DeclareSignal {
                name: (*signal_name).to_string(),
                initial,
            })
        }
        ["bind_text", var_name, signal_name, rest @ ..] => Ok(Instruction::BindText {
            var_name: (*var_name).to_string(),
            signal_name: (*signal_name).to_string(),
            template: rest.join(" "),
        }),
        ["on_add", var_name, event_name, signal_name, delta_str] => {
            let delta = parse_i64(line, delta_str)?;
            Ok(Instruction::OnAdd {
                var_name: (*var_name).to_string(),
                event_name: (*event_name).to_string(),
                signal_name: (*signal_name).to_string(),
                delta,
            })
        }
        ["on_set", var_name, event_name, signal_name, value_str] => {
            let value = parse_i64(line, value_str)?;
            Ok(Instruction::OnSet {
                var_name: (*var_name).to_string(),
                event_name: (*event_name).to_string(),
                signal_name: (*signal_name).to_string(),
                value,
            })
        }
        ["on_text", var_name, event_name, target_var, rest @ ..] => Ok(Instruction::OnText {
            var_name: (*var_name).to_string(),
            event_name: (*event_name).to_string(),
            target_var: (*target_var).to_string(),
            text: rest.join(" "),
        }),
        ["create", var_name, tag] => Ok(Instruction::CreateElement {
            var_name: (*var_name).to_string(),
            tag: (*tag).to_string(),
        }),
        ["comment", var_name, rest @ ..] => Ok(Instruction::CreateComment {
            var_name: (*var_name).to_string(),
            text: rest.join(" "),
        }),
        ["text", var_name, rest @ ..] => Ok(Instruction::SetText {
            var_name: (*var_name).to_string(),
            text: normalize_text_payload(rest),
        }),
        ["attr", var_name, key, rest @ ..] => Ok(Instruction::SetAttribute {
            var_name: (*var_name).to_string(),
            key: (*key).to_string(),
            value: normalize_text_payload(rest),
        }),
        ["icon", var_name, app_id, ico_path] => Ok(Instruction::SetIcon {
            var_name: (*var_name).to_string(),
            app_id: (*app_id).to_string(),
            ico_path: (*ico_path).to_string(),
        }),
        ["append", parent_var, child_var] => Ok(Instruction::AppendChild {
            parent_var: (*parent_var).to_string(),
            child_var: (*child_var).to_string(),
        }),
        ["insert_before", parent_var, child_var] => Ok(Instruction::InsertBefore {
            parent_var: (*parent_var).to_string(),
            child_var: (*child_var).to_string(),
            ref_var: None,
        }),
        ["insert_before", parent_var, child_var, ref_var] => Ok(Instruction::InsertBefore {
            parent_var: (*parent_var).to_string(),
            child_var: (*child_var).to_string(),
            ref_var: Some((*ref_var).to_string()),
        }),
        ["remove_child", parent_var, child_var] => Ok(Instruction::RemoveChild {
            parent_var: (*parent_var).to_string(),
            child_var: (*child_var).to_string(),
        }),
        ["add_class", var_name, class_name] => Ok(Instruction::AddClass {
            var_name: (*var_name).to_string(),
            class_name: (*class_name).to_string(),
        }),
        ["destroy", var_name] => Ok(Instruction::Destroy {
            var_name: (*var_name).to_string(),
        }),
        ["theme", mode] => Ok(Instruction::SetTheme {
            mode: (*mode).to_string(),
        }),
        ["clipboard", rest @ ..] => Ok(Instruction::SetClipboard {
            text: normalize_text_payload(rest),
        }),
        _ => Err(EngineError::ScriptSyntaxError(line.to_string())),
    }
}

fn normalize_text_payload(parts: &[&str]) -> String {
    let joined = parts.join(" ");
    if joined == "\"\"" || joined == "''" {
        return String::new();
    }
    if (joined.starts_with('"') && joined.ends_with('"') && joined.len() >= 2)
        || (joined.starts_with('\'') && joined.ends_with('\'') && joined.len() >= 2)
    {
        return joined[1..joined.len() - 1].to_string();
    }
    joined
}

fn parse_i64(line: &str, raw: &str) -> Result<i64, EngineError> {
    raw.parse::<i64>()
        .map_err(|_| EngineError::ScriptSyntaxError(line.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_phase2_layout_and_anchor_instructions() {
        let script = r"
            comment anchor_if @if(showExtra)
            create hdr ng-header-bar
            attr hdr subtitle GTK4 + Libadwaita
            insert_before vbox extra anchor_if
            remove_child vbox extra
        ";
        let parsed = parse_script(script).unwrap();
        assert_eq!(parsed.len(), 5);
        assert_eq!(
            parsed[0],
            Instruction::CreateComment {
                var_name: String::from("anchor_if"),
                text: String::from("@if(showExtra)"),
            }
        );
        assert_eq!(
            parsed[2],
            Instruction::SetAttribute {
                var_name: String::from("hdr"),
                key: String::from("subtitle"),
                value: String::from("GTK4 + Libadwaita"),
            }
        );
        assert_eq!(
            parsed[3],
            Instruction::InsertBefore {
                parent_var: String::from("vbox"),
                child_var: String::from("extra"),
                ref_var: Some(String::from("anchor_if")),
            }
        );
    }

    #[test]
    fn supports_empty_and_quoted_text_or_attribute_payloads() {
        let script = r#"
            text entry1 ""
            attr entry1 value ""
            attr entry1 placeholder "Escribe tu nombre aquí"
        "#;
        let parsed = parse_script(script).unwrap();
        assert_eq!(
            parsed[0],
            Instruction::SetText {
                var_name: String::from("entry1"),
                text: String::new(),
            }
        );
        assert_eq!(
            parsed[1],
            Instruction::SetAttribute {
                var_name: String::from("entry1"),
                key: String::from("value"),
                value: String::new(),
            }
        );
        assert_eq!(
            parsed[2],
            Instruction::SetAttribute {
                var_name: String::from("entry1"),
                key: String::from("placeholder"),
                value: String::from("Escribe tu nombre aquí"),
            }
        );
    }
}
