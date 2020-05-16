use std::error::Error;

use tui::Terminal;

use crate::stateful_table::StatefulPasswordTable;
use crate::util::event::{Event, Events};
use crate::util::inputs;
use crate::util::json_utils::write_new_password;
use crate::util::json_utils::{read_config, read_passwords};
use crate::util::ui;
use crate::util::ui::{Backend, RenderMode};
use crate::util::utils::build_table_rows;
use termion::event::Key;

pub fn render_password_table(
  terminal: &mut Terminal<Backend>,
  key: u8,
) -> Result<(), Box<dyn Error>> {
  let events = Events::new();
  let mut table = StatefulPasswordTable::new(key);
  table.items = build_table_rows(read_passwords()?);

  while table.active {
    // Reading the config in the loop allows for live editing of colours/style/etc.
    let cfg = match read_config() {
      Ok(c) => c,
      Err(e) => {
        terminal.show_cursor()?;
        panic!("Unable to read config file: {}", e);
      }
    };

    terminal.draw(|mut f| {
      ui::draw_table(
        &mut table.state,
        &table.items,
        &cfg,
        &mut f,
        &table.decrypted,
      );
      match table.render_mode {
        RenderMode::WithHelp => {
          ui::draw_help_window(&mut f);
        }
        RenderMode::NewPassword | RenderMode::NewUserName | RenderMode::PasswordCreated => {
          ui::draw_add_password(&mut f, &table.render_mode, &table.input);
        }
        RenderMode::DeletePassword | RenderMode::PasswordDeleted | RenderMode::NoSuchPassword => {
          ui::draw_delete_password(&mut f, &table.render_mode, &table.input);
        }
        _ => {}
      };
    })?;

    match table.render_mode {
      RenderMode::Normal | RenderMode::WithHelp => match events.next()? {
        Event::Input(key) => {
          inputs::password_table_input_handler(&mut table, key);
        }
        _ => {}
      },
      RenderMode::NewUserName | RenderMode::NewPassword | RenderMode::PasswordCreated => {
        match events.next()? {
          Event::Input(key) => {
            inputs::add_password_input_handler(&mut table, key);
          }
          _ => {}
        }
      }
      RenderMode::DeletePassword | RenderMode::PasswordDeleted => match events.next()? {
        Event::Input(key) => {
          inputs::delete_password_input_handler(&mut table, key);
        }
        _ => {}
      },
      _ => {}
    }

    if !table.new_username.is_empty() && !table.new_password.is_empty() {
      write_new_password(&table.new_username, &table.new_password, table.key)?;
      table.new_username.clear();
      table.new_password.clear();
      table.re_encrypt()?;
    }
  }

  Ok(())
}
