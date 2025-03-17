use arboard::Clipboard;
use chrono::{DateTime, Local, TimeZone};
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self};
use std::io::{self, stdout};
use std::path::PathBuf;

use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::text::{Line, Span};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClipboardHistoryEntry {
    pub content: String,
    pub timestamp: String,
}

pub fn save_clipboard_history(content: String) {
    // Try to get home directory, fallback to current directory
    // Get the home directory from the HOME env var (works on Linux/macOS)
    let history_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| {
            println!("Warning: HOME not set. Using current directory.");
            PathBuf::from(".zp")
        });

    // Create the directory if it doesn't exist
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir).expect("Failed to create .zp directory");
    }
    let history_file = history_dir.join("clipboard_history.json");
    let timestamp = Local::now().to_rfc3339();

    let entry = ClipboardHistoryEntry { content, timestamp };

    // Load existing history
    let mut history = if let Ok(content) = fs::read_to_string(&history_file) {
        serde_json::from_str::<Vec<ClipboardHistoryEntry>>(&content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    // Add new entry
    history.push(entry);

    // Serialize and write updated history
    let serialized_history =
        serde_json::to_string_pretty(&history).expect("Failed to serialize clipboard history");
    fs::write(&history_file, serialized_history).expect("Failed to write clipboard history");
}

pub fn load_clipboard_history() -> Result<Vec<ClipboardHistoryEntry>, io::Error> {
    let history_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| {
            println!("Warning: HOME not set. Using current directory.");
            PathBuf::from(".zp")
        });

    let history_file = history_dir.join("clipboard_history.json");

    // Ensure the history directory exists
    if fs::metadata(history_dir).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "History directory not found",
        ));
    }

    // Read the contents of the history file
    let content = fs::read_to_string(&history_file)?;

    // Deserialize the JSON content into a vector of ClipboardHistoryEntry
    if content.trim() == "[]" {
        return Ok(vec![]);
    }
    // Ensure the JSON is valid and not empty
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() || !trimmed_content.starts_with('[') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid or empty clipboard history file",
        ));
    }

    Ok(serde_json::from_str(trimmed_content)?)
}

pub fn print_clipboard_history() -> Result<(), io::Error> {
    let entries = load_clipboard_history().map_err(|e| {
        eprintln!("Failed to load clipboard history: {}", e);
        e
    })?;

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut state = ListState::default();
    if !entries.is_empty() {
        state.select(Some(0));
    }

    let result = run_app(&mut terminal, &entries, &mut state);

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    result
}

fn format_elapsed_time(timestamp: &str) -> String {
    let entry_time = DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Local))
        .unwrap_or_else(|_| Local.timestamp_opt(0, 0).unwrap());
    let now = Local::now();
    let duration = now.signed_duration_since(entry_time);

    let formatted = if duration.num_seconds() < 60 {
        format!("{}s ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else {
        format!("{}d ago", duration.num_days())
    };

    format!("{:>7}", formatted) // Right-align with 7 characters
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    entries: &[ClipboardHistoryEntry],
    state: &mut ListState,
) -> io::Result<()> {
    let mut clipboard = Clipboard::new().unwrap();
    state.select(Some(entries.len().saturating_sub(1))); // Start from the bottom

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let app_height = size.height / 2;
            let app_area = Rect {
                y: size.height - app_height, // Push to the bottom
                width: size.width,
                height: app_height,
                ..size
            };

            // Calculate max visible items in the half-screen area
            let max_visible_items = app_height as usize - 2; // Subtract for borders

            // Ensure the entries stick to the bottom
            let visible_entries = if entries.len() <= max_visible_items {
                entries
            } else {
                &entries[entries.len() - max_visible_items..]
            };

            // Calculate empty rows at the top to push content to the bottom
            let empty_rows = max_visible_items.saturating_sub(visible_entries.len());
            let mut items: Vec<ListItem> = (0..empty_rows).map(|_| ListItem::new("")).collect();

            items.extend(visible_entries.iter().enumerate().map(|(i, entry)| {
                let elapsed = format_elapsed_time(&entry.timestamp);
                let elapsed_styled = Span::styled(elapsed, Style::default().fg(Color::Green));
                let content_styled =
                    Span::styled(entry.content.clone(), Style::default().fg(Color::White));

                let actual_index = entries.len() - visible_entries.len() + i;

                // Add the highlight symbol only when selected
                let highlight_symbol = if state.selected() == Some(actual_index) {
                    Span::styled(
                        "> ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::raw("  ") // Maintain alignment when not selected
                };

                let line = Line::from(vec![
                    highlight_symbol,
                    elapsed_styled,
                    Span::raw(" "),
                    content_styled,
                ]);

                if state.selected() == Some(actual_index) {
                    ListItem::new(line).style(Style::default().bg(Color::DarkGray))
                } else {
                    ListItem::new(line)
                }
            }));

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" zp ")
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
            );

            f.render_stateful_widget(list, app_area, state);
        })?;

        if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    // Navigate up to older entries
                    if let Some(selected) = state.selected() {
                        if selected > 0 {
                            state.select(Some(selected - 1));
                        }
                    }
                }
                KeyCode::Down => {
                    // Navigate down to newer entries
                    if let Some(selected) = state.selected() {
                        if selected < entries.len() - 1 {
                            state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = state.selected() {
                        let content = &entries[selected].content;
                        clipboard.set_text(content.to_owned()).unwrap();
                        println!("Copied: {}", content);
                        break;
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    Ok(())
}
