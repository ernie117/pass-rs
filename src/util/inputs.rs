use crate::util::json_utils::delete_password;
use crate::util::stateful_table::{CurrentMode, StatefulPasswordTable};
use std::io::Write;
use termion::event::Key;

use super::json_utils::write_new_password;
use std::{error::Error, io};

pub enum JumpDirection {
    DOWN,
    UP,
}

pub enum LeapDirection {
    TOP,
    MIDDLE,
    BOTTOM,
}

pub enum MoveDirection {
    DOWN,
    UP,
}

pub fn password_table_input_handler(table: &mut StatefulPasswordTable, key: Key) {
    match key {
        Key::Char('c') => {
            table.current_mode = CurrentMode::NewUserName;
        }
        Key::Char('j') | Key::Down => {
            table.select(MoveDirection::DOWN);
        }
        Key::Char('k') | Key::Up => {
            table.select(MoveDirection::UP);
        }
        Key::Char('d') => {
            table.decrypt();
        }
        Key::Char('y') => {
            table.copy();
        }
        Key::Char('D') => {
            table.current_mode = CurrentMode::DeletePassword;
        }
        Key::Char('q') => {
            table.active = false;
        }
        Key::Ctrl('d') => {
            table.move_by_5(JumpDirection::DOWN);
        }
        Key::Ctrl('u') => {
            table.move_by_5(JumpDirection::UP);
        }
        Key::Char('g') => {
            table.leap(LeapDirection::TOP);
        }
        Key::Char('M') => {
            table.leap(LeapDirection::MIDDLE);
        }
        Key::Char('G') => {
            table.leap(LeapDirection::BOTTOM);
        }
        Key::Char('?') => {
            table.current_mode = match table.current_mode {
                CurrentMode::Normal => CurrentMode::WithHelp,
                CurrentMode::WithHelp => CurrentMode::Normal,
                _ => table.current_mode,
            };
        }
        Key::Char('r') => {
            if let Err(e) = table.re_encrypt() {
                panic!("Error reading files: {}", e);
            }
        }
        _ => {}
    }
}

pub fn with_help_input_handler(table: &mut StatefulPasswordTable, key: Key) {
    match key {
        Key::Char('?') => {
            table.current_mode = match table.current_mode {
                CurrentMode::Normal => CurrentMode::WithHelp,
                CurrentMode::WithHelp => CurrentMode::Normal,
                _ => table.current_mode,
            };
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
) -> Result<(), Box<dyn Error>> {
    io::stdout().flush().ok();

    match table.current_mode {
        CurrentMode::NewUserName => match key {
            Key::Esc => {
                table.current_mode = CurrentMode::Normal;
                table.input.clear();
                table.new_username.clear();
            }
            // TODO Need to check whether a password currently exists for the given service.
            Key::Char('\n') => {
                if table.input.is_empty() {
                    // do nothing
                }
                table.new_username.push_str(&table.input);
                table.input.clear();
                table.current_mode = CurrentMode::NewPassword;
            }
            Key::Char(c) => {
                table.input.push(c);
            }
            Key::Backspace => {
                table.input.pop();
            }
            _ => {}
        },
        CurrentMode::NewPassword => match key {
            Key::Esc => {
                table.current_mode = CurrentMode::Normal;
                table.input.clear();
                table.new_username.clear();
                table.new_password.clear();
            }
            Key::Char('\n') => {
                if table.input.is_empty() {
                    // do nothing
                }
                table.new_password.push_str(&table.input);
                table.input.clear();
                table.current_mode = CurrentMode::PasswordCreated;

                if !table.new_username.is_empty() && !table.new_password.is_empty() {
                    write_new_password(&table.new_username, &table.new_password, &table.key)?;
                    table.new_username.clear();
                    table.new_password.clear();
                    table.re_encrypt()?;
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
        CurrentMode::PasswordCreated => {
            if let Key::Esc = key {
                table.current_mode = CurrentMode::Normal;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn delete_password_input_handler(table: &mut StatefulPasswordTable, key: Key) {
    match table.current_mode {
        CurrentMode::DeletePassword => match key {
            Key::Esc => {
                table.current_mode = CurrentMode::Normal;
                table.input.clear();
            }
            Key::Char('\n') => {
                if table.input.is_empty() {
                    return;
                }
                let result = delete_password(&table.input).unwrap();
                if result {
                    // Password existed.
                    table.current_mode = CurrentMode::PasswordDeleted;
                    table.input.clear();
                    table.re_encrypt().unwrap();
                } else {
                    // Password didn't exist.
                    table.current_mode = CurrentMode::NoSuchPassword;
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
        CurrentMode::PasswordDeleted => {
            if let Key::Esc = key {
                table.current_mode = CurrentMode::Normal;
            }
        }
        CurrentMode::NoSuchPassword => {
            if let Key::Esc = key {
                table.current_mode = CurrentMode::Normal;
            }
        }
        _ => {}
    }
}
