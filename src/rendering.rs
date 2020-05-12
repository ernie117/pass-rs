use std::error::Error;
use std::io::Write;

use tui::style::{Color, Style};
use tui::Terminal;

use crate::stateful_table::{InputMode, StatefulPasswordTable};
use crate::util::event::{Event, Events};
use crate::util::json_utils::{read_config, read_passwords};
use crate::util::ui;
use crate::util::ui::Backend;
use crate::util::utils::build_table_rows;
use std::io;
use termion::event::Key;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Text};

pub fn render_password_table(
  mut terminal: Terminal<Backend>,
  key: u8,
) -> Result<(), Box<dyn Error>> {
  let mut events = Events::new();

  let mut table = StatefulPasswordTable::new(key);
  table.items = build_table_rows(read_passwords()?)?;
  let mut render_add_password = false;
  let mut coming_from_add_password = false;
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
      ui::draw_table(
        &mut table.state,
        &table.items,
        cfg,
        &mut f,
        highlight_colour,
      );

      // Draw help messages
      ui::draw_help_window(&mut f);

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
          InputMode::Normal => "Press 'q' to close input, press 'i' to enter service/password",
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
              coming_from_add_password = true;
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

      if render_add_password {
        continue;
      }
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

  if coming_from_add_password {
    if let Err(error) = render_password_table(terminal, key) {
      println!("Error rendering table: {}", error);
    }
  }

  Ok(())
}
