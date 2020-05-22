use std::error::Error;
use std::io;
use std::io::prelude::*;
use base64::decode;

use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::util::json_utils::{check_directory_exists, check_files, write_new_password};
use crate::util::utils::{verify_dev, decrypt, encrypt_known};
use aes_gcm::Aes128Gcm;
use aead::{Aead, NewAead, generic_array::GenericArray};

mod app;
mod stateful_table;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
  let mut key = String::new();

  key = if verify_dev() {
    "testing123456789".to_string()
  } else {
    let stdin = io::stdin();
    print!("Enter your key: ");
    io::stdout().flush()?;
    stdin.read_line(&mut key)?;
    key.trim_end().parse::<String>()?
  };

  let final_key = GenericArray::clone_from_slice(key.as_bytes());
  let aead = Aes128Gcm::new(final_key);

  check_directory_exists()?;
  check_files()?;

  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;

  if let Err(error) = app::run(&mut terminal, aead) {
    terminal.show_cursor()?;
    println!("Error rendering table: {}", error);
  }

  terminal.show_cursor()?;

  // write_new_password("testing", "password", key.as_bytes());
  // let thing = decrypt("LCg3S8mBFw946JaMD8GB+EpowACYQvJ2", key.as_bytes(), &decode("dmxEVGFMQUxEamYy")?);
  // println!("{:?}", thing);
  // println!("{:?}", String::from_utf8_lossy(&decode("dmxEVGFMQUxEamYy").expect("problems")));
  // println!("{:?}", encrypt_known("password", &aead, "HUUP0FAGadQh"));

  Ok(())
}
