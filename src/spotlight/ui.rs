//! The interactive terminal launcher behind `keylex --spotlight`. Built on
//! `ratatui` (crossterm backend), which is pure terminal I/O with no
//! OS-specific code of its own, so this behaves identically in a Linux,
//! macOS, or Windows terminal.

use std::io;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal};

use super::{Index, Match};
use crate::dispatch::Router;
use crate::focus;

const HELP: &str = "Type to filter, \u{2191}/\u{2193} to move, Enter to run, Esc to quit";

/// The terminal, restored to its normal mode on the way out however this
/// function returns -- including on an error or a panic mid-loop, which is
/// what would otherwise leave the user's terminal unusable.
struct Screen(Terminal<CrosstermBackend<io::Stdout>>);

impl Screen {
    fn enter() -> io::Result<Screen> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Screen(terminal))
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// Type to fuzzy-filter, Up/Down to move, Enter to dispatch the highlighted
/// action through `router` -- the same `Router::dispatch` the capture loop
/// uses -- against whichever app is focused, Esc/Ctrl-C to quit.
pub fn run_interactive(index: &mut Index, router: &Router) -> io::Result<()> {
    let mut screen = Screen::enter()?;
    let mut query = String::new();
    let mut selected = 0usize;
    let mut last_dispatch: Option<String> = None;

    loop {
        let matches = index.search(&query);
        selected = selected.min(matches.len().saturating_sub(1));
        screen
            .0
            .draw(|frame| render(frame, &query, &matches, selected, last_dispatch.as_deref()))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Esc => return Ok(()),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
            KeyCode::Enter => {
                let Some(entry) = matches.get(selected).map(|m| m.entry.clone()) else {
                    continue;
                };
                let outcome = entry.dispatch(focus::focused_process_name().as_deref(), router);
                index.record_use(&entry.action_id);
                last_dispatch = Some(format!("{} -> {outcome}", entry.action_id));
                query.clear();
                selected = 0;
            }
            KeyCode::Backspace => {
                query.pop();
                selected = 0;
            }
            KeyCode::Up => selected = selected.saturating_sub(1),
            KeyCode::Down => selected = selected.saturating_add(1),
            KeyCode::Char(c) => {
                query.push(c);
                selected = 0;
            }
            _ => {}
        }
    }
}

fn render(
    frame: &mut Frame,
    query: &str,
    matches: &[Match],
    selected: usize,
    last_dispatch: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let input = Paragraph::new(format!("> {query}")).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Keylex spotlight"),
    );
    frame.render_widget(input, chunks[0]);

    let items: Vec<ListItem> = matches
        .iter()
        .map(|m| {
            let key_hint = m
                .entry
                .key_hint
                .as_deref()
                .map_or_else(String::new, |k| format!(" ({k})"));
            let line = Line::from(vec![
                Span::raw(format!("{}{key_hint}  ", m.entry.title)),
                Span::styled(
                    format!("[{}]", m.entry.source),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list_title = match matches.len() {
        0 => "No matches".to_string(),
        1 => "1 match".to_string(),
        n => format!("{n} matches"),
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !matches.is_empty() {
        state.select(Some(selected));
    }
    frame.render_stateful_widget(list, chunks[1], &mut state);

    let footer =
        last_dispatch.map_or_else(|| HELP.to_string(), |message| format!("last: {message}"));
    frame.render_widget(Paragraph::new(footer), chunks[2]);
}
