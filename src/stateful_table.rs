use crate::util::utils::{copy_to_clipboard, decrypt};
use std::fmt::Write as FmtWrite;
use tui::widgets::TableState;

pub struct StatefulPasswordTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<Vec<String>>,
    pub(crate) encrypted: bool,
}

impl StatefulPasswordTable {
    pub(crate) fn new() -> StatefulPasswordTable {
        StatefulPasswordTable {
            state: TableState::default(),
            items: Vec::new(),
            encrypted: false,
        }
    }
    pub fn next(&mut self) {
        // If moving to a new password from a decrypted one, re-apply encryption.
        if self.encrypted {
            self.encrypted = !self.encrypted
        };
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.encrypted {
            self.encrypted = !self.encrypted
        };
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn decrypt(&mut self) {
        // TODO find a way to actually obfuscate the passwords
        match self.state.selected() {
            Some(i) => {
                self.encrypted = !self.encrypted;
            }
            None => (),
        };
    }

    pub fn copy(&mut self) {
        if let Some(i) = self.state.selected() {
            if let Err(error) = copy_to_clipboard(&decrypt(&self.items[i][1])) {
                println!("Error copying to clipboard: {}", error);
            }
        }
    }
}
