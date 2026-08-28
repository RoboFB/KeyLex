//! The interactive terminal launcher behind `keylex --spotlight`. Built on
//! `crossterm`, which is pure terminal I/O with no OS-specific code of its
//! own, so this behaves identically in a Linux, macOS, or Windows terminal.

use std::io::{self, Write as _};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue};

use super::{Index, Match};
use crate::dispatch::Router;
use crate::focus;

const MAX_VISIBLE_MATCHES: usize = 12;

/// The alternate screen and raw mode, restored on the way out however this
/// function returns -- including on an error or a panic mid-loop, which is
/// what would otherwise leave the user's terminal unusable.
struct Screen(io::Stdout);

impl Screen {
    fn enter() -> io::Result<Screen> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Screen(stdout))
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(self.0, Show, LeaveAlternateScreen);
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
        render(
            &mut screen.0,
            &query,
            &matches,
            selected,
            last_dispatch.as_deref(),
        )?;

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
    stdout: &mut io::Stdout,
    query: &str,
    matches: &[Match],
    selected: usize,
    last_dispatch: Option<&str>,
) -> io::Result<()> {
    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print("Keylex spotlight -- type to search, Enter to run, Esc to quit\r\n"),
        Print(format!("> {query}\r\n\r\n"))
    )?;

    if matches.is_empty() {
        queue!(stdout, Print("  (no matches)\r\n"))?;
    }
    for (i, m) in matches.iter().take(MAX_VISIBLE_MATCHES).enumerate() {
        let marker = if i == selected { ">" } else { " " };
        let key_hint = m
            .entry
            .key_hint
            .as_deref()
            .map_or_else(String::new, |k| format!(" ({k})"));
        queue!(
            stdout,
            Print(format!(
                "{marker} {}{key_hint}  [{}]\r\n",
                m.entry.title, m.entry.source
            ))
        )?;
    }

    if let Some(message) = last_dispatch {
        queue!(stdout, Print(format!("\r\nlast: {message}\r\n")))?;
    }

    stdout.flush()
}
