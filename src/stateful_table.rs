use crate::util::json_utils::read_passwords;
use crate::util::utils::{build_table_rows, copy_to_clipboard, decrypt, encrypt_known};
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
    pub(crate) state: TableState,
    pub(crate) items: Vec<Vec<String>>,
    pub(crate) decrypted: bool,
    pub(crate) key: Aes128Gcm,
    pub(crate) input: String,
    pub(crate) current_mode: CurrentMode,
    pub(crate) active: bool,
    pub(crate) new_username: String,
    pub(crate) new_password: String,
}

impl StatefulPasswordTable {
    pub(crate) fn new(key: Aes128Gcm) -> StatefulPasswordTable {
        StatefulPasswordTable {
            state: TableState::default(),
            items: Vec::new(),
            decrypted: false,
            key,
            input: String::new(),
            current_mode: CurrentMode::Normal,
            active: true,
            new_username: String::new(),
            new_password: String::new(),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.items[i][1] =
                        encrypt_known(&self.items[i][1], &self.key, &self.items[i][2]);
                }
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        if self.decrypted {
            self.decrypted = false;
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.items[i][1] =
                        encrypt_known(&self.items[i][1], &self.key, &self.items[i][2]);
                }
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        if self.decrypted {
            self.decrypted = false;
        };
        self.state.select(Some(i));
    }

    pub fn decrypt(&mut self) {
        if self.items.len() == 0 {
            // So we don't subscript an array that's empty.
            return;
        }
        match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.decrypted = false;
                    self.items[i][1] =
                        encrypt_known(&self.items[i][1], &self.key, &self.items[i][2]);
                } else {
                    self.decrypted = true;
                    self.items[i][1] = decrypt(&self.items[i][1], &self.key, &self.items[i][2]);
                }
            }
            None => (),
        };
    }

    pub fn copy(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.decrypted {
                if let Err(error) = copy_to_clipboard(&self.items[i][1]) {
                    panic!("Error copying to clipboard: {}", error);
                }
                self.decrypted = false;
                self.items[i][1] = encrypt_known(&self.items[i][1], &self.key, &self.items[i][2]);
            } else {
                if let Err(error) =
                    copy_to_clipboard(&decrypt(&self.items[i][1], &self.key, &self.items[i][2]))
                {
                    panic!("Error copying to clipboard: {}", error);
                }
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
}
