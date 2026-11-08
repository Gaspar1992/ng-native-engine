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
    if let Some(reactive) = parse_reactive_instruction(line, &parts) {
        return reactive;
    }
    match parts.as_slice() {
        [
            "style",
            class_name,
            bg,
            fg,
            padding_str,
            spacing_str,
            extensions @ ..,
        ] => parse_style_instruction(
            line,
            [class_name, bg, fg, padding_str, spacing_str],
            extensions,
        ),
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

/// Parses reactive opcodes (`signal`, `bind_text`, `on_add`, `on_set`, `on_text`).
/// Returns `None` when the line is not a reactive instruction.
fn parse_reactive_instruction(
    line: &str,
    parts: &[&str],
) -> Option<Result<Instruction, EngineError>> {
    let parsed =
        match parts {
            ["signal", signal_name, initial_str] => {
                parse_i64(line, initial_str).map(|initial| Instruction::DeclareSignal {
                    name: (*signal_name).to_string(),
                    initial,
                })
            }
            ["bind_text", var_name, signal_name, rest @ ..] => Ok(Instruction::BindText {
                var_name: (*var_name).to_string(),
                signal_name: (*signal_name).to_string(),
                template: rest.join(" "),
            }),
            ["on_add", var_name, event_name, signal_name, delta_str] => parse_i64(line, delta_str)
                .map(|delta| Instruction::OnAdd {
                    var_name: (*var_name).to_string(),
                    event_name: (*event_name).to_string(),
                    signal_name: (*signal_name).to_string(),
                    delta,
                }),
            ["on_set", var_name, event_name, signal_name, value_str] => parse_i64(line, value_str)
                .map(|value| Instruction::OnSet {
                    var_name: (*var_name).to_string(),
                    event_name: (*event_name).to_string(),
                    signal_name: (*signal_name).to_string(),
                    value,
                }),
            ["on_text", var_name, event_name, target_var, rest @ ..] => Ok(Instruction::OnText {
                var_name: (*var_name).to_string(),
                event_name: (*event_name).to_string(),
                target_var: (*target_var).to_string(),
                text: rest.join(" "),
            }),
            _ => return None,
        };
    Some(parsed)
}

/// Parses `style <class> <bg> <fg> <padding> <spacing> [key=value ...]`.
/// Trailing `key=value` extensions (`radius`, `font-size`, `font-weight`) are optional,
/// so legacy 6-token bundles remain valid.
fn parse_style_instruction(
    line: &str,
    [class_name, bg, fg, padding_str, spacing_str]: [&str; 5],
    extensions: &[&str],
) -> Result<Instruction, EngineError> {
    let mut rule = parse_style_rule(line, class_name, bg, fg, padding_str, spacing_str)?;
    for ext in extensions {
        let (key, value) = ext
            .split_once('=')
            .ok_or_else(|| EngineError::ScriptSyntaxError(line.to_string()))?;
        if !rule.apply_extension(key, value) {
            return Err(EngineError::ScriptSyntaxError(line.to_string()));
        }
    }
    Ok(Instruction::RegisterStyle(rule))
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
        ..StyleRule::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_and_extended_style_instructions() {
        let legacy = parse_instruction("style card #242424 #ffffff 16 12").unwrap();
        let Instruction::RegisterStyle(legacy_rule) = legacy else {
            panic!("expected RegisterStyle");
        };
        assert_eq!(legacy_rule.border_radius, None);

        let extended = parse_instruction(
            "style card #242424 none 16 12 radius=14 font-size=20 font-weight=700",
        )
        .unwrap();
        let Instruction::RegisterStyle(rule) = extended else {
            panic!("expected RegisterStyle");
        };
        assert_eq!(rule.foreground, None);
        assert_eq!(rule.border_radius, Some(14));
        assert_eq!(rule.font_size, Some(20));
        assert_eq!(rule.font_weight, Some(700));

        assert!(parse_instruction("style card none none 0 0 radius").is_err());
        assert!(parse_instruction("style card none none 0 0 shadow=4").is_err());
    }

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
