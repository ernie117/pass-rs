use std::error::Error;
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::Terminal;
use tui::widgets::{Block, Borders, BorderType, Widget};

use crate::event::{Event, Events};

mod event;

struct PasswordTable {
    invert: bool
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    let mut table: PasswordTable = PasswordTable { invert: false };
    let mut colour: Color = Color::DarkGray;

    loop {
        if table.invert {
            colour = Color::Reset;
        } else {
            colour = Color::DarkGray;
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                    ].as_ref()
                )
                .split(Rect {
                    x: (f.size().width / 2) - 14,
                    y: f.size().height / 2 - 12,
                    width: 35,
                    height: 20,
                });
            let block = Block::default()
                .style(Style::default().bg(colour))
                .title("Passwords")
                .title_style(Style::default()
                    .bg(Color::DarkGray)
                    .modifier(Modifier::ITALIC))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default()
                    .bg(Color::DarkGray)
                    .modifier(Modifier::BOLD));
            f.render_widget(block, chunks[1]);
        })?;

        match events.next()? {
            Event::Input(key) => {
                if key == Key::Char('q') {
                    break;
                }
                if key == Key::Char('i') {
                    table.invert = !table.invert;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn copy_to_clipboard(string_to_copy: &str) -> Result<(), io::Error> {
    let process = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()?
        .stdin
        .unwrap()
        .write(string_to_copy.as_bytes());

    if let Err(e) = process {
        println!("Encountered error: {}", e);
    }

    Ok(())
}
