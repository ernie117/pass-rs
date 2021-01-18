use crate::util::banner::BANNER;
use crate::util::configs::CursesConfigs;
use crate::util::stateful_table::CurrentMode;
use crate::util::utils::{build_help_messages, TableEntry};
use std::io::Stdout;
use termion::input::MouseTerminal;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, Row, Table, TableState, Text};
use tui::Frame;

pub type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;

static NEW_USERNAME_TITLE: &str = "Enter a new username. Press Esc to cancel";
static NEW_PASSWORD_TITLE: &str = "Enter a new password. Press Esc to cancel";
static PASSWORD_CREATED: &str = "Password created! Press Esc to close";
static DELETE_PASSWORD: &str = "Enter username of password to delete. Press Esc to cancel";
static PASSWORD_DELETED: &str = "Password deleted! Press Esc to close";
static NO_SUCH_PASSWORD: &str = "No such password! Press Esc to close";
static BOX_WIDTH: u16 = 70;
static BOX_HEIGHT: u16 = 20;

static HELP_PROMPT_HEIGHT: u16 = 3;
static HELP_BOX_HEIGHT: u16 = 10;

static ADD_DEL_PASSWORD_BOX_WIDTH: u16 = BOX_WIDTH + 10;
static ADD_DEL_PASSWORD_BOX_HEIGHT: u16 = 8;

static BANNER_LEN: u16 = 70;
static BANNER_HEIGHT: u16 = 12;

/// Draws the main view including the password table and banner.
pub fn draw_table(
    table_state: &mut TableState,
    table_items: &[TableEntry],
    cfg: &CursesConfigs,
    f: &mut Frame<Backend>,
    table_decrypted: &bool,
) {
    let highlight_colour = if *table_decrypted {
        Color::Green
    } else {
        Color::Red
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(12),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(Rect {
            x: (f.size().width / 2) - BOX_WIDTH / 2,
            y: (f.size().height / 2) - BOX_HEIGHT - 3,
            width: BANNER_LEN,
            height: BANNER_HEIGHT,
        });

    let banner = [Text::styled(
        BANNER,
        Style::default()
            .fg(highlight_colour)
            .modifier(Modifier::BOLD),
    )];
    let banner_box = Paragraph::new(banner.iter()).block(Block::default().borders(Borders::NONE));
    f.render_widget(banner_box, chunks[1]);

    let row_style = Style::default().fg(Color::White);
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(Rect {
            x: (f.size().width / 2) - BOX_WIDTH / 2,
            y: (f.size().height / 2) - BOX_HEIGHT / 2,
            width: BOX_WIDTH,
            height: BOX_HEIGHT,
        });

    let vec_entries: Vec<Vec<_>> = table_items
        .iter()
        .map(|e| vec![&e.service, &e.password, &e.nonce])
        .collect();

    let rows = vec_entries
        .iter()
        .map(|i| Row::StyledData(i.iter(), row_style));

    let t = Table::new(["Username", "Password"].iter(), rows)
        .block(
            Block::default()
                .title("Passwords")
                .title_style(Style::default().modifier(cfg.title_style))
                .borders(Borders::ALL)
                .border_type(cfg.border_type)
                .border_style(Style::default().modifier(cfg.border_style)),
        )
        .header_style(Style::default().fg(Color::Yellow))
        .highlight_style(Style::default().fg(Color::Black).bg(highlight_colour))
        .widths(&[Constraint::Length(35), Constraint::Length(35)])
        .style(Style::default().fg(Color::White))
        .column_spacing(1);

    f.render_stateful_widget(t, rects[0], table_state);

    let rects_2 = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(Rect {
            x: (f.size().width / 2) - BOX_WIDTH / 2,
            y: ((f.size().height / 2) - BOX_HEIGHT / 2) + BOX_HEIGHT,
            width: BOX_WIDTH,
            height: HELP_PROMPT_HEIGHT,
        });

    let text = [Text::raw("? for help")];
    let block = Block::default().borders(Borders::NONE);
    let paragraph = Paragraph::new(text.iter())
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, rects_2[0]);
}

/// Draws the help window.
pub fn draw_help_window(f: &mut Frame<Backend>) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(Rect {
            x: (f.size().width / 2) - BOX_WIDTH / 2,
            y: ((f.size().height / 2) - BOX_HEIGHT / 2) + BOX_HEIGHT,
            width: BOX_WIDTH,
            height: HELP_BOX_HEIGHT,
        });

    let messages = build_help_messages();
    let help =
        List::new(messages.into_iter()).block(Block::default().borders(Borders::ALL).title("Help"));

    f.render_widget(help, rects[0]);
}
/// Draws the input box for adding/deleting a new password.
pub fn draw_add_delete_password(
    f: &mut Frame<Backend>,
    current_mode: &CurrentMode,
    table_input: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(Rect {
            x: (f.size().width / 2) - ADD_DEL_PASSWORD_BOX_WIDTH / 2,
            y: (f.size().height / 2) - (BOX_HEIGHT + 12) / 2,
            width: ADD_DEL_PASSWORD_BOX_WIDTH,
            height: ADD_DEL_PASSWORD_BOX_HEIGHT,
        });

    let title = match current_mode {
        CurrentMode::NewUserName => NEW_USERNAME_TITLE,
        CurrentMode::NewPassword => NEW_PASSWORD_TITLE,
        CurrentMode::DeletePassword => DELETE_PASSWORD,
        CurrentMode::PasswordDeleted => PASSWORD_DELETED,
        CurrentMode::PasswordCreated => PASSWORD_CREATED,
        CurrentMode::NoSuchPassword => NO_SUCH_PASSWORD,
        _ => "Unknown mode",
    };
    let text = [Text::styled(
        table_input,
        Style::default().fg(Color::Black).bg(Color::White),
    )];
    let input = Paragraph::new(text.iter())
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        );
    f.render_widget(input, chunks[1]);
}
