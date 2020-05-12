use crate::util::json_utils::CursesConfigs;
use std::io::Stdout;
use termion::input::MouseTerminal;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, Row, Table, TableState, Text};
use tui::Frame;

pub type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;

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
      x: (f.size().width / 2) - 35,
      y: (f.size().height / 2) - 12,
      width: 70,
      height: 24,
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

  f.render_stateful_widget(t, rects[0], table_state);
}

pub fn draw_help_window(f: &mut Frame<Backend>) {
  let rects = Layout::default()
    .constraints([Constraint::Percentage(100)].as_ref())
    .split(Rect {
      x: (f.size().width / 2) - 35,
      y: f.size().height - 12,
      width: 70,
      height: 8,
    });

  let zipped_iterator = ["j/down", "k/up", "y", "d", "r", "c"].iter().zip(
    [
      "move down",
      "move up",
      "copy the password",
      "decrypt password",
      "refresh passwords",
      "create a new password",
    ]
    .iter(),
  );

  let mut messages = Vec::new();
  for (button, effect) in zipped_iterator {
    let spaced_btn = format!("{} ", button);
    let main_str = format!(" {:.<40} {}", spaced_btn, effect);
    messages.push(Text::raw(format!("  {:<70}", main_str)));
  }

  let help =
    List::new(messages.into_iter()).block(Block::default().borders(Borders::ALL).title("Help"));

  f.render_widget(help, rects[0]);
}