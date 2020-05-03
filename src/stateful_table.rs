use crate::util::utils::{copy_to_clipboard, decrypt_value, build_table_rows};
use tui::widgets::TableState;
use crate::util::json::read_passwords;
use std::error::Error;

pub struct StatefulPasswordTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<Vec<String>>,
    pub(crate) decrypted: bool,
}

impl StatefulPasswordTable {
    pub(crate) fn new() -> StatefulPasswordTable {
        StatefulPasswordTable {
            state: TableState::default(),
            items: Vec::new(),
            decrypted: false,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.items[i][1] = decrypt_value(&self.items[i][1]);
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
            self.decrypted = !self.decrypted
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.decrypted {
                    self.items[i][1] = decrypt_value(&self.items[i][1]);
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
            self.decrypted = !self.decrypted
        };
        self.state.select(Some(i));
    }

    pub fn decrypt(&mut self) {
        match self.state.selected() {
            Some(i) => {
                self.decrypted = !self.decrypted;
                self.items[i][1] = decrypt_value(&self.items[i][1]);
            }
            None => (),
        };
    }

    pub fn copy(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.decrypted {
                if let Err(error) = copy_to_clipboard(&self.items[i][1]) {
                    println!("Error copying to clipboard: {}", error);
                }
            } else {
                if let Err(error) = copy_to_clipboard(&decrypt_value(&self.items[i][1])) {
                    println!("Error copying to clipboard: {}", error);
                }
            }
        }
    }

    pub fn re_encrypt(&mut self) -> Result<(), Box<dyn Error>> {
        self.items = build_table_rows(read_passwords()?)?;
        if self.decrypted {
            self.decrypted = !self.decrypted;
        }

        Ok(())
    }
}
