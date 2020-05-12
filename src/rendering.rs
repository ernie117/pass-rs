use std::error::Error;
use std::io::{Stdout, Write};

use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Style};
use tui::{Frame, Terminal};

use crate::stateful_table::{InputMode, StatefulPasswordTable};
use crate::util::event::{Event, Events};
use crate::util::json_utils::{read_config, read_passwords, CursesConfigs};
use crate::util::utils::build_table_rows;
use std::io;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, Borders, List, Paragraph, Row, Table, TableState, Text};

type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;

pub fn render_password_table(
  mut terminal: Terminal<Backend>,
  key: u8,
) -> Result<(), Box<dyn Error>> {
  let mut events = Events::new();

  let mut table = StatefulPasswordTable::new(key);
  table.items = build_table_rows(read_passwords()?)?;
  let mut render_add_password = false;
  let mut coming_from_add = false;
  let mut pwd_input: Vec<String> = Vec::new();

  loop {
    // Re-reading the config in the loop allows for live editing of colours/style/etc.
    let cfg = read_config()?;

    let highlight_colour = if table.decrypted {
      Color::Green
    } else {
      Color::Red
    };

    terminal.draw(|mut f| {
      // Draw table
      draw_table(
        &mut table.state,
        &table.items,
        cfg,
        &mut f,
        highlight_colour,
      );

      // Draw help messages
      draw_help_window(&mut f);

      // Render the input box to enter a new password
      if render_add_password {
        let chunks = Layout::default()
          .direction(Direction::Vertical)
          .margin(2)
          .constraints(
            [
              Constraint::Length(1),
              Constraint::Length(3),
              Constraint::Min(1),
            ]
            .as_ref(),
          )
          .split(f.size());

        let title = match table.input_mode {
          InputMode::Normal => "Press q to exit, press i to enter service/password",
          InputMode::Insert => "Type service and password separated by ':'",
        };
        let text = [Text::raw(&table.input)];
        let input = Paragraph::new(text.iter())
          .style(Style::default().fg(Color::Yellow))
          .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(input, chunks[1]);
      }
    })?;

    if render_add_password {
      write!(terminal.backend_mut(), "{}", Goto(4 as u16, 5))?;
      // stdout is buffered, flush it to see the effect immediately when hitting backspace
      io::stdout().flush().ok();

      match events.next()? {
        Event::Input(key) => match table.input_mode {
          InputMode::Normal => match key {
            Key::Char('i') => {
              table.input_mode = InputMode::Insert;
              events.disable_exit_key();
            }
            Key::Char('q') => {
              render_add_password = false;
              coming_from_add = true;
              break;
            }
            _ => {}
          },
          InputMode::Insert => match key {
            Key::Char('\n') => {
              pwd_input.push(table.input.drain(..).collect());
            }
            Key::Char(c) => {
              table.input.push(c);
            }
            Key::Backspace => {
              table.input.pop();
            }
            Key::Esc => {
              table.input_mode = InputMode::Normal;
              events.enable_exit_key();
            }
            _ => {}
          },
        },
        _ => {}
      }

      continue;
    }

    match events.next()? {
      Event::Input(key) => match key {
        Key::Char('c') => {
          render_add_password = !render_add_password;
        }
        Key::Char('j') => {
          table.next();
        }
        Key::Down => {
          table.next();
        }
        Key::Char('k') => {
          table.previous();
        }
        Key::Up => {
          table.previous();
        }
        Key::Char('d') => {
          table.decrypt();
        }
        Key::Char('y') => {
          table.copy();
        }
        Key::Char('r') => {
          if let Err(e) = table.re_encrypt() {
            panic!("Error reading files: {}", e);
          }
        }
        Key::Char('q') => {
          break;
        }
        _ => {}
      },
      _ => {}
    }
  }

  if coming_from_add {
    render_password_table(terminal, key);
  }

  Ok(())
}

fn draw_table(
  table_state: &mut TableState,
  table_items: &Vec<Vec<String>>,
  cfg: CursesConfigs,
  f: &mut Frame<Backend>,
  highlight_colour: Color,
) {
  let row_style = Style::default().fg(Color::White);
  let rects = Layout::default()
    .constraints([Constraint::Percentage(100)].as_ref())
    .split(Rect {
      x: (f.size().width / 2) - 35,
      y: (f.size().height / 2) - 12,
      width: 70,
      height: 24,
    });

  let rows = table_items
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
  f.render_stateful_widget(t, rects[0], table_state);
}

fn draw_help_window(f: &mut Frame<Backend>) {
  let rects_2 = Layout::default()
    .constraints([Constraint::Percentage(100)].as_ref())
    .split(Rect {
      x: (f.size().width / 2) - 35,
      y: f.size().height - 12,
      width: 70,
      height: 7,
    });
  let messages = [
    "j/down to move down",
    "k/up to move up",
    "y to copy to clipboard",
    "d to decrypt",
    "r to refresh passwords",
  ]
  .iter()
  .map(|i| Text::raw(format!("{:^70}", i)));
  let help = List::new(messages).block(Block::default().borders(Borders::ALL).title("Help"));
  f.render_widget(help, rects_2[0]);
}
