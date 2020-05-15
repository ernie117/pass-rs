use crate::stateful_table::StatefulPasswordTable;
use crate::util::ui::RenderMode;
use std::io::Write;
use termion::event::Key;

use std::io;

pub enum InputMode {
  Normal,
  NewService,
  NewPassword,
}

pub fn password_table_input_handler(table: &mut StatefulPasswordTable, key: Key) {
  match key {
    Key::Char('c') => {
      table.render_mode = RenderMode::NewPassword;
    }
    Key::Char('j') | Key::Down => {
      table.next();
    }
    Key::Char('k') | Key::Up => {
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
        RenderMode::NewService => RenderMode::NewService,
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

pub fn add_password_input_handler(table: &mut StatefulPasswordTable, key: Key) {
  io::stdout().flush().ok();

  match table.input_mode {
    InputMode::Normal => match key {
      Key::Char('i') => {
        table.input_mode = InputMode::NewService;
      }
      Key::Ctrl('c') => {
        table.render_mode = RenderMode::Normal;
        table.input_mode = InputMode::Normal;
        table.input.clear();
      }
      _ => {}
    },
    InputMode::NewService => match key {
      Key::Ctrl('c') => {
        table.input_mode = InputMode::Normal;
        table.input.clear();
      }
      Key::Char('\n') => {
        table.new_service.push_str(&table.input);
        table.input.clear();
        table.input_mode = InputMode::NewPassword;
      }
      Key::Char(c) => {
        table.input.push(c);
      },
      Key::Backspace => {
        table.input.pop();
      },
      _ => {}
    },
    InputMode::NewPassword => match key {
      Key::Ctrl('c') => {
        table.input_mode = InputMode::Normal;
        table.input.clear();
      }
      Key::Char('\n') => {
        table.new_password.push_str(&table.input);
        table.input.clear();
        table.input_mode = InputMode::Normal;
      }
      Key::Char(c) => {
        table.input.push(c);
      },
      Key::Backspace => {
        table.input.pop();
      },
      _ => {}
    },
  }
}
