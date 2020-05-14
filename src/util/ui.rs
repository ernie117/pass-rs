use crate::util::json_utils::CursesConfigs;
use std::io::Stdout;
use termion::input::MouseTerminal;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, Paragraph, Row, Table, TableState, Text};
use tui::Frame;

pub type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;

pub enum InputMode {
  Normal,
  Insert,
}

pub enum RenderMode {
  Normal,
  WithHelp,
  NewPassword,
}

static BUTTONS: [&str; 6] = ["j/down", "k/up", "y", "d", "r", "c"];
static EFFECTS: [&str; 6] = [
  "move down",
  "move up",
  "copy password",
  "decrypt the password",
  "refresh passwords",
  "create new password (NOT WORKING)",
];

static BOX_WIDTH: u16 = 70;
static BOX_HEIGHT: u16 = 24;

static NORMAL_MODE_TITLE: &str = "Press 'q' to close input, press 'i' to enter service/password";
static INSERT_MODE_TITLE: &str = "Type service and password separated by ':'";

static HELP_BOX_HEIGHT: u16 = EFFECTS.len() as u16 + 2;
static HELP_MSG_SPACING: usize = 40;

pub fn draw_table(
  table_state: &mut TableState,
  table_items: &Vec<Vec<String>>,
  cfg: CursesConfigs,
  f: &mut Frame<Backend>,
  highlight_colour: Color,
) {
  let row_style = Style::default().fg(Color::White);
  let rects = Layout::default()
    .constraints([Constraint::Percentage(100)].as_ref())
    .split(Rect {
      x: (f.size().width / 2) - BOX_WIDTH / 2,
      y: (f.size().height / 2) - BOX_HEIGHT / 2,
      width: BOX_WIDTH,
      height: BOX_HEIGHT,
    });

  let rows = table_items
    .iter()
    .map(|i| Row::StyledData(i.into_iter(), row_style));

  let t = Table::new(["Service", "Password"].iter(), rows)
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

  let rects_2 = Layout::default()
    .constraints([Constraint::Percentage(100)].as_ref())
    .split(Rect {
      x: (f.size().width / 2) - BOX_WIDTH / 2,
      y: ((f.size().height / 2) - BOX_HEIGHT / 2) + BOX_HEIGHT,
      width: BOX_WIDTH,
      height: 3,
    });

  let text = [Text::raw("? for help")];
  let block = Block::default()
      .borders(Borders::NONE);
  let paragraph = Paragraph::new(text.iter())
      .block(block)
      .alignment(Alignment::Left);

  f.render_stateful_widget(t, rects[0], table_state);
  f.render_widget(paragraph, rects_2[0]);
}

pub fn draw_help_window(f: &mut Frame<Backend>) {
  let rects = Layout::default()
    .constraints([Constraint::Percentage(100)].as_ref())
    .split(Rect {
      x: (f.size().width / 2) - BOX_WIDTH / 2,
      y: ((f.size().height / 2) - BOX_HEIGHT / 2) + BOX_HEIGHT,
      width: BOX_WIDTH,
      height: HELP_BOX_HEIGHT,
    });

  let zipped_help = BUTTONS.iter().zip(EFFECTS.iter());

  let mut messages = Vec::new();
  for (button, effect) in zipped_help {
    let spacing = (HELP_MSG_SPACING - effect.len()) - button.len();
    let main_str = format!(
      "{} {:.<spacing$} {}",
      button,
      ".",
      effect,
      spacing = spacing as usize
    );
    messages.push(Text::raw(format!("{:^69}", main_str)));
  }

  let help =
    List::new(messages.into_iter()).block(Block::default().borders(Borders::ALL).title("Help"));

  f.render_widget(help, rects[0]);
}

pub fn draw_add_password(
  f: &mut Frame<Backend>,
  table_input_mode: &InputMode,
  table_input: &String,
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

  let title = match table_input_mode {
    InputMode::Normal => NORMAL_MODE_TITLE,
    InputMode::Insert => INSERT_MODE_TITLE,
  };
  let text = [Text::raw(table_input)];
  let input = Paragraph::new(text.iter())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title(title));
  f.render_widget(input, chunks[1]);
}
