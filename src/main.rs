use std::error::Error;
use std::io;
use std::io::prelude::*;

use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::rendering::render_password_table;
use crate::util::json_utils::check_directories_and_files;

mod rendering;
mod stateful_table;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
  let stdin = io::stdin();
  let mut key = String::new();
  print!("Enter your key: ");
  io::stdout().flush()?;
  stdin.read_line(&mut key)?;
  let u8_key = key.trim_end().parse::<u8>()?;
  check_directories_and_files()?;

  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;

  if let Err(error) = render_password_table(terminal, u8_key) {
    println!("Error rendering table: {}", error);
  }

  Ok(())
}
