use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};

use aes_gcm::Aes128Gcm; // Or `Aes256Gcm`
use aead::{Aead, NewAead, generic_array::GenericArray};

use rand::Rng;
use rand::distributions::Alphanumeric;

use base64::{encode, decode};

use super::json_utils::PasswordEntry;
use std::convert::TryInto;

#[inline]
pub fn build_table_rows(map: HashMap<String, PasswordEntry>) -> Vec<Vec<String>> {
  let mut vec_of_vecs = Vec::new();
  for (key, value) in map {
    vec_of_vecs.push(
      vec![
      key,
      value.password,
      value.nonce
    ]
    );
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

pub fn encrypt<'a>(password: &'a str, password_key: &'a [u8]) -> (Vec<u8>, String) {
  let key = GenericArray::clone_from_slice(password_key);
  let aead = Aes128Gcm::new(key);
  let nonce: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(12)
    .collect();

  let cipher_text = aead.encrypt(GenericArray::from_slice(nonce.as_bytes()), password.as_bytes().as_ref()).unwrap();

  (cipher_text, nonce)
}

pub fn encrypt_known(password: &str, password_key: &[u8], nonce: &[u8]) -> String {
  let key = GenericArray::clone_from_slice(password_key);
  let aead = Aes128Gcm::new(key);

  let cipher_text = aead.encrypt(GenericArray::from_slice(nonce), password.as_bytes().as_ref()).unwrap();

  encode(String::from_utf8_lossy(&cipher_text).to_string())
}

pub fn decrypt(password: &str, password_key: &[u8], nonce: &[u8]) -> String {
  let key = GenericArray::clone_from_slice(password_key);
  let aead = Aes128Gcm::new(key);

  let decoded_password = decode(password.as_bytes()).unwrap();
  let decrypted = aead.decrypt(GenericArray::from_slice(nonce), decoded_password.as_ref()).expect("decryption failure!");
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
    Some(_s) => {},
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

