use crate::util::inputs::{LeapDirection, MoveDirection};
use crate::util::json_utils::{delete_password, read_passwords, write_new_password};
use crate::util::utils::{build_table_rows, copy_to_clipboard, decrypt, encrypt_known};
use aes_gcm::{aead::generic_array::GenericArray, Aes128Gcm, NewAead};
use tui::text::Span;
use tui::widgets::{Cell, Row, TableState};

use std::convert::TryInto;

#[derive(Copy, Clone, Debug, PartialEq)]
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

pub enum EntryState {
    PasswordDeleted,
    NoSuchPassword,
}

enum EncryptionMode {
    ENCRYPT,
    DECRYPT,
}

#[derive(Debug)]
pub struct TableEntry {
    pub(crate) service: String,
    pub(crate) password: String,
    pub(crate) nonce: String,
}

impl Default for TableEntry {
    fn default() -> Self {
        let cipher = Aes128Gcm::new(GenericArray::from_slice(b"testing987654321"));
        let nonce = "asdfjklqasdf";
        let password = encrypt_known("test_pass", &cipher, nonce);

        Self {
            service: String::from("test_user"),
            password,
            nonce: String::from(nonce),
        }
    }
}

impl TableEntry {
    pub fn new(service: String, password: String, nonce: String) -> Self {
        Self {
            service,
            password,
            nonce,
        }
    }

    pub fn to_cells(&self) -> Row {
        Row::new(
            [&self.service, &self.password, &self.nonce]
                .iter()
                .map(|e| Cell::from(Span::raw(*e)))
                .collect::<Vec<Cell>>(),
        )
    }
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
            None => match direction {
                MoveDirection::DOWN => 0,
                MoveDirection::UP => self.items.len() - 1,
            },
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
            LeapDirection::MIDDLE => {
                if self.items.len() % 2 == 0 {
                    (self.items.len() / 2) - 1
                } else {
                    self.items.len() / 2
                }
            }
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
                && write_new_password(
                    self.new_username.drain(..).collect(),
                    self.new_password.drain(..).collect(),
                    &self.key,
                )
                .is_ok()
            {
                self.re_encrypt();
            }
        }
    }

    pub fn delete_entry(&mut self) {
        if self.input.is_empty() {
            return;
        }
        match delete_password(&self.input).unwrap() {
            EntryState::PasswordDeleted => {
                self.current_mode = CurrentMode::PasswordDeleted;
                self.input.clear();
                self.re_encrypt();
            }
            EntryState::NoSuchPassword => {
                self.current_mode = CurrentMode::NoSuchPassword;
                self.input.clear();
            }
        }
    }

    pub fn clear_inputs(&mut self) {
        self.input.clear();
        self.new_username.clear();
        self.new_password.clear();
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
        // Should handle the results of these `try_into`s properly
        // but I don't think it's mathematically possible for None
        // to come out of either of them. The index coming in should
        // always be >= 0.
        let len_isize: isize = self.items.len().try_into().unwrap();

        (((idx as isize - 1) + len_isize) % len_isize)
            .try_into()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::json_utils::delete_password;
    use aes_gcm::{aead::generic_array::GenericArray, Aes128Gcm, NewAead};
    use std::process::Command;

    // Only need this implementation for tests.
    impl Default for StatefulPasswordTable {
        fn default() -> Self {
            Self {
                active: true,
                current_mode: CurrentMode::Normal,
                decrypted: false,
                input: String::new(),
                items: vec![
                    TableEntry::default(),
                    TableEntry::default(),
                    TableEntry::default(),
                ],
                key: Aes128Gcm::new(GenericArray::from_slice(b"testing987654321")),
                new_username: String::new(),
                new_password: String::new(),
                state: TableState::default(),
            }
        }
    }

    #[test]
    fn test_highlight_movement_down_from_no_selection() {
        let mut table = StatefulPasswordTable::default();
        table.select(MoveDirection::DOWN);
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_highlight_movement_up_from_no_selection() {
        let mut table = StatefulPasswordTable::default();
        table.select(MoveDirection::UP);
        assert_eq!(table.state.selected(), Some(table.items.len() - 1));
    }

    #[test]
    fn test_highlight_movement_down_from_first_selection() {
        let mut table = StatefulPasswordTable::default();
        table.state.select(Some(0));
        table.select(MoveDirection::DOWN);
        assert_eq!(table.state.selected(), Some(1));
        table.select(MoveDirection::DOWN);
        assert_eq!(table.state.selected(), Some(2));
    }

    #[test]
    fn test_highlight_movement_up_from_last_selection() {
        let mut table = StatefulPasswordTable::default();
        table.state.select(Some(table.items.len() - 1));
        table.select(MoveDirection::UP);
        assert_eq!(table.state.selected(), Some(1));
        table.select(MoveDirection::UP);
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_highlight_movement_top_wrap_around() {
        let mut table = StatefulPasswordTable::default();
        table.state.select(Some(0));
        table.select(MoveDirection::UP);
        assert_eq!(table.state.selected(), Some(table.items.len() - 1));
    }

    #[test]
    fn test_highlight_movement_bottom_wrap_around() {
        let mut table = StatefulPasswordTable::default();
        table.state.select(Some(table.items.len() - 1));
        table.select(MoveDirection::DOWN);
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_highlight_movement_down_by_5() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        table.state.select(Some(0));
        table.move_by_5(MoveDirection::DOWN);
        assert_eq!(table.state.selected(), Some(5));
    }

    #[test]
    fn test_highlight_movement_up_by_5() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        table.move_by_5(MoveDirection::UP);
        assert_eq!(table.state.selected(), Some(5));
    }

    #[test]
    fn test_highlight_movement_down_by_5_when_within_5_moves() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.state.select(Some(6));
        table.move_by_5(MoveDirection::DOWN);
        assert_eq!(table.state.selected(), Some(table.items.len() - 1));
    }

    #[test]
    fn test_highlight_movement_up_by_5_when_within_5_moves() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(2));
        assert_eq!(table.items.len(), 5);
        table.state.select(Some(2));
        table.move_by_5(MoveDirection::UP);
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_leap_bottom() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.state.select(Some(0));
        table.leap(LeapDirection::BOTTOM);
        assert_eq!(table.state.selected(), Some(9));
    }

    #[test]
    fn test_leap_bottom_when_no_selection() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.leap(LeapDirection::BOTTOM);
        assert_eq!(table.state.selected(), Some(9));
    }

    #[test]
    fn test_leap_top() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.state.select(Some(table.items.len() - 1));
        table.leap(LeapDirection::TOP);
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_leap_top_when_no_selection() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.leap(LeapDirection::TOP);
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_leap_middle_when_no_selection_even_items() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.leap(LeapDirection::MIDDLE);
        assert_eq!(table.state.selected(), Some(4));
    }

    #[test]
    fn test_leap_middle_with_selection_even_items() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(7));
        assert_eq!(table.items.len(), 10);
        table.state.select(Some(8));
        table.leap(LeapDirection::MIDDLE);
        assert_eq!(table.state.selected(), Some(4));
    }

    #[test]
    fn test_leap_middle_with_odd_no_of_items() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(8));
        assert_eq!(table.items.len(), 11);
        table.state.select(Some(1));
        table.leap(LeapDirection::MIDDLE);
        assert_eq!(table.state.selected(), Some(5));
    }

    #[test]
    fn test_leap_middle_with_odd_no_of_items_no_selection() {
        let mut table = StatefulPasswordTable::default();
        table.items.extend(more_table_entries(8));
        assert_eq!(table.items.len(), 11);
        table.leap(LeapDirection::MIDDLE);
        assert_eq!(table.state.selected(), Some(5));
    }

    #[test]
    fn test_encryption_encrypt() {
        let mut table = StatefulPasswordTable::default();
        table.state.select(Some(0));
        table.decrypt();
        assert_eq!(
            table.encryption(EncryptionMode::ENCRYPT, 0),
            String::from("Fw6SsG9VijR6gMioOvBRcaFzUkgbiGD/hw==")
        );
    }

    #[test]
    fn test_encryption_decrypt() {
        let table = StatefulPasswordTable::default();
        assert_eq!(
            table.encryption(EncryptionMode::DECRYPT, 0),
            String::from("test_pass")
        );
    }

    #[test]
    fn test_backwards_wraparound_zero_idx() {
        let mut table = StatefulPasswordTable::default();
        let current_idx = 0;
        table.state.select(Some(current_idx));
        assert_eq!(
            table.backwards_wraparound(current_idx),
            table.items.len() - 1
        );
    }

    #[test]
    fn test_backwards_wraparound_nonzero_idx() {
        let mut table = StatefulPasswordTable::default();
        let current_idx = 1;
        table.state.select(Some(current_idx));
        assert_eq!(table.backwards_wraparound(current_idx), 0);
    }

    #[test]
    fn test_copy_to_clipboard() {
        let mut table = StatefulPasswordTable::default();
        table.state.select(Some(0));
        table.copy();

        // Took me way too long to figure this out. Turns out making
        // system calls is very slow and we have to wait for the
        // copied password text to make it into `pbcopy` before we can
        // paste it out with `pbpaste`. Not noticeable when using the
        // program, thankfully.
        std::thread::sleep(std::time::Duration::from_millis(4));

        let result = if cfg!(target_os = "macos") {
            Command::new("pbpaste")
                .output()
                .expect("pbpaste command failed!")
                .stdout
        } else {
            Command::new("xclip")
                .arg("-select")
                .arg("clipboard")
                .arg("-o")
                .output()
                .expect("xclip command failed!")
                .stdout
        };

        assert_eq!(
            String::from_utf8(result).unwrap(),
            String::from("test_pass")
        );
    }

    #[test]
    fn test_new_username() {
        let mut table = StatefulPasswordTable::default();
        table.input.push_str("new_test_user");
        table.new_username();
        assert_eq!(table.new_username, "new_test_user");
        assert!(table.input.is_empty());
        assert_eq!(table.current_mode, CurrentMode::NewPassword);
    }

    #[test]
    fn test_new_password() {
        let mut table = StatefulPasswordTable::default();
        table.input.push_str("new_test_password");
        table.new_username.push_str("new_test_user");
        table.new_password();
        assert_eq!(table.current_mode, CurrentMode::PasswordCreated);
        assert!(table.input.is_empty());
        assert!(table.new_username.is_empty());
        assert!(table.new_password.is_empty());
        delete_password("new_test_user").unwrap();
    }

    fn more_table_entries(num: u8) -> Vec<TableEntry> {
        (0..num).map(|_| TableEntry::default()).collect()
    }
}
