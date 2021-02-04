use std::error::Error;

use tui::Terminal;

use crate::util::event::{Event, Events};
use crate::util::inputs;
use crate::util::json_utils::{read_config, read_passwords};
use crate::util::stateful_table::{CurrentMode, StatefulPasswordTable};
use crate::util::ui::{self, Backend};
use crate::util::utils::build_table_rows;
use aes_gcm::Aes128Gcm;

pub fn run(terminal: &mut Terminal<Backend>, key: Aes128Gcm) -> Result<(), Box<dyn Error>> {
    let events = Events::new();
    let mut table = StatefulPasswordTable::new(key);
    table.items = build_table_rows(read_passwords()?);

    loop {
        // Reading the config in the loop allows for live editing of colours/style/etc.
        let cfg = match read_config() {
            Ok(c) => c,
            Err(e) => {
                terminal.show_cursor()?;
                panic!("Unable to read config file: {}", e);
            }
        };

        terminal.draw(|mut f| {
            match table.current_mode {
                CurrentMode::Normal => {
                    ui::draw_table(
                        &mut table.state,
                        &table.items,
                        &cfg,
                        &mut f,
                        &table.decrypted,
                        None,
                    );
                }
                CurrentMode::WithHelp => {
                    ui::draw_help_window(&mut f);
                }
                CurrentMode::NewPassword
                | CurrentMode::NewUserName
                | CurrentMode::PasswordCreated
                | CurrentMode::DeletePassword
                | CurrentMode::PasswordDeleted
                | CurrentMode::NoSuchPassword => {
                    ui::draw_table(
                        &mut table.state,
                        &table.items,
                        &cfg,
                        &mut f,
                        &table.decrypted,
                        None,
                    );
                    ui::draw_add_delete_password(&mut f, &table.current_mode, &table.input);
                }
            };
        })?;

        match table.current_mode {
            CurrentMode::Normal => {
                if let Event::Input(key) = events.next()? {
                    inputs::password_table_input_handler(&mut table, key);
                }
            }
            CurrentMode::WithHelp => {
                if let Event::Input(key) = events.next()? {
                    inputs::with_help_input_handler(&mut table, key);
                }
            }
            #[rustfmt::skip]
            CurrentMode::NewUserName
            | CurrentMode::NewPassword
            | CurrentMode::PasswordCreated => {
                if let Event::Input(key) = events.next()? {
                    inputs::add_password_input_handler(&mut table, key)?;
                }
            },
            CurrentMode::DeletePassword
            | CurrentMode::PasswordDeleted
            | CurrentMode::NoSuchPassword => {
                if let Event::Input(key) = events.next()? {
                    inputs::delete_password_input_handler(&mut table, key);
                }
            }
        }

        if !table.active {
            break;
        }
    }

    Ok(())
}
