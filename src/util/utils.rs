use std::char;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};

#[inline]
pub fn build_table_rows(mut map: HashMap<String, String>, decrypt_key: u8) -> Vec<Vec<String>> {
  let mut vec_of_vecs = map
    .iter_mut()
    .map(|(key, value)| {
      vec![
        decrypt_value(key, decrypt_key).to_string(),
        value.to_string(),
      ]
    })
    .collect::<Vec<Vec<String>>>();

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
pub fn decrypt_value(string: &str, key: u8) -> String {
  string.chars().map(|ch| (key ^ ch as u8) as char).collect()
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

  let tmp = raw_password.as_os_str().to_str().unwrap().to_string();
  let raw_password_bytes = tmp.as_bytes();

  let final_password = encrypted_password.as_os_str().to_str().unwrap();

  if final_password.is_empty() {
    return false;
  }

  argon2::verify_encoded(final_password, raw_password_bytes).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use std::process::Command;

  #[test]
  fn test_encrypt_value() {
    assert_eq!(decrypt_value("password2", 6), "vguuqitb4");
  }

  #[test]
  fn test_decrypt_value() {
    assert_eq!(decrypt_value("vguuqitb4", 6), "password2");
  }

  #[test]
  fn test_build_rows() {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("rcuructpoec7".to_string(), "rcurvguuqitb7".to_string());
    map.insert("rcuructpoec4".to_string(), "rcurvguuqitb4".to_string());
    map.insert("rcuructpoec5".to_string(), "rcurvguuqitb5".to_string());

    let result = build_table_rows(map, 6);

    assert_eq!(
      result,
      vec![
        ["testservice1", "rcurvguuqitb7"],
        ["testservice2", "rcurvguuqitb4"],
        ["testservice3", "rcurvguuqitb5"],
      ]
    );
  }

  #[test]
  #[ignore]
  fn test_copy_to_clipboard() {
    copy_to_clipboard("test string").unwrap();
    let result = Command::new("pbpaste").output().unwrap();
    let text = String::from_utf8_lossy(&result.stdout);
    assert_eq!(text, "test string");
  }
}
