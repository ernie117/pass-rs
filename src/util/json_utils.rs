use crate::util::configs::{CursesConfigs, RawConfigs};
use crate::util::utils::decrypt_value;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct PasswordsTemplate {
  example_service1: String,
  example_service2: String,
  example_service3: String,
  example_service4: String,
}

impl Default for PasswordsTemplate {
  fn default() -> PasswordsTemplate {
    PasswordsTemplate {
      example_service1: "example_password1".to_string(),
      example_service2: "example_password2".to_string(),
      example_service3: "example_password3".to_string(),
      example_service4: "example_password4".to_string(),
    }
  }
}

pub fn read_passwords() -> Result<HashMap<String, String>, Box<dyn Error>> {
  let bufreader = read_json_file("passwords")?;
  let map: HashMap<String, String>;
  match serde_json::from_reader(bufreader) {
    Ok(s) => map = s,
    Err(e) => panic!("Error serializing from reader: {}", e),
  }

  Ok(map)
}

pub fn read_config() -> Result<CursesConfigs, Box<dyn Error>> {
  let bufreader = read_json_file("config")?;
  let raw_config: RawConfigs = serde_json::from_reader(bufreader)?;
  let mut cfg = CursesConfigs::default();
  cfg.set_border_type(raw_config.border_type);
  cfg.set_border_style(raw_config.border_style);
  cfg.set_title_style(raw_config.title_style);

  Ok(cfg)
}

pub fn read_json_file(path: &str) -> Result<BufReader<File>, Box<dyn Error>> {
  let full_path = format!("{}/{}.json", get_home_dir()?, path);
  let file = OpenOptions::new().read(true).write(true).open(&full_path)?;

  let bufreader = BufReader::new(file);

  Ok(bufreader)
}

pub fn write_new_password(
  new_username: &str,
  new_password: &str,
  key: u8,
) -> Result<(), Box<dyn Error>> {
  let bufreader = read_json_file("passwords")?;
  let mut map: HashMap<String, String> = match serde_json::from_reader(bufreader) {
    Ok(s) => s,
    Err(e) => panic!("Error serializing from reader: {}", e),
  };

  map.insert(
    decrypt_value(new_username, key).to_string(),
    decrypt_value(new_password, key).to_string(),
  );
  let new_passwords = serde_json::to_string_pretty(&map)?;

  let passwords_path = format!("{}/{}.json", &get_home_dir()?, "passwords");
  let mut file = OpenOptions::new().write(true).open(&passwords_path)?;

  file.write_all(new_passwords.as_bytes())?;

  Ok(())
}

pub fn delete_password(username_key: &str, key: u8) -> Result<bool, Box<dyn Error>> {
  let bufreader = read_json_file("passwords")?;
  let mut map: HashMap<String, String> = match serde_json::from_reader(bufreader) {
    Ok(s) => s,
    Err(e) => panic!("Error serializing from reader: {}", e),
  };

  let result = map.remove_entry(&decrypt_value(username_key, key));
  match result {
    None => return Ok(false),
    _ => {}
  };

  let new_passwords = serde_json::to_string_pretty(&map)?;

  let passwords_path = format!("{}/{}.json", &get_home_dir()?, "passwords");
  let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(&passwords_path)?;

  file.write_all(new_passwords.as_bytes())?;

  Ok(true)
}

pub fn check_directories_and_files() -> Result<(), Box<dyn Error>> {
  check_directory_exists()?;
  check_files()?;

  Ok(())
}

fn check_directory_exists() -> Result<(), Box<dyn Error>> {
  if !Path::new(&get_home_dir()?).exists() {
    match fs::create_dir(get_home_dir()?) {
      Ok(_s) => {}
      Err(e) => panic!("Could not create passcurses directory: {}", e),
    }
  }

  Ok(())
}

fn check_files() -> Result<(), Box<dyn Error>> {
  let passwords_path = format!("{}/{}.json", &get_home_dir()?, "passwords");
  if !Path::new(&passwords_path).exists() {
    let mut file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(&passwords_path)?;

    let passwords_template = serde_json::to_string_pretty(&PasswordsTemplate::default())?;
    file.write_all(passwords_template.as_bytes())?;
  }
  let config_path = format!("{}/{}.json", &get_home_dir()?, "config");
  if !Path::new(&config_path).exists() {
    let mut file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(&config_path)?;

    let default_json_config = serde_json::to_string_pretty(&RawConfigs::default())?;
    file.write_all(default_json_config.as_bytes())?;
  }

  Ok(())
}

fn get_home_dir() -> Result<String, Box<dyn Error>> {
  Ok(home_dir().unwrap().into_os_string().into_string().unwrap() + "/.passcurses")
}
