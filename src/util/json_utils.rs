use crate::util::configs::{CursesConfigs, RawConfigs};
use crate::util::utils::encrypt;
use aes_gcm::Aes128Gcm;
use argon2::Config;
use base64::encode;
use dirs::home_dir;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;

enum FileType {
    Password,
    Config,
    Passrc,
}

#[derive(Debug, Serialize, Deserialize)]
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
    let cfg = CursesConfigs::new(
        raw_config.border_type,
        raw_config.border_style,
        raw_config.title_style,
    );

    Ok(cfg)
}

#[inline]
pub fn read_json_file(path: &str) -> Result<BufReader<File>, Box<dyn Error>> {
    let full_path = format!("{}/{}.json", get_home_dir(), path);
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

    let passwords_path = format!("{}/{}.json", &get_home_dir(), "passwords");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&passwords_path)?;

    file.write_all(new_passwords.as_bytes())?;

    Ok(())
}

pub fn delete_password(username_key: &str) -> Result<bool, Box<dyn Error>> {
    let bufreader = read_json_file("passwords")?;
    let mut map: HashMap<String, PasswordEntry> = match serde_json::from_reader(bufreader) {
        Ok(s) => s,
        Err(e) => panic!("Error serializing from reader: {}", e),
    };

    if map.remove_entry(username_key).is_none() {
        return Ok(false);
    };

    let new_passwords = serde_json::to_string_pretty(&map)?;

    let passwords_path = format!("{}/{}.json", &get_home_dir(), "passwords");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&passwords_path)?;

    file.write_all(new_passwords.as_bytes())?;

    Ok(true)
}

#[inline]
pub fn check_directory_exists() -> Result<(), Box<dyn Error>> {
    if !Path::new(&get_home_dir()).exists() {
        match fs::create_dir(get_home_dir()) {
            Ok(_s) => {}
            Err(e) => panic!("Could not create passcurses directory: {}", e),
        }
    }

    Ok(())
}

pub fn check_files(key: String) -> Result<(), Box<dyn Error>> {
    let home_dir = &get_home_dir();
    let passwords_path = format!("{}/{}.json", home_dir, "passwords");
    if !Path::new(&passwords_path).exists() {
        println!("Creating passwords json file...");
        populate_new_file(FileType::Password, passwords_path, None)?;
    }
    let config_path = format!("{}/{}.json", home_dir, "config");
    if !Path::new(&config_path).exists() {
        println!("Creating configuration json file...");
        populate_new_file(FileType::Config, config_path, None)?;
    }
    let passrc_path = format!("{}/{}.json", home_dir, "passrc");
    if !Path::new(&passrc_path).exists() {
        println!("Creating passrc json file...");
        populate_new_file(FileType::Passrc, passrc_path, Some(key))?;
    }

    Ok(())
}

#[inline]
fn populate_new_file(
    file_type: FileType,
    path: String,
    key: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut new_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;

    let template = match file_type {
        FileType::Password => json!({}).to_string(),
        FileType::Config => serde_json::to_string_pretty(&RawConfigs::default())?,
        FileType::Passrc => serde_json::to_string_pretty(&new_passrc(key.unwrap().as_bytes()))?,
    };

    Ok(new_file.write_all(template.as_bytes())?)
}

#[inline]
fn new_passrc(key: &[u8]) -> serde_json::Value {
    let mut salt = [0_u8; 16];
    thread_rng().try_fill(&mut salt[..]).unwrap();

    json!({
        "key": argon2::hash_encoded(&key, &salt, &Config::default()).unwrap(),
        "salt": salt,
    })
}

#[inline]
fn get_home_dir() -> String {
    home_dir().unwrap().into_os_string().into_string().unwrap() + "/.passcurses"
}
