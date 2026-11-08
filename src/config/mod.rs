use crate::engine::EngineError;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AngularNativeConfig {
    pub project_name: String,
    pub target_platform: String,
    pub output_path: String,
}

impl AngularNativeConfig {
    /// Reads and extracts native build settings from an `angular.json` file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| EngineError::ConfigError(e.to_string()))?;

        let project_name = Self::extract_json_string(&content, "defaultProject")
            .ok_or_else(|| EngineError::ConfigError(String::from("Missing 'defaultProject'")))?;
        let target_platform = Self::extract_json_string(&content, "targetPlatform")
            .ok_or_else(|| EngineError::ConfigError(String::from("Missing 'targetPlatform'")))?;
        let output_path = Self::extract_json_string(&content, "outputPath")
            .ok_or_else(|| EngineError::ConfigError(String::from("Missing 'outputPath'")))?;

        Ok(Self {
            project_name,
            target_platform,
            output_path,
        })
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
