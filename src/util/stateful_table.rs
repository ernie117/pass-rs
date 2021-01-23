use crate::util::inputs::{LeapDirection, MoveDirection};
use crate::util::json_utils::{read_passwords, write_new_password};
use crate::util::utils::{build_table_rows, copy_to_clipboard, decrypt, encrypt_known, TableEntry};
use aes_gcm::Aes128Gcm;
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

enum EncryptionMode {
    ENCRYPT,
    DECRYPT,
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
        self.state.select(Some(match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.decrypted = false;
                    self.items[i].password = self.encryption(EncryptionMode::ENCRYPT, i);
                }
                match direction {
                    MoveDirection::DOWN => (i + 1) % self.items.len(),
                    MoveDirection::UP => self.backwards_wraparound(i),
                }
            }
            None => 0,
        }));
    }

    pub fn move_by_5(&mut self, direction: MoveDirection) {
        self.state.select(Some(match self.state.selected() {
            Some(i) => match direction {
                MoveDirection::DOWN => {
                    if i >= (self.items.len() - 5) {
                        self.items.len() - 1
                    } else {
                        i + 5
                    }
                }
                MoveDirection::UP => {
                    if i < 5 {
                        0
                    } else {
                        i - 5
                    }
                }
            },
            None => match direction {
                MoveDirection::DOWN => 5,
                MoveDirection::UP => self.items.len() - 5,
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
            let mode: EncryptionMode;
            if self.decrypted {
                self.decrypted = false;
                mode = EncryptionMode::ENCRYPT;
            } else {
                self.decrypted = true;
                mode = EncryptionMode::DECRYPT;
            }
            self.items[i].password = self.encryption(mode, i);
        };
    }

    pub fn copy(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.decrypted {
                if let Err(error) = copy_to_clipboard(&self.items[i].password) {
                    panic!("Error copying to clipboard: {}", error);
                }
                self.decrypted = false;
                self.items[i].password = self.encryption(EncryptionMode::ENCRYPT, i);
            } else if let Err(error) =
                copy_to_clipboard(&self.encryption(EncryptionMode::DECRYPT, i))
            {
                panic!("Error copying to clipboard: {}", error);
            }
        }
    }

    pub fn new_username(&mut self) {
        if self.input.is_empty() {
            // do nothing
        } else {
            self.new_username.push_str(&self.input);
            self.input.clear();
            self.current_mode = CurrentMode::NewPassword;
        }
    }

    pub fn new_password(&mut self) {
        if self.input.is_empty() {
            // do nothing
        } else {
            self.new_password.push_str(&self.input);
            self.input.clear();
            self.current_mode = CurrentMode::PasswordCreated;

            if !self.new_username.is_empty()
                && !self.new_password.is_empty()
                && write_new_password(&self.new_username, &self.new_password, &self.key).is_ok()
            {
                self.new_username.clear();
                self.new_password.clear();
                self.re_encrypt();
            }
        }
    }

    pub fn re_encrypt(&mut self) {
        if let Ok(items) = read_passwords() {
            if self.decrypted {
                self.decrypted = !self.decrypted;
            }
            self.items = build_table_rows(items);
        }
    }

    fn encryption(&self, mode: EncryptionMode, idx: usize) -> String {
        match mode {
            EncryptionMode::ENCRYPT => {
                encrypt_known(&self.items[idx].password, &self.key, &self.items[idx].nonce)
            }
            EncryptionMode::DECRYPT => {
                decrypt(&self.items[idx].password, &self.key, &self.items[idx].nonce)
            }
        }
    }

    /// Decrements the highlighted index by 1 and wraps around to the last element
    /// if the current index is 0.
    ///
    /// Follows this formula:
    ///
    /// ((index - 1) + k) % k
    fn backwards_wraparound(&self, idx: usize) -> usize {
        (((idx as isize - 1) + self.items.len() as isize) % self.items.len() as isize) as usize
    }
}
