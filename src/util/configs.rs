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
    pub fn set_border_type(&mut self, border_type: String) {
        self.border_type = match border_type.to_ascii_lowercase().as_ref() {
            "rounded" => BorderType::Rounded,
            "plain" => BorderType::Plain,
            "double" => BorderType::Double,
            "thick" => BorderType::Thick,
            _ => BorderType::Plain,
        };
    }

    pub fn set_border_style(&mut self, modifier: String) {
        self.border_style = self.match_modifier(modifier);
    }

    pub fn set_title_style(&mut self, modifier: String) {
        self.title_style = self.match_modifier(modifier);
    }

    fn match_modifier(&mut self, modifier: String) -> Modifier {
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
}
