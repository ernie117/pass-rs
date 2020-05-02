use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use tui::style::Modifier;
use tui::widgets::BorderType;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawConfigs {
    border_type: String,
    border_style: String,
    title_style: String,
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

pub fn read_password_file() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let path =
        home_dir().unwrap().into_os_string().into_string().unwrap() + "/.passcurses/passwords.json";
    let file = File::open(path)?;
    let bufreader = BufReader::new(file);

    let map: HashMap<String, String> = serde_json::from_reader(bufreader)?;

    Ok(map)
}

pub fn read_config_file() -> Result<CursesConfigs, Box<dyn Error>> {
    let path =
        home_dir().unwrap().into_os_string().into_string().unwrap() + "/.passcurses/config.json";
    let file = File::open(path)?;
    let bufreader = BufReader::new(file);

    let raw_config: RawConfigs = serde_json::from_reader(bufreader)?;

    let cfg: CursesConfigs = map_configs(raw_config);

    Ok(cfg)
}

fn map_configs(raw_config: RawConfigs) -> CursesConfigs {
    let mut cfg: CursesConfigs = CursesConfigs::default();

    cfg.border_type = match raw_config.border_type.to_ascii_lowercase().as_ref() {
        "rounded" => BorderType::Rounded,
        "plain" => BorderType::Plain,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        _ => cfg.border_type,
    };

    cfg.border_style = match raw_config.border_style.to_ascii_lowercase().as_ref() {
        "bold" => Modifier::BOLD,
        "dim" => Modifier::DIM,
        "italic" => Modifier::ITALIC,
        "underlined" => Modifier::UNDERLINED,
        "slow_blink" => Modifier::SLOW_BLINK,
        "rapid_blink" => Modifier::RAPID_BLINK,
        "reversed" => Modifier::REVERSED,
        "hidden" => Modifier::HIDDEN,
        "crossed_out" => Modifier::CROSSED_OUT,
        _ => cfg.border_style,
    };

    cfg.title_style = match raw_config.title_style.to_ascii_lowercase().as_ref() {
        "bold" => Modifier::BOLD,
        "dim" => Modifier::DIM,
        "italic" => Modifier::ITALIC,
        "underlined" => Modifier::UNDERLINED,
        "slow_blink" => Modifier::SLOW_BLINK,
        "rapid_blink" => Modifier::RAPID_BLINK,
        "reversed" => Modifier::REVERSED,
        "hidden" => Modifier::HIDDEN,
        "crossed_out" => Modifier::CROSSED_OUT,
        _ => cfg.border_style,
    };

    cfg
}
