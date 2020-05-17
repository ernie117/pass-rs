use crate::stateful_table::StatefulPasswordTable;
use crate::util::json_utils::delete_password;
use crate::util::ui::RenderMode;
use std::io::Write;
use termion::event::Key;

use std::io;

pub enum InputMode {
  Normal,
  NewUserName,
  NewPassword,
  PasswordCreated,
  DeletePassword,
  PasswordDeleted,
  NoSuchPassword,
}

pub fn password_table_input_handler(table: &mut StatefulPasswordTable, key: Key) {
  match key {
    Key::Char('c') => {
      table.render_mode = RenderMode::NewUserName;
      table.input_mode = InputMode::NewUserName;
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
        RenderMode::NewUserName => RenderMode::NewUserName,
        RenderMode::DeletePassword => RenderMode::DeletePassword,
        RenderMode::PasswordDeleted => RenderMode::PasswordDeleted,
        RenderMode::PasswordCreated => RenderMode::PasswordCreated,
        RenderMode::NoSuchPassword => RenderMode::NoSuchPassword,
      };
    }
    Key::Char('r') => {
      if let Err(e) = table.re_encrypt() {
        panic!("Error reading files: {}", e);
      }
    }
    Key::Char('D') => {
      table.render_mode = RenderMode::DeletePassword;
      table.input_mode = InputMode::DeletePassword;
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
    InputMode::NewUserName => match key {
      Key::Esc => {
        table.render_mode = RenderMode::Normal;
        table.input_mode = InputMode::Normal;
        table.input.clear();
        table.new_username.clear();
      }
      Key::Char('\n') => {
        table.new_username.push_str(&table.input);
        table.input.clear();
        table.input_mode = InputMode::NewPassword;
        table.render_mode = RenderMode::NewPassword;
      }
      Key::Char(c) => {
        table.input.push(c);
      }
      Key::Backspace => {
        table.input.pop();
      }
      _ => {}
    },
    InputMode::NewPassword => match key {
      Key::Esc => {
        table.render_mode = RenderMode::Normal;
        table.input_mode = InputMode::Normal;
        table.input.clear();
        table.new_username.clear();
        table.new_password.clear();
      }
      Key::Char('\n') => {
        table.new_password.push_str(&table.input);
        table.input.clear();
        table.input_mode = InputMode::PasswordCreated;
        table.render_mode = RenderMode::PasswordCreated;
      }
      Key::Char(c) => {
        table.input.push(c);
      }
      Key::Backspace => {
        table.input.pop();
      }
      _ => {}
    },
    InputMode::PasswordCreated => match key {
      Key::Esc => {
        table.input_mode = InputMode::Normal;
        table.render_mode = RenderMode::Normal;
      }
      _ => {}
    },
    _ => {}
  }
}

pub fn delete_password_input_handler(table: &mut StatefulPasswordTable, key: Key) {
  match table.input_mode {
    InputMode::DeletePassword => match key {
      Key::Esc => {
        table.input_mode = InputMode::Normal;
        table.render_mode = RenderMode::Normal;
        table.input.clear();
      }
      Key::Char('\n') => {
        if table.input.is_empty() {
          return;
        }
        let result = delete_password(&table.input, table.key).unwrap();
        if result {
          // Password existed.
          table.input_mode = InputMode::PasswordDeleted;
          table.render_mode = RenderMode::PasswordDeleted;
          table.input.clear();
          table.re_encrypt().unwrap();
        } else {
          // Password didn't exist.
          table.input_mode = InputMode::NoSuchPassword;
          table.render_mode = RenderMode::NoSuchPassword;
          table.input.clear();
        }
      }
      Key::Char(c) => {
        table.input.push(c);
      }
      Key::Backspace => {
        table.input.pop();
      }
      _ => {}
    },
    InputMode::PasswordDeleted => match key {
      Key::Esc => {
        table.input_mode = InputMode::Normal;
        table.render_mode = RenderMode::Normal;
      }
      _ => {}
    },
    InputMode::NoSuchPassword => match key {
      Key::Esc => {
        table.input_mode = InputMode::Normal;
        table.render_mode = RenderMode::Normal;
      }
      _ => {}
    }
    _ => {}
  }
}
