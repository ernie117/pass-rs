use std::error::Error;
use std::io;
use std::io::{Stdout, Write};
use std::process::{Command, Stdio};

use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, BorderType, Borders, List, Row, Table, TableState, Text};
use tui::Terminal;

use util::event::{Event, Events};
use util::json::{read_config_file, read_password_file};
use util::utils::{build_table_rows, copy_to_clipboard};

use crate::rendering::render_password_table;
use crate::stateful_table::StatefulPasswordTable;

mod rendering;
mod stateful_table;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    render_password_table(terminal);

    Ok(())
}
