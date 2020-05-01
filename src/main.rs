use std::error::Error;
use std::io;
use std::io::{Stdout, Write};
use std::process::{Command, Stdio};

use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, BorderType, Borders, Row, Table, Widget, TableState};
use tui::Terminal;

use crate::event::{Event, Events};

mod event;

struct PasswordTable {
    invert: bool,
}

struct StatefulPasswordTable<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
    encrypted: bool
}

impl<'a> StatefulPasswordTable<'a> {
    fn new() -> StatefulPasswordTable<'a> {
        StatefulPasswordTable {
            state: TableState::default(),
            items: vec![
                vec!["Gmail", "password1"],
                vec!["Outlook", "password2"],
                vec!["Reddit", "password3"],
                vec!["Twitch", "password4"]
            ],
            encrypted: false
        }
    }
    pub fn next(&mut self) {
        // If moving to a new password from a decrypted one, re-apply encryption.
        if self.encrypted {self.encrypted = !self.encrypted };
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.encrypted {self.encrypted = !self.encrypted };
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn decrypt(&mut self) {
        match self.state.selected() {
            Some(i) => self.encrypted = !self.encrypted,
            None => (),
        };
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // render_a_block(terminal);
    render_password_table(terminal);

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

fn render_password_table(
    mut terminal: Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>,
) -> Result<(), Box<dyn Error>> {
    let events = Events::new();
    let row_style = Style::default().fg(Color::White);

    let mut table = StatefulPasswordTable::new();

    loop {
        let mut highlight_colour = Color::Red;
        if table.encrypted {
            highlight_colour = Color::Green;
        }

        terminal.draw(|mut f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(Rect {
                    x: (f.size().width / 2) - 35,
                    y: (f.size().height / 2) - 12,
                    width: 70,
                    height: 24,
                });
            let header = ["Service", "Passwords"];

            let rows = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.into_iter(), row_style));
            let t = Table::new(
                ["Service", "Password"].iter(),
                vec![
                    Row::StyledData(["GGmailGmailGmailmail", "ppassword1password1assword1h"].iter(), row_style),
                    Row::StyledData(["Outlook", "password2"].iter(), row_style),
                    Row::StyledData(["Reddit", "password3"].iter(), row_style),
                    Row::StyledData(["Twitch", "password4"].iter(), row_style),
                ].into_iter(),
            )
                .block(
                    Block::default()
                        .title("Passwords")
                        .title_style(Style::default().modifier(Modifier::ITALIC))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().modifier(Modifier::BOLD)),
                )
                .header_style(Style::default().fg(Color::Yellow))
                .highlight_style(Style::default().fg(Color::Black).bg(highlight_colour))
                .widths(&[Constraint::Length(35), Constraint::Length(35)])
                .style(Style::default().fg(Color::White))
                .column_spacing(1);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;

        match events.next()? {
            Event::Input(key) => {
                if key == Key::Char('q') {
                    break;
                }
                if key == Key::Char('j') || key == Key::Down {
                    table.next();
                }
                if key == Key::Char('k') || key == Key::Up {
                    table.previous();
                }
                if key == Key::Char('d') {
                    table.decrypt();
                }
            },
            _ => {}
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn render_a_block(
    mut terminal: Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>,
) -> Result<(), Box<dyn Error>> {
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
                .constraints([Constraint::Percentage(10), Constraint::Percentage(10)].as_ref())
                .split(Rect {
                    x: (f.size().width / 2) - 14,
                    y: f.size().height / 2 - 12,
                    width: 35,
                    height: 20,
                });
            let block = Block::default()
                .style(Style::default().bg(colour))
                .title("Passwords")
                .title_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .modifier(Modifier::ITALIC),
                )
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .modifier(Modifier::BOLD),
                );
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
