use crate::util::banner::BANNER;
use crate::util::configs::CursesConfigs;
use crate::util::stateful_table::{CurrentMode, TableEntry};

use std::io::Stdout;

use lazy_static::lazy_static;

use termion::input::MouseTerminal;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, TableState, Wrap,
};
use tui::Frame;

pub type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;
pub type HelpList = Vec<ListItem<'static>>;

static NEW_USERNAME_TITLE: &str = "Enter a new username. Press Esc to cancel";
static NEW_PASSWORD_TITLE: &str = "Enter a new password. Press Esc to cancel";
static DELETE_PASSWORD: &str = "Enter username of password to delete. Press Esc to cancel";
static PASSWORD_CREATED: &str = "Password created! Press any key to close";
static PASSWORD_DELETED: &str = "Password deleted! Press any key to close";
static NO_SUCH_PASSWORD: &str = "No such password! Press any key to close";
static PASSWORD_EXISTS: &str = "Password already exists for this service! Press any key to close";
static BOX_WIDTH: u16 = 70;
static BOX_HEIGHT: u16 = 20;

static HELP_PROMPT_HEIGHT: u16 = 3;
static HELP_BOX_HEIGHT: u16 = 16;

// static ADD_DEL_PASSWORD_BOX_WIDTH: u16 = BOX_WIDTH;
// static ADD_DEL_PASSWORD_BOX_HEIGHT: u16 = 8;

static BANNER_LEN: u16 = 70;
static BANNER_HEIGHT: u16 = 10;

static BUTTONS: [&str; 14] = [
    "j/down", "k/up", "Ctrl-d", "Ctrl-u", "g", "G", "M", "y", "d", "r", "c", "D", "?", "q",
];
static EFFECTS: [&str; 14] = [
    "move down",
    "move up",
    "move down x5",
    "move up x5",
    "jump to top",
    "jump to bottom",
    "Jump to middle",
    "copy password",
    "decrypt the password",
    "refresh passwords",
    "create new password",
    "delete password",
    "hide/show help",
    "quit",
];

static HELP_MSG_SPACING: usize = 40;

lazy_static! {
    static ref HELP_MESSAGES: HelpList = {
        BUTTONS
            .iter()
            .zip(EFFECTS.iter())
            .map(|(b, e)| {
                let main_str = format!(
                    "{} {:.<spacing$} {}",
                    b,
                    ".",
                    e,
                    spacing = (HELP_MSG_SPACING - e.len()) - b.len()
                );
                ListItem::new(Text::styled(
                    format!("{:^69}", main_str),
                    Style::default().add_modifier(Modifier::ITALIC),
                ))
            })
            .collect::<HelpList>()
    };
}

/// Draws the main view including the password table and, optionally, the banner.
pub fn draw_table(
    table_state: &mut TableState,
    table_items: &[TableEntry],
    cfg: &CursesConfigs,
    f: &mut Frame<Backend>,
    table_decrypted: &bool,
    show_banner: Option<bool>,
) {
    let highlight_colour = if *table_decrypted {
        Color::Green
    } else {
        Color::Red
    };

    if show_banner.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(10),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(Rect {
                x: (f.size().width / 2) - BOX_WIDTH / 2,
                y: (f.size().height / 2) - BOX_HEIGHT + 2,
                width: BANNER_LEN,
                height: BANNER_HEIGHT,
            });

        let banner = Spans::from(vec![Span::raw(BANNER)]);
        let banner_box = Paragraph::new(banner)
            .block(Block::default().borders(Borders::NONE))
            .style(
                Style::default()
                    .fg(highlight_colour)
                    .add_modifier(Modifier::BOLD),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(banner_box, chunks[1]);
    }

    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .horizontal_margin(1)
        .split(Rect {
            x: 0,
            y: 0,
            width: f.size().width,
            height: f.size().height - 3,
        });

    let rows: Vec<_> = table_items.iter().map(|i| i.to_cells()).collect();

    let header_cells = ["Username", "Password"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(cfg.title_style)));

    let header = Row::new(header_cells).style(Style::default().fg(Color::Yellow));

    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title(Span::styled(
                    "Passwords",
                    Style::default().add_modifier(cfg.title_style),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().add_modifier(cfg.border_style)),
        )
        .highlight_style(Style::default().fg(Color::Black).bg(highlight_colour))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
        .style(Style::default().fg(Color::White))
        .column_spacing(1);

    f.render_stateful_widget(t, rects[0], table_state);

    let rects_2 = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .horizontal_margin(1)
        .split(Rect {
            x: 0,
            y: f.size().height - 3,
            width: f.size().width,
            height: HELP_PROMPT_HEIGHT,
        });

    let text = vec![Span::styled(
        "? for help",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(Spans::from(text))
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, rects_2[0]);
}

/// Draws the help window.
pub fn draw_help_window(f: &mut Frame<Backend>) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(Rect {
            x: (f.size().width / 2) - BOX_WIDTH / 2,
            y: ((f.size().height / 2) - BOX_HEIGHT / 2),
            width: BOX_WIDTH,
            height: HELP_BOX_HEIGHT,
        });

    let help = List::new(HELP_MESSAGES.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Help"));

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
        .split(f.size());
    // .split(Rect {
    //     x: (f.size().width / 2) - ADD_DEL_PASSWORD_BOX_WIDTH / 2,
    //     y: (f.size().height / 2) - (BOX_HEIGHT + 12) / 2,
    //     width: ADD_DEL_PASSWORD_BOX_WIDTH,
    //     height: ADD_DEL_PASSWORD_BOX_HEIGHT,
    // });

    let title = match current_mode {
        CurrentMode::NewUserName => NEW_USERNAME_TITLE,
        CurrentMode::NewPassword => NEW_PASSWORD_TITLE,
        CurrentMode::DeletePassword => DELETE_PASSWORD,
        CurrentMode::PasswordDeleted => PASSWORD_DELETED,
        CurrentMode::PasswordCreated => PASSWORD_CREATED,
        CurrentMode::NoSuchPassword => NO_SUCH_PASSWORD,
        CurrentMode::PasswordExists => PASSWORD_EXISTS,
        _ => "UNKNOWN MODE",
    };
    let text = Text::styled(table_input, Style::default());
    let input = Paragraph::new(text)
        .style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().add_modifier(Modifier::BOLD))
                .title(title)
                .style(Style::default()),
        );
    f.render_widget(Clear, chunks[1]); // Clears the background of the popup.
    f.render_widget(input, chunks[1]);
}
