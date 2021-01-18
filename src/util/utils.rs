use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};
use tui::style::{Modifier, Style};
use tui::widgets::Text;

use aead::{generic_array::GenericArray, Aead};
use aes_gcm::Aes128Gcm; // Or `Aes256Gcm`

use rand::distributions::Alphanumeric;
use rand::Rng;

use base64::{decode, encode};

use super::json_utils::PasswordEntry;

static BUTTONS: [&str; 8] = ["j/down", "k/up", "y", "d", "r", "c", "D", "q"];
static EFFECTS: [&str; 8] = [
    "move down",
    "move up",
    "copy password",
    "decrypt the password",
    "refresh passwords",
    "create new password",
    "delete password",
    "quit",
];

static HELP_MSG_SPACING: usize = 40;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct TableEntry {
    pub(crate) service: String,
    pub(crate) password: String,
    pub(crate) nonce: String,
}

impl TableEntry {
    fn new(service: String, password: String, nonce: String) -> Self {
        Self {
            service,
            password,
            nonce,
        }
    }
}

#[inline]
pub fn build_table_rows(map: HashMap<String, PasswordEntry>) -> Vec<Vec<String>> {
    let mut vec_of_vecs = Vec::new();
    for (key, password_entry) in map {
        vec_of_vecs.push(vec![key, password_entry.password, password_entry.nonce]);
    }

    vec_of_vecs.sort();

    vec_of_vecs
}

#[inline]
pub fn copy_to_clipboard(string_to_copy: &str) -> Result<(), Box<dyn Error>> {
    let process = if cfg!(target_os = "macos") {
        Command::new("pbcopy").stdin(Stdio::piped()).spawn()?
    } else {
        Command::new("xclip")
            .arg("-selection")
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
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .collect();

    let cipher_text = aead
        .encrypt(
            GenericArray::from_slice(nonce.as_bytes()),
            password.as_bytes().as_ref(),
        )
        .unwrap();

    (cipher_text, nonce)
}

#[inline]
pub fn encrypt_known(password: &str, aead: &Aes128Gcm, nonce: &str) -> String {
    let cipher_text = aead
        .encrypt(
            GenericArray::from_slice(nonce.as_bytes()),
            password.as_bytes().as_ref(),
        )
        .unwrap();

    encode(cipher_text)
}

#[inline]
pub fn decrypt(password: &str, aead: &Aes128Gcm, nonce: &str) -> String {
    let decoded_password = decode(password.as_bytes()).unwrap();
    let decrypted = aead
        .decrypt(
            GenericArray::from_slice(nonce.as_bytes()),
            decoded_password.as_ref(),
        )
        .expect("decryption failure!");
    String::from_utf8(decrypted).unwrap()
}

#[inline]
pub fn verify_dev() -> bool {
    let encrypted_password = match std::env::var_os("PASSCURSES_ENC_DEV_PASSWORD") {
        Some(value) => value,
        None => OsString::new(),
    };
    let raw_password = match std::env::var_os("PASSCURSES_RAW_DEV_PASSWORD") {
        Some(value) => value,
        None => OsString::new(),
    };

    match raw_password.as_os_str().to_str() {
        Some(_s) => {}
        None => return false,
    }

    let tmp = raw_password.into_string().unwrap();
    let raw_password_bytes = tmp.as_bytes();

    let final_password = encrypted_password.as_os_str().to_str().unwrap();

    if final_password.is_empty() {
        return false;
    }

    argon2::verify_encoded(final_password, raw_password_bytes).unwrap()
}

#[inline]
pub fn build_help_messages() -> Vec<Text<'static>> {
    let zipped_help = BUTTONS.iter().zip(EFFECTS.iter());

    let mut messages = Vec::new();
    for (button, effect) in zipped_help {
        let spacing = (HELP_MSG_SPACING - effect.len()) - button.len();
        let main_str = format!(
            "{} {:.<spacing$} {}",
            button,
            ".",
            effect,
            spacing = spacing as usize
        );
        messages.push(Text::styled(
            format!("{:^69}", main_str),
            Style::default().modifier(Modifier::ITALIC),
        ));
    }

    messages
}
