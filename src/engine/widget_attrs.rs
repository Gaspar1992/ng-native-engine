use super::widget::WidgetKind;

impl WidgetKind {
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        match self {
            Self::Window { title, .. } if key == "title" => {
                *title = value.to_string();
            }
            Self::HeaderBar {
                title,
                subtitle,
                show_title_buttons,
                ..
            } => match key {
                "title" => *title = value.to_string(),
                "subtitle" => *subtitle = (!value.is_empty()).then(|| value.to_string()),
                "show-title-buttons" => *show_title_buttons = value != "false" && value != "0",
                _ => {}
            },
            Self::Scroll { min_height, .. } if key == "min-height" => {
                if let Ok(parsed) = Self::parse_px(value) {
                    *min_height = parsed;
                }
            }
            Self::Box {
                vertical, spacing, ..
            } => match key {
                "orientation" => *vertical = value != "horizontal",
                "spacing" | "gap" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *spacing = parsed;
                    }
                }
                _ => {}
            },
            Self::Grid {
                row_spacing,
                column_spacing,
                ..
            } => match key {
                "spacing" | "gap" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *row_spacing = parsed;
                        *column_spacing = parsed;
                    }
                }
                "row-spacing" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *row_spacing = parsed;
                    }
                }
                "column-spacing" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *column_spacing = parsed;
                    }
                }
                _ => {}
            },
            Self::FlowBox {
                row_spacing,
                column_spacing,
                min_children_per_line,
                max_children_per_line,
                homogeneous,
                ..
            } => match key {
                "spacing" | "gap" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *row_spacing = parsed;
                        *column_spacing = parsed;
                    }
                }
                "row-spacing" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *row_spacing = parsed;
                    }
                }
                "column-spacing" => {
                    if let Ok(parsed) = Self::parse_px(value) {
                        *column_spacing = parsed;
                    }
                }
                "min-children" | "min-per-row" => {
                    if let Ok(parsed) = value.trim().parse::<u32>() {
                        *min_children_per_line = parsed;
                    }
                }
                "max-children" | "max-per-row" => {
                    if let Ok(parsed) = value.trim().parse::<u32>() {
                        *max_children_per_line = parsed;
                    }
                }
                "homogeneous" => {
                    *homogeneous = value != "false" && value != "0";
                }
                _ => {}
            },
            Self::Separator { vertical } if key == "orientation" => {
                *vertical = value == "vertical";
            }
            _ => self.set_interactive_attribute(key, value),
        }
    }

    fn set_interactive_attribute(&mut self, key: &str, value: &str) {
        match self {
            Self::Entry {
                text, placeholder, ..
            } => match key {
                "value" | "text" => value.clone_into(text),
                "placeholder" => value.clone_into(placeholder),
                _ => {}
            },
            Self::Switch { active } => {
                if key == "active" || key == "checked" || key == "value" {
                    *active = value == "true" || value == "1";
                }
            }
            Self::CheckButton { label, active } => match key {
                "label" => value.clone_into(label),
                "active" | "checked" | "value" => *active = value == "true" || value == "1",
                _ => {}
            },
            Self::Slider {
                value: val,
                min,
                max,
            } => match key {
                "value" => {
                    if let Ok(v) = value.trim().parse::<i64>() {
                        *val = v.clamp(*min, *max);
                    }
                }
                "min" => {
                    if let Ok(m) = value.trim().parse::<i64>() {
                        *min = m;
                    }
                }
                "max" => {
                    if let Ok(m) = value.trim().parse::<i64>() {
                        *max = m;
                    }
                }
                _ => {}
            },
            Self::DropDown { options, selected } => match key {
                "options" => {
                    *options = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "selected" | "value" => {
                    if let Ok(idx) = value.trim().parse::<usize>() {
                        *selected = idx;
                    }
                }
                _ => {}
            },
            Self::Image { src, width, height } => match key {
                "src" => value.clone_into(src),
                "width" => {
                    if let Ok(w) = Self::parse_px(value) {
                        *width = w;
                    }
                }
                "height" => {
                    if let Ok(h) = Self::parse_px(value) {
                        *height = h;
                    }
                }
                _ => {}
            },
            Self::LevelBar {
                value: val,
                min,
                max,
            } => match key {
                "value" => {
                    if let Ok(v) = value.trim().parse::<f64>() {
                        *val = v;
                    }
                }
                "min" => {
                    if let Ok(m) = value.trim().parse::<f64>() {
                        *min = m;
                    }
                }
                "max" => {
                    if let Ok(m) = value.trim().parse::<f64>() {
                        *max = m;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn parse_px(raw: &str) -> Result<u32, std::num::ParseIntError> {
        let trimmed = raw.trim().trim_end_matches("px").trim();
        trimmed.parse::<u32>()
    }
}
