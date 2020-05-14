use crate::stateful_table::StatefulPasswordTable;
use crate::util::ui::{InputMode, RenderMode};
use std::io::Write;
use termion::event::Key;

use std::io;

pub fn password_table_input_handler(table: &mut StatefulPasswordTable, key: Key) {
  match key {
    Key::Char('c') => {
      table.render_mode = RenderMode::NewPassword;
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
    Key::Char('?') => {
      table.render_mode = match table.render_mode {
        RenderMode::Normal => RenderMode::WithHelp,
        RenderMode::WithHelp => RenderMode::Normal,
        RenderMode::NewPassword => RenderMode::NewPassword,
      };
    }
    Key::Char('r') => {
      if let Err(e) = table.re_encrypt() {
        panic!("Error reading files: {}", e);
      }
    }
    Key::Char('q') => {
      table.active = false;
    }
    _ => {}
  }
}

pub fn add_password_input_handler(
  table: &mut StatefulPasswordTable,
  key: Key,
  pwd_input: &mut Vec<String>,
) {
  // stdout is buffered, flush it to see the effect immediately when hitting backspace
  io::stdout().flush().ok();

  match table.input_mode {
    InputMode::Normal => match key {
      Key::Char('i') => {
        table.input_mode = InputMode::Insert;
      }
      Key::Ctrl('c') => {
        table.render_mode = RenderMode::Normal;
        table.input_mode = InputMode::Normal;
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
      }
      _ => {}
    },
  }
}
