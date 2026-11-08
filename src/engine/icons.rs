use super::node::EngineError;
use std::fs;
use std::path::Path;

/// Represents a single resolution entry inside a multi-resolution icon container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconResolution {
    pub width: u16,
    pub height: u16,
    pub bits_per_pixel: u16,
}

/// Multi-resolution icon asset associated with a native `Window`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowIconAsset {
    pub app_id: String,
    pub source_path: String,
    pub resolutions: Vec<IconResolution>,
}

impl WindowIconAsset {
    /// Parses an `.ico` binary container from disk and extracts all embedded resolutions.
    pub fn from_ico_file(
        app_id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, EngineError> {
        let path_ref = path.as_ref();
        let bytes = fs::read(path_ref).map_err(|e| {
            EngineError::ConfigError(format!("Failed to read icon '{}': {e}", path_ref.display()))
        })?;
        let resolutions = Self::parse_ico_bytes(&bytes)?;
        Ok(Self {
            app_id: app_id.into(),
            source_path: path_ref.to_string_lossy().into_owned(),
            resolutions,
        })
    }

    /// Parses the binary `ICONDIR` header and `ICONDIRENTRY` table of an `.ico` buffer.
    pub fn parse_ico_bytes(bytes: &[u8]) -> Result<Vec<IconResolution>, EngineError> {
        if bytes.len() < 6 {
            return Err(EngineError::ConfigError(String::from(
                "Invalid .ico file: buffer shorter than 6-byte ICONDIR header",
            )));
        }

        let reserved = u16::from_le_bytes([bytes[0], bytes[1]]);
        let icon_type = u16::from_le_bytes([bytes[2], bytes[3]]);
        let count = usize::from(u16::from_le_bytes([bytes[4], bytes[5]]));

        if reserved != 0 || icon_type != 1 || count == 0 {
            return Err(EngineError::ConfigError(String::from(
                "Invalid .ico file: bad magic header or empty icon directory",
            )));
        }

        let required_len = 6 + count * 16;
        if bytes.len() < required_len {
            return Err(EngineError::ConfigError(String::from(
                "Invalid .ico file: truncated ICONDIRENTRY table",
            )));
        }

        let mut resolutions = Vec::with_capacity(count);
        for i in 0..count {
            let offset = 6 + i * 16;
            let raw_w = bytes[offset];
            let raw_h = bytes[offset + 1];
            let width = if raw_w == 0 { 256 } else { u16::from(raw_w) };
            let height = if raw_h == 0 { 256 } else { u16::from(raw_h) };
            let bits_per_pixel = u16::from_le_bytes([bytes[offset + 6], bytes[offset + 7]]);

            resolutions.push(IconResolution {
                width,
                height,
                bits_per_pixel,
            });
        }

        resolutions.sort_by_key(|r| r.width);
        Ok(resolutions)
    }

    pub fn format_resolutions(&self) -> String {
        self.resolutions
            .iter()
            .map(|r| format!("{}x{}", r.width, r.height))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_angular_favicon_ico_resolutions() {
        let asset = WindowIconAsset::from_ico_file(
            "ng-linux-counter",
            "examples/counter-app/public/favicon.ico",
        )
        .unwrap();
        assert_eq!(asset.app_id, "ng-linux-counter");
        assert_eq!(
            asset.resolutions,
            vec![
                IconResolution {
                    width: 16,
                    height: 16,
                    bits_per_pixel: 32,
                },
                IconResolution {
                    width: 32,
                    height: 32,
                    bits_per_pixel: 32,
                },
                IconResolution {
                    width: 48,
                    height: 48,
                    bits_per_pixel: 32,
                },
            ]
        );
    }

    #[test]
    fn rejects_invalid_ico_header() {
        let bad = [0u8, 0, 2, 0, 1, 0];
        assert!(WindowIconAsset::parse_ico_bytes(&bad).is_err());
    }
}
