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
use crate::protocol::{AskRequest, ShelfModule};

pub fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn run_picker(request: &AskRequest) -> Result<Option<String>> {
    if !is_tty() {
        return Err(PresentError::Bad(
            "present --interactive needs a terminal. pipe a tty or drop the flag".into(),
        ));
    }
    let result = run_picker_inner(request);
    restore_terminal();
    result
}

fn run_picker_inner(request: &AskRequest) -> Result<Option<String>> {
    let mut stdout = std::io::stdout();
    enable_raw_mode().map_err(|e| PresentError::Bad(format!("could not enter raw mode: {e}")))?;
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| PresentError::Bad(format!("could not enter alternate screen: {e}")))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| PresentError::Bad(format!("could not open terminal: {e}")))?;

    let mut state = ListState::default();
    state.select(Some(0));
    let mut cancelled = false;

    loop {
        terminal
            .draw(|frame| {
                let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)])
                    .split(frame.area());
                let header = Paragraph::new(vec![
                    Line::from(request.message.clone()),
                    Line::from(Span::styled(
                        "arrows move, enter confirms, esc cancels",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]);
                frame.render_widget(header, chunks[0]);

                let items: Vec<ListItem> = request
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| {
                        ListItem::new(Line::from(vec![
                            Span::raw(format!("  ({}) ", idx + 1)),
                            Span::raw(opt.as_str()),
                        ]))
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
            KeyCode::Enter => break,
            _ => {}
        }
    }

    if cancelled {
        return Ok(None);
    }
    let idx = state.selected().unwrap_or(0);
    match request.options.get(idx) {
        Some(opt) => Ok(Some(opt.clone())),
        None => Ok(None),
    }
}

pub fn run_shelf_browser(modules: &[ShelfModule]) -> Result<Option<String>> {
    if !is_tty() {
        return Err(PresentError::Bad(
            "present needs a terminal to browse the shelf. run it in a tty".into(),
        ));
    }
    let result = run_shelf_inner(modules);
    restore_terminal();
    result
}

fn run_shelf_inner(modules: &[ShelfModule]) -> Result<Option<String>> {
    if modules.is_empty() {
        return Err(PresentError::Bad(
            "the shelf is empty. run sheol share to add a module".into(),
        ));
    }

    let mut stdout = std::io::stdout();
    enable_raw_mode().map_err(|e| PresentError::Bad(format!("could not enter raw mode: {e}")))?;
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| PresentError::Bad(format!("could not enter alternate screen: {e}")))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| PresentError::Bad(format!("could not open terminal: {e}")))?;

    let mut query = String::new();
    let mut state = ListState::default();
    state.select(Some(0));
    let mut cancelled = false;

    loop {
        let filtered = filter_modules(modules, &query);
        if filtered.is_empty() {
            state.select(None);
        } else {
            let current = state.selected().unwrap_or(0);
            if current >= filtered.len() {
                state.select(Some(0));
            }
        }

        terminal
            .draw(|frame| {
                let chunks = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(frame.area());

                let filter = Paragraph::new(format!("filter: {query}"))
                    .block(Block::default().borders(Borders::ALL).title("sheol shelf"));
                frame.render_widget(filter, chunks[0]);

                let items: Vec<ListItem> = if filtered.is_empty() {
                    vec![ListItem::new(Line::from(Span::styled(
                        "no modules match your filter",
                        Style::default().fg(Color::DarkGray),
                    )))]
                } else {
                    filtered
                        .iter()
                        .map(|&idx| {
                            let m = &modules[idx];
                            let desc = if m.description.is_empty() {
                                String::new()
                            } else {
                                format!(" — {}", m.description)
                            };
                            ListItem::new(Line::from(vec![
                                Span::raw(m.name.clone()),
                                Span::styled(desc, Style::default().fg(Color::DarkGray)),
                            ]))
                        })
                        .collect()
                };

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("modules"))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    );
                frame.render_stateful_widget(list, chunks[1], &mut state);

                let help = if filtered.is_empty() {
                    "no match. backspace clears the filter"
                } else {
                    "enter picks · esc cancels · type to filter"
                };
                frame.render_widget(
                    Paragraph::new(Span::styled(help, Style::default().fg(Color::DarkGray))),
                    chunks[2],
                );
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
            KeyCode::Enter => {
                if filtered.is_empty() {
                    continue;
                }
                let sel = state.selected().unwrap_or(0);
                if let Some(&original_idx) = filtered.get(sel) {
                    return Ok(Some(modules[original_idx].id.clone()));
                }
                break;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if filtered.is_empty() {
                    continue;
                }
                let idx = state.selected().unwrap_or(0);
                let next = if idx + 1 >= filtered.len() {
                    0
                } else {
                    idx + 1
                };
                state.select(Some(next));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if filtered.is_empty() {
                    continue;
                }
                let idx = state.selected().unwrap_or(0);
                let prev = if idx == 0 {
                    filtered.len() - 1
                } else {
                    idx - 1
                };
                state.select(Some(prev));
            }
            KeyCode::Backspace => {
                query.pop();
                state.select(Some(0));
            }
            KeyCode::Char(c) => {
                if c.is_control() {
                    continue;
                }
                query.push(c);
                state.select(Some(0));
            }
            _ => {}
        }
    }

    if cancelled {
        return Ok(None);
    }
    Ok(None)
}

fn filter_modules(modules: &[ShelfModule], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..modules.len()).collect();
    }
    let lower = query.to_lowercase();
    modules
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            let matches = m.name.to_lowercase().contains(&lower)
                || m.description.to_lowercase().contains(&lower)
                || m.id.to_lowercase().contains(&lower)
                || m.tags.iter().any(|t| t.to_lowercase().contains(&lower));
            if matches {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
}
