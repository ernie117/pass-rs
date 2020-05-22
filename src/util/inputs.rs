use crate::stateful_table::{CurrentMode, StatefulPasswordTable};
use crate::util::json_utils::delete_password;
use std::io::Write;
use termion::event::Key;

use super::json_utils::write_new_password;
use std::{error::Error, io};

pub fn password_table_input_handler(table: &mut StatefulPasswordTable, key: Key) {
    match key {
        Key::Char('c') => {
            table.current_mode = CurrentMode::NewUserName;
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
        Key::Char('D') => {
            table.current_mode = CurrentMode::DeletePassword;
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
            Key::Char('\n') => {
                if table.input.is_empty() {
                    ()
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
                    ()
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
        CurrentMode::PasswordCreated => match key {
            Key::Esc => {
                table.current_mode = CurrentMode::Normal;
            }
            _ => {}
        },
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
        CurrentMode::PasswordDeleted => match key {
            Key::Esc => {
                table.current_mode = CurrentMode::Normal;
            }
            _ => {}
        },
        CurrentMode::NoSuchPassword => match key {
            Key::Esc => {
                table.current_mode = CurrentMode::Normal;
            }
            _ => {}
        },
        _ => {}
    }
}
