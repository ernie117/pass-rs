use crate::util::configs::{CursesConfigs, RawConfigs};
use crate::util::utils::encrypt;
use aes_gcm::Aes128Gcm;
use base64::encode;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub(crate) password: String,
    pub(crate) nonce: String,
}

impl PasswordEntry {
    fn new(new_password: String, new_nonce: String) -> Self {
        PasswordEntry {
            password: new_password,
            nonce: new_nonce,
        }
    }
}

#[inline]
pub fn read_passwords() -> Result<HashMap<String, PasswordEntry>, Box<dyn Error>> {
    let bufreader = read_json_file("passwords")?;
    let map: HashMap<String, PasswordEntry>;
    match serde_json::from_reader(bufreader) {
        Ok(s) => map = s,
        Err(e) => panic!("Error serializing from reader: {}", e),
    }

    Ok(map)
}

#[inline]
pub fn read_config() -> Result<CursesConfigs, Box<dyn Error>> {
    let bufreader = read_json_file("config")?;
    let raw_config: RawConfigs = serde_json::from_reader(bufreader)?;
    let mut cfg = CursesConfigs::default();
    cfg.set_border_type(raw_config.border_type);
    cfg.set_border_style(raw_config.border_style);
    cfg.set_title_style(raw_config.title_style);

    Ok(cfg)
}

#[inline]
pub fn read_json_file(path: &str) -> Result<BufReader<File>, Box<dyn Error>> {
    let full_path = format!("{}/{}.json", get_home_dir()?, path);
    let file = OpenOptions::new().read(true).write(true).open(&full_path)?;

    Ok(BufReader::new(file))
}

pub fn write_new_password(
    new_username: &str,
    new_password: &str,
    key: &Aes128Gcm,
) -> Result<(), Box<dyn Error>> {
    let bufreader = read_json_file("passwords")?;
    let mut map: HashMap<String, PasswordEntry> = match serde_json::from_reader(bufreader) {
        Ok(s) => s,
        Err(e) => panic!("Error serializing from reader: {}", e),
    };

    let (encrypted_pwd, pwd_nonce) = encrypt(new_password, &key);
    let new_entry = PasswordEntry::new(encode(encrypted_pwd), pwd_nonce);

    map.insert(new_username.to_string(), new_entry);

    let new_passwords = serde_json::to_string_pretty(&map)?;

    let passwords_path = format!("{}/{}.json", &get_home_dir()?, "passwords");
    let mut file = OpenOptions::new().write(true).open(&passwords_path)?;

    file.write_all(new_passwords.as_bytes())?;

    Ok(())
}

pub fn delete_password(username_key: &str) -> Result<bool, Box<dyn Error>> {
    let bufreader = read_json_file("passwords")?;
    let mut map: HashMap<String, PasswordEntry> = match serde_json::from_reader(bufreader) {
        Ok(s) => s,
        Err(e) => panic!("Error serializing from reader: {}", e),
    };

    let result = map.remove_entry(username_key);
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

#[inline]
pub fn check_directory_exists() -> Result<(), Box<dyn Error>> {
    if !Path::new(&get_home_dir()?).exists() {
        match fs::create_dir(get_home_dir()?) {
            Ok(_s) => {}
            Err(e) => panic!("Could not create passcurses directory: {}", e),
        }
    }

    Ok(())
}

pub fn check_files() -> Result<(), Box<dyn Error>> {
    let passwords_path = format!("{}/{}.json", &get_home_dir()?, "passwords");
    if !Path::new(&passwords_path).exists() {
        populate_new_file("passwords", passwords_path)?;
    }
    let config_path = format!("{}/{}.json", &get_home_dir()?, "config");
    if !Path::new(&config_path).exists() {
        populate_new_file("config", config_path)?;
    }

    Ok(())
}

#[inline]
fn populate_new_file(file_type: &str, path: String) -> Result<(), Box<dyn Error>> {
    let mut new_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;

    let mut new_template = json!({}).to_string();
    match file_type {
        "passwords" => {
            // remains an empty json
        }
        "config" => {
            new_template = serde_json::to_string_pretty(&RawConfigs::default())?;
        }
        _ => {}
    }
    new_file.write_all(new_template.as_bytes())?;

    Ok(())
}

#[inline]
fn get_home_dir() -> Result<String, Box<dyn Error>> {
    Ok(home_dir().unwrap().into_os_string().into_string().unwrap() + "/.passcurses")
}
