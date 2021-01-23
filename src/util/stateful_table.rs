use crate::util::inputs::{JumpDirection, LeapDirection, MoveDirection};
use crate::util::json_utils::read_passwords;
use crate::util::utils::{build_table_rows, copy_to_clipboard, decrypt, encrypt_known, TableEntry};
use aes_gcm::Aes128Gcm;
use std::error::Error;
use tui::widgets::TableState;

#[derive(Copy, Clone)]
pub enum CurrentMode {
    Normal,
    WithHelp,
    NewUserName,
    NewPassword,
    PasswordCreated,
    DeletePassword,
    PasswordDeleted,
    NoSuchPassword,
}

pub struct StatefulPasswordTable {
    pub(crate) active: bool,
    pub(crate) current_mode: CurrentMode,
    pub(crate) decrypted: bool,
    pub(crate) input: String,
    pub(crate) items: Vec<TableEntry>,
    pub(crate) key: Aes128Gcm,
    pub(crate) new_username: String,
    pub(crate) new_password: String,
    pub(crate) state: TableState,
}

impl StatefulPasswordTable {
    pub(crate) fn new(key: Aes128Gcm) -> StatefulPasswordTable {
        StatefulPasswordTable {
            active: true,
            current_mode: CurrentMode::Normal,
            decrypted: false,
            input: String::new(),
            items: Vec::new(),
            key,
            new_username: String::new(),
            new_password: String::new(),
            state: TableState::default(),
        }
    }

    pub fn select(&mut self, direction: MoveDirection) {
        if self.decrypted {
            self.decrypted = false;
        };
        self.state.select(Some(match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.items[i].password =
                        encrypt_known(&self.items[i].password, &self.key, &self.items[i].nonce);
                }
                match direction {
                    MoveDirection::DOWN => (i + 1) % self.items.len(),
                    MoveDirection::UP => self.backwards_wraparound(i),
                }
            }
            None => 0,
        }));
    }

    pub fn move_by_5(&mut self, direction: JumpDirection) {
        self.state.select(Some(match self.state.selected() {
            Some(i) => match direction {
                JumpDirection::DOWN => {
                    if i >= (self.items.len() - 5) {
                        self.items.len() - 1
                    } else {
                        i + 5
                    }
                }
                JumpDirection::UP => {
                    if i < 5 {
                        0
                    } else {
                        i - 5
                    }
                }
            },
            None => match direction {
                JumpDirection::DOWN => 5,
                JumpDirection::UP => self.items.len() - 5,
            },
        }));
    }

    pub fn leap(&mut self, direction: LeapDirection) {
        self.state.select(Some(match direction {
            LeapDirection::TOP => 0,
            LeapDirection::MIDDLE => self.items.len() / 2,
            LeapDirection::BOTTOM => self.items.len() - 1,
        }));
    }

    pub fn decrypt(&mut self) {
        if self.items.is_empty() {
            // So we don't subscript an array that's empty.
            return;
        }
        if let Some(i) = self.state.selected() {
            if self.decrypted {
                self.decrypted = false;
                self.items[i].password =
                    encrypt_known(&self.items[i].password, &self.key, &self.items[i].nonce);
            } else {
                self.decrypted = true;
                self.items[i].password =
                    decrypt(&self.items[i].password, &self.key, &self.items[i].nonce);
            }
        };
    }

    pub fn copy(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.decrypted {
                if let Err(error) = copy_to_clipboard(&self.items[i].password) {
                    panic!("Error copying to clipboard: {}", error);
                }
                self.decrypted = false;
                self.items[i].password =
                    encrypt_known(&self.items[i].password, &self.key, &self.items[i].nonce);
            } else if let Err(error) = copy_to_clipboard(&decrypt(
                &self.items[i].password,
                &self.key,
                &self.items[i].nonce,
            )) {
                panic!("Error copying to clipboard: {}", error);
            }
        }
    }

    pub fn re_encrypt(&mut self) -> Result<(), Box<dyn Error>> {
        self.items = build_table_rows(read_passwords()?);
        if self.decrypted {
            self.decrypted = !self.decrypted;
        }

        Ok(())
    }

    fn backwards_wraparound(&self, idx: usize) -> usize {
        (((idx as isize - 1) + self.items.len() as isize) % self.items.len() as isize) as usize
    }
}
