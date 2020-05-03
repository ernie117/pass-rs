use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use tui::style::Modifier;
use tui::widgets::BorderType;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawConfigs {
    border_type: String,
    border_style: String,
    title_style: String,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordsTemplate {
    example_service: String,
}

impl Default for PasswordsTemplate {
    fn default() -> PasswordsTemplate {
        PasswordsTemplate {
            example_service: "example_password".to_string(),
        }
    }
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

pub fn read_passwords() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let bufreader = read_json_file("passwords")?;
    let map: HashMap<String, String> = serde_json::from_reader(bufreader)?;

    Ok(map)
}

pub fn read_config() -> Result<CursesConfigs, Box<dyn Error>> {
    let bufreader = read_json_file("config")?;
    let raw_config: RawConfigs = serde_json::from_reader(bufreader)?;
    let cfg: CursesConfigs = map_configs(raw_config);

    Ok(cfg)
}

pub fn read_json_file(path: &str) -> Result<BufReader<File>, Box<dyn Error>> {
    let full_path = home_dir().unwrap().into_os_string().into_string().unwrap()
        + "/.passcurses/"
        + path
        + ".json";
    let clone_path = full_path.clone();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(full_path)?;
    let metadata = fs::metadata(clone_path)?;

    if metadata.len() == 0 {
        if path == "config" {
            let default_json_config = serde_json::to_string_pretty(&RawConfigs::default())?;
            file.write_all(default_json_config.as_bytes())?;
        } else if path == "passwords" {
            let passwords_template = serde_json::to_string_pretty(&PasswordsTemplate::default())?;
            file.write_all(passwords_template.as_bytes())?;
        }
    }

    let bufreader = BufReader::new(file);

    Ok(bufreader)
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
