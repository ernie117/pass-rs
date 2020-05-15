use std::error::Error;

use tui::Terminal;

use crate::stateful_table::StatefulPasswordTable;
use crate::util::event::{Event, Events};
use crate::util::inputs;
use crate::util::json_utils::{read_config, read_passwords};
use crate::util::ui;
use crate::util::ui::{Backend, RenderMode};
use crate::util::utils::build_table_rows;

pub fn render_password_table(
  terminal: &mut Terminal<Backend>,
  key: u8,
) -> Result<(), Box<dyn Error>> {
  let events = Events::new();
  let mut table = StatefulPasswordTable::new(key);
  let mut pwd_input: Vec<String> = Vec::new();
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
      match table.render_mode {
        RenderMode::Normal => {
          ui::draw_table(
            &mut table.state,
            &table.items,
            &cfg,
            &mut f,
            &table.decrypted,
          );
        }
        RenderMode::WithHelp => {
          ui::draw_table(
            &mut table.state,
            &table.items,
            &cfg,
            &mut f,
            &table.decrypted,
          );
          ui::draw_help_window(&mut f);
        }
        RenderMode::NewPassword => {
          ui::draw_add_password(&mut f, &table.input_mode, &table.input);
        }
      };
    })?;

    match table.render_mode {
      RenderMode::Normal | RenderMode::WithHelp => match events.next()? {
        Event::Input(key) => {
          inputs::password_table_input_handler(&mut table, key);
        }
        _ => {}
      },
      RenderMode::NewPassword => match events.next()? {
        Event::Input(key) => {
          inputs::add_password_input_handler(&mut table, key, &mut pwd_input);
        }
        _ => {}
      },
    }
  }

  Ok(())
}
