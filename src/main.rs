use std::error::Error;
use std::io;
use std::io::prelude::*;

use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{Aes128Gcm, NewAead};

mod app;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let mut key = String::new();

    key = if util::utils::verify_dev() {
        std::env::var_os("PASSCURSES_DEV_PASSWORD_KEY")
            .ok_or("Dev key not set")
            .unwrap()
            .into_string()
            .unwrap()
    } else {
        let stdin = io::stdin();
        print!("Enter your key: ");
        io::stdout().flush()?;
        stdin.read_line(&mut key)?;
        key.trim_end().parse::<String>()?
    };

    let final_key = GenericArray::clone_from_slice(key.as_bytes());
    let aead = Aes128Gcm::new(&final_key);

    util::json_utils::check_directory_exists()?;
    util::json_utils::check_files(key)?;

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    if let Err(error) = app::run(&mut terminal, aead) {
        std::fs::write("debugging.txt", format!("Error rendering table: {}", error)).unwrap();
    }

    terminal.show_cursor()?;

    Ok(())
}
