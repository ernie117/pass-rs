use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};
use tui::style::{Modifier, Style};
use tui::text::{Span, Text};
use tui::widgets::{Cell, ListItem, Row};

use aes_gcm::aead::{generic_array::GenericArray, Aead};
use aes_gcm::Aes128Gcm; // Or `Aes256Gcm`

use rand::distributions::Alphanumeric;
use rand::Rng;

use base64::{decode, encode};

use super::json_utils::PasswordEntry;

static BUTTONS: [&str; 14] = [
    "j/down", "k/up", "Ctrl-d", "Ctrl-u", "g", "G", "M", "y", "d", "r", "c", "D", "?", "q",
];
static EFFECTS: [&str; 14] = [
    "move down",
    "move up",
    "move down x5",
    "move up x5",
    "jump to top",
    "jump to bottom",
    "Jump to middle",
    "copy password",
    "decrypt the password",
    "refresh passwords",
    "create new password",
    "delete password",
    "hide/show help",
    "quit",
];

static HELP_MSG_SPACING: usize = 40;

#[derive(Debug)]
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

    pub fn to_cells(&self) -> Row {
        Row::new(
            [&self.service, &self.password, &self.nonce]
                .iter()
                .map(|e| Cell::from(Span::raw(*e)))
                .collect::<Vec<Cell>>(),
        )
    }
}

#[inline]
pub fn build_table_rows(map: HashMap<String, PasswordEntry>) -> Vec<TableEntry> {
    let mut vec_of_vecs = map
        .into_iter()
        .map(|(k, v)| TableEntry::new(k, v.password, v.nonce))
        .collect::<Vec<TableEntry>>();

    vec_of_vecs.sort_by(|a, b| a.service.partial_cmp(&b.service).unwrap());

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
    let nonce: Vec<u8> = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .collect();

    let cipher_text = aead
        .encrypt(GenericArray::from_slice(&nonce), password.as_bytes())
        .unwrap();

    (cipher_text, String::from_utf8(nonce).unwrap())
}

#[inline]
pub fn encrypt_known(password: &str, aead: &Aes128Gcm, nonce: &str) -> String {
    encode(
        aead.encrypt(
            GenericArray::from_slice(nonce.as_bytes()),
            password.as_bytes(),
        )
        .unwrap(),
    )
}

#[inline]
pub fn decrypt(password: &str, aead: &Aes128Gcm, nonce: &str) -> String {
    let decoded_password = decode(password.as_bytes()).unwrap();
    if let Ok(decrypted) = aead.decrypt(
        GenericArray::from_slice(nonce.as_bytes()),
        decoded_password.as_ref(),
    ) {
        String::from_utf8(decrypted).unwrap()
    } else {
        String::from("Wrong login key for this password!")
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

#[inline]
pub fn build_help_messages() -> Vec<ListItem<'static>> {
    BUTTONS
        .iter()
        .zip(EFFECTS.iter())
        .map(|(b, e)| {
            let main_str = format!(
                "{} {:.<spacing$} {}",
                b,
                ".",
                e,
                spacing = (HELP_MSG_SPACING - e.len()) - b.len()
            );
            ListItem::new(Text::styled(
                format!("{:^69}", main_str),
                Style::default().add_modifier(Modifier::ITALIC),
            ))
        })
        .collect::<Vec<ListItem>>()
}
