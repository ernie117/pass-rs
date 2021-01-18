use serde::{Deserialize, Serialize};
use tui::style::Modifier;
use tui::widgets::BorderType;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawConfigs {
    pub(crate) border_type: String,
    pub(crate) border_style: String,
    pub(crate) title_style: String,
}

impl Default for RawConfigs {
    fn default() -> RawConfigs {
        RawConfigs {
            border_type: "rounded".to_string(),
            border_style: "bold".to_string(),
            title_style: "italic".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CursesConfigs {
    pub border_type: BorderType,
    pub border_style: Modifier,
    pub title_style: Modifier,
}

impl Default for CursesConfigs {
    fn default() -> CursesConfigs {
        CursesConfigs {
            border_type: BorderType::Rounded,
            border_style: Modifier::BOLD,
            title_style: Modifier::ITALIC,
        }
    }
}

impl CursesConfigs {
    pub fn new(border_type: String, border_style: String, title_style: String) -> Self {
        CursesConfigs {
            border_type: match_border_type(border_type),
            border_style: match_modifier(border_style),
            title_style: match_modifier(title_style),
        }
    }
}

fn match_border_type(border_type: String) -> BorderType {
    let result = match border_type.to_ascii_lowercase().as_ref() {
        "rounded" => BorderType::Rounded,
        "plain" => BorderType::Plain,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        _ => BorderType::Plain,
    };

    result
}

fn match_modifier(modifier: String) -> Modifier {
    let result = match modifier.to_ascii_lowercase().as_ref() {
        "bold" => Modifier::BOLD,
        "dim" => Modifier::DIM,
        "italic" => Modifier::ITALIC,
        "underlined" => Modifier::UNDERLINED,
        "slow_blink" => Modifier::SLOW_BLINK,
        "rapid_blink" => Modifier::RAPID_BLINK,
        "reversed" => Modifier::REVERSED,
        "hidden" => Modifier::HIDDEN,
        "crossed_out" => Modifier::CROSSED_OUT,
        _ => Modifier::BOLD,
    };

    result
}
