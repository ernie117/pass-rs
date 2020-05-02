use std::error::Error;
use std::io::Stdout;
use std::io::Write;

use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Style};
use tui::Terminal;

use crate::stateful_table::StatefulPasswordTable;
use crate::util::event::{Event, Events};
use crate::util::json::{read_config, read_passwords};
use crate::util::utils::build_table_rows;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::MouseTerminal;
use tui::layout::{Constraint, Layout, Rect};
use tui::widgets::{Block, Borders, List, Row, Table, Text};

pub fn render_password_table(
    mut terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
) -> Result<(), Box<dyn Error>> {
    let events = Events::new();
    let row_style = Style::default().fg(Color::White);

    let mut table = StatefulPasswordTable::new();

    loop {
        table.items = build_table_rows(read_passwords()?)?;
        let cfg = read_config()?;
        let mut highlight_colour = Color::Red;
        if table.encrypted {
            highlight_colour = Color::Green;
        }

        terminal.draw(|mut f| {
            // Layout and rendering for password table
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(Rect {
                    x: (f.size().width / 2) - 35,
                    y: (f.size().height / 2) - 12,
                    width: 70,
                    height: 24,
                });

            let rows = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.into_iter(), row_style));
            let t = Table::new(["Service", "Password"].iter(), rows)
                .block(
                    Block::default()
                        .title("Passwords")
                        .title_style(Style::default().modifier(cfg.title_style))
                        .borders(Borders::ALL)
                        .border_type(cfg.border_type)
                        .border_style(Style::default().modifier(cfg.border_style)),
                )
                .header_style(Style::default().fg(Color::Yellow))
                .highlight_style(Style::default().fg(Color::Black).bg(highlight_colour))
                .widths(&[Constraint::Length(35), Constraint::Length(35)])
                .style(Style::default().fg(Color::White))
                .column_spacing(1);
            f.render_stateful_widget(t, rects[0], &mut table.state);

            // Layout and rendering for help messages
            let rects_2 = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(Rect {
                    x: (f.size().width / 2) - 35,
                    y: f.size().height - 12,
                    width: 70,
                    height: 4,
                });
            let messages = [
                "j/down to move down",
                "k/up to move up",
                "y to copy to clipboard",
                "d to decrypt",
            ]
            .iter()
            .map(|i| Text::raw(format!("{:^70}", i)));
            let help =
                List::new(messages).block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help, rects_2[0]);
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
                    // table.decrypt();
                    // write!(
                        // terminal.backend_mut(),
                        // "{}",
                        // term_cursor::Goto(x, y)
                    // )?;
                }
                if key == Key::Char('y') {
                    table.copy();
                }
            }
            _ => {}
        }
    }

    Ok(())
}
