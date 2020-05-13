use std::error::Error;

use tui::style::Color;
use tui::Terminal;

use crate::stateful_table::StatefulPasswordTable;
use crate::util::event::{Event, Events};
use crate::util::inputs;
use crate::util::json_utils::{read_config, read_passwords};
use crate::util::ui;
use crate::util::ui::{Backend, RenderMode};
use crate::util::utils::build_table_rows;

pub fn render_password_table(
  mut terminal: Terminal<Backend>,
  key: u8,
) -> Result<(), Box<dyn Error>> {

  let mut events = Events::new();
  let mut table = StatefulPasswordTable::new(key);
  let mut pwd_input: Vec<String> = Vec::new();
  table.items = build_table_rows(read_passwords()?)?;

  loop {
    // Re-reading the config in the loop allows for live editing of colours/style/etc.
    let cfg = read_config()?;

    let highlight_colour = if table.decrypted {
      Color::Green
    } else {
      Color::Red
    };

    terminal.draw(|mut f| {
      match table.render_mode {
        RenderMode::Normal => {
          ui::draw_table(
            &mut table.state,
            &table.items,
            cfg,
            &mut f,
            highlight_colour,
          );
        }
        RenderMode::WithHelp => {
          ui::draw_table(
            &mut table.state,
            &table.items,
            cfg,
            &mut f,
            highlight_colour,
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
          inputs::add_password_input_handler(&mut table, key, &mut pwd_input, &mut events);
        }
        _ => {}
      },
    }

    if !table.active {
      break;
    }
  }

  Ok(())
}
