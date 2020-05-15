use crate::util::inputs::InputMode;
use crate::util::json_utils::read_passwords;
use crate::util::ui::RenderMode;
use crate::util::utils::{build_table_rows, copy_to_clipboard, decrypt_value};
use std::error::Error;
use tui::widgets::TableState;

pub struct StatefulPasswordTable {
  pub(crate) state: TableState,
  pub(crate) items: Vec<Vec<String>>,
  pub(crate) decrypted: bool,
  pub(crate) key: u8,
  pub(crate) input: String,
  pub(crate) input_mode: InputMode,
  pub(crate) render_mode: RenderMode,
  pub(crate) active: bool,
}

impl StatefulPasswordTable {
  pub(crate) fn new(key: u8) -> StatefulPasswordTable {
    StatefulPasswordTable {
      state: TableState::default(),
      items: Vec::new(),
      decrypted: false,
      key,
      input: String::new(),
      input_mode: InputMode::Normal,
      render_mode: RenderMode::Normal,
      active: true,
    }
  }
  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if self.decrypted {
          self.items[i][1] = decrypt_value(&self.items[i][1], self.key);
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
          self.items[i][1] = decrypt_value(&self.items[i][1], self.key);
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
        self.items[i][1] = decrypt_value(&self.items[i][1], self.key);
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
      } else {
        if let Err(error) = copy_to_clipboard(&decrypt_value(&self.items[i][1], self.key)) {
          panic!("Error copying to clipboard: {}", error);
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
