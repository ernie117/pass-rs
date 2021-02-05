use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

use aes_gcm::aead::{generic_array::GenericArray, Aead};
use aes_gcm::Aes128Gcm;

use rand::distributions::Alphanumeric;
use rand::Rng;

use base64::{decode, encode};

use super::json_utils::PasswordEntry;
use crate::util::stateful_table::TableEntry;

pub struct EncryptionData<'a> {
    pub password: &'a str,
    pub nonce: &'a str,
    pub key: &'a Aes128Gcm,
}

#[inline]
pub fn build_table_rows(map: HashMap<String, PasswordEntry>) -> Vec<TableEntry> {
    let mut entries = map
        .into_iter()
        .map(|(k, v)| TableEntry::new(k, v.password, v.nonce))
        .collect::<Vec<TableEntry>>();

    entries.sort_by(|a, b| a.service.partial_cmp(&b.service).unwrap());

    entries
}

#[inline]
pub fn copy_to_clipboard(string_to_copy: &str) -> Result<(), Box<dyn Error>> {
    let process = if cfg!(target_os = "macos") {
        Command::new("pbcopy").stdin(Stdio::piped()).spawn()?
    } else {
        Command::new("xclip")
            .arg("-select")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .spawn()?
    };

    if let Err(e) = process
        .stdin
        .ok_or("Couldn't unwrap stdin.")?
        .write_all(string_to_copy.as_bytes())
    {
        println!("Couldn't copy to clipboard: {}", e);
    }

    Ok(())
}

#[inline]
pub fn encrypt(password: &str, aead: &Aes128Gcm) -> (Vec<u8>, String) {
    let nonce: Vec<u8> = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .collect();

    let cipher_text = aead
        .encrypt(&mut GenericArray::from_slice(&nonce), password.as_bytes())
        .unwrap();

    (cipher_text, String::from_utf8(nonce).unwrap())
}

#[inline]
pub fn encrypt_known(data: EncryptionData) -> String {
    encode(
        data.key
            .encrypt(
                &mut GenericArray::from_slice(data.nonce.as_bytes()),
                data.password.as_bytes(),
            )
            .unwrap(),
    )
}

#[inline]
pub fn decrypt(data: EncryptionData) -> String {
    let decoded_password = decode(data.password.as_bytes()).unwrap();
    if let Ok(decrypted) = data.key.decrypt(
        &mut GenericArray::from_slice(data.nonce.as_bytes()),
        decoded_password.as_ref(),
    ) {
        String::from_utf8(decrypted).unwrap()
    } else {
        "Wrong login key for this password!".into()
    }
}

#[inline]
pub fn verify_dev() -> bool {
    let encrypted_password = match std::env::var_os("PASSCURSES_ENC_DEV_PASSWORD") {
        Some(value) => value.into_string().unwrap(),
        None => String::new(),
    };
    let raw_password = match std::env::var_os("PASSCURSES_RAW_DEV_PASSWORD") {
        Some(value) => value.into_string().unwrap(),
        None => String::new(),
    };

    if !encrypted_password.is_empty() {
        argon2::verify_encoded(&encrypted_password, raw_password.as_bytes()).unwrap()
    } else {
        false
    }
}
