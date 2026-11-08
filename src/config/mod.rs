use crate::engine::EngineError;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AngularNativeConfig {
    pub project_name: String,
    pub target_platform: String,
    pub js_engine: String,
    pub output_path: String,
}

impl AngularNativeConfig {
    /// Reads and extracts native build settings from an Angular CLI `angular.json` file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| EngineError::ConfigError(e.to_string()))?;

        let project_name = Self::extract_json_string(&content, "defaultProject")
            .or_else(|| Self::extract_first_project_name(&content))
            .ok_or_else(|| {
                EngineError::ConfigError(String::from("Missing project in 'angular.json'"))
            })?;
        let target_platform = Self::extract_json_string(&content, "targetPlatform")
            .ok_or_else(|| EngineError::ConfigError(String::from("Missing 'targetPlatform'")))?;
        let js_engine =
            Self::extract_json_string(&content, "jsEngine").unwrap_or_else(|| String::from("jsc"));
        let output_path = Self::extract_json_string(&content, "outputPath")
            .ok_or_else(|| EngineError::ConfigError(String::from("Missing 'outputPath'")))?;

        Ok(Self {
            project_name,
            target_platform,
            js_engine,
            output_path,
        })
    }

    fn extract_first_project_name(json: &str) -> Option<String> {
        let projects_key = "\"projects\"";
        let start = json.find(projects_key)? + projects_key.len();
        let rest = &json[start..];
        let brace = rest.find('{')? + 1;
        let inside = rest[brace..].trim_start();
        if !inside.starts_with('"') {
            return None;
        }
        let name_part = &inside[1..];
        let end_quote = name_part.find('"')?;
        Some(name_part[..end_quote].to_string())
    }

    fn extract_json_string(json: &str, key: &str) -> Option<String> {
        let pattern = format!("\"{key}\"");
        let start = json.find(&pattern)? + pattern.len();
        let rest = &json[start..];
        let colon = rest.find(':')? + 1;
        let after_colon = rest[colon..].trim_start();
        if !after_colon.starts_with('"') {
            return None;
        }
        let value_part = &after_colon[1..];
        let end_quote = value_part.find('"')?;
        Some(value_part[..end_quote].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_workspace_angular_json_configuration() {
        let cfg = AngularNativeConfig::from_file("angular.json").unwrap();
        assert_eq!(cfg.project_name, "ng-linux-counter");
        assert_eq!(cfg.target_platform, "linux-gtk4");
        assert_eq!(cfg.js_engine, "jsc");
        assert_eq!(cfg.output_path, "dist/ng-linux-counter");
    }
}
