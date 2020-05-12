use std::char;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn build_table_rows(
  mut map: HashMap<String, String>,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
  let mut vec_of_vecs = map
    .iter_mut()
    .map(|(key, value)| vec![key.to_string(), value.to_string()])
    .collect::<Vec<Vec<String>>>();

  vec_of_vecs.sort();
  Ok(vec_of_vecs)
}

#[inline]
pub fn copy_to_clipboard(string_to_copy: &str) -> Result<(), Box<dyn Error>> {
  let process = Command::new("xclip")
    .arg("-selection")
    .arg("clipboard")
    .stdin(Stdio::piped())
    .spawn()?
    .stdin
    .unwrap()
    .write(string_to_copy.as_bytes());

  if let Err(e) = process {
    panic!("Encountered error: {}", e);
  }

  Ok(())
}

#[inline]
pub fn decrypt_value(string: &str, key: u8) -> String {
  string
    .chars()
    .map(|ch| key ^ ch as u8)
    .map(|d| d as char)
    .collect()
}
