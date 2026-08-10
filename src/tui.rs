use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

use crate::error::{PresentError, Result};
use crate::protocol::AskRequest;

pub fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn run(request: &AskRequest) -> Result<Option<Vec<String>>> {
    if !is_tty() {
        return Err(PresentError::Bad(
            "present --interactive needs a terminal. pipe a tty or drop the flag".into(),
        ));
    }
    let result = run_inner(request);
    restore_terminal();
    result
}

fn run_inner(request: &AskRequest) -> Result<Option<Vec<String>>> {
    let mut stdout = std::io::stdout();
    enable_raw_mode().map_err(|e| PresentError::Bad(format!("could not enter raw mode: {e}")))?;
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| PresentError::Bad(format!("could not enter alternate screen: {e}")))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| PresentError::Bad(format!("could not open terminal: {e}")))?;

    let mut state = ListState::default();
    state.select(Some(0));
    let mut chosen: Vec<usize> = Vec::new();
    let mut cancelled = false;

    loop {
        terminal
            .draw(|frame| {
                let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)])
                    .split(frame.area());
                let header = Paragraph::new(vec![
                    Line::from(request.message.clone()),
                    if request.multiple {
                        Line::from(Span::styled(
                            "space toggles, enter confirms, esc cancels",
                            Style::default().fg(Color::DarkGray),
                        ))
                    } else {
                        Line::from(Span::styled(
                            "arrows move, enter confirms, esc cancels",
                            Style::default().fg(Color::DarkGray),
                        ))
                    },
                ]);
                frame.render_widget(header, chunks[0]);

                let items: Vec<ListItem> = request
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| {
                        let mark = if chosen.contains(&idx) {
                            "[x] "
                        } else {
                            "[ ] "
                        };
                        ListItem::new(Line::from(vec![Span::raw(mark), Span::raw(opt.as_str())]))
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("pick"))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                frame.render_stateful_widget(list, chunks[1], &mut state);
            })
            .map_err(|e| PresentError::Bad(format!("draw failed: {e}")))?;

        if !event::poll(std::time::Duration::from_millis(250))
            .map_err(|e| PresentError::Bad(format!("event poll failed: {e}")))?
        {
            continue;
        }

        let Event::Key(key) =
            event::read().map_err(|e| PresentError::Bad(format!("event read failed: {e}")))?
        else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Esc => {
                cancelled = true;
                break;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let idx = state.selected().unwrap_or(0);
                let next = if idx + 1 >= request.options.len() {
                    0
                } else {
                    idx + 1
                };
                state.select(Some(next));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let idx = state.selected().unwrap_or(0);
                let prev = if idx == 0 {
                    request.options.len() - 1
                } else {
                    idx - 1
                };
                state.select(Some(prev));
            }
            KeyCode::Char(' ') if request.multiple => {
                let idx = state.selected().unwrap_or(0);
                if let Some(pos) = chosen.iter().position(|x| *x == idx) {
                    chosen.remove(pos);
                } else {
                    chosen.push(idx);
                }
            }
            KeyCode::Enter => break,
            _ => {}
        }
    }

    if cancelled {
        return Ok(None);
    }
    if request.multiple {
        if chosen.is_empty() {
            return Ok(None);
        }
        let picked: Vec<String> = chosen
            .iter()
            .filter_map(|i| request.options.get(*i).cloned())
            .collect();
        return Ok(Some(picked));
    }
    let idx = state.selected().unwrap_or(0);
    match request.options.get(idx) {
        Some(opt) => Ok(Some(vec![opt.clone()])),
        None => Ok(None),
    }
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
}
