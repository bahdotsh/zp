use crate::p2p::P2PNode;
use arboard::Clipboard;
use chrono::{DateTime, Local, TimeZone};
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self};
use std::io::{self, stdout};
use std::path::PathBuf;
use std::thread;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardHistoryEntry {
    pub content: String,
    pub timestamp: String,
}

pub fn merge_clipboard_entry(entry: ClipboardHistoryEntry) {
    // Load existing history
    let history_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| PathBuf::from(".zp"));

    let history_file = history_dir.join("clipboard_history.json");

    // Create directory if needed
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir).expect("Failed to create .zp directory");
    }

    // Load existing history
    let mut history = if let Ok(content) = fs::read_to_string(&history_file) {
        serde_json::from_str::<Vec<ClipboardHistoryEntry>>(&content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    // Check if the entry already exists by comparing content and timestamp
    let already_exists = history
        .iter()
        .any(|existing| existing.content == entry.content && existing.timestamp == entry.timestamp);

    if !already_exists {
        // Add the new entry
        history.push(entry);

        // Sort the history by timestamp
        history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Limit history size if needed
        if history.len() > 100 {
            // Keep only most recent 100 entries
            history.drain(0..history.len() - 100);
        }

        // Save updated history
        let serialized_history =
            serde_json::to_string_pretty(&history).expect("Failed to serialize clipboard history");
        fs::write(&history_file, serialized_history).expect("Failed to write clipboard history");
    }
}

pub fn merge_clipboard_history(entries: Vec<ClipboardHistoryEntry>) {
    // Process each entry individually to avoid duplicates
    for entry in entries {
        merge_clipboard_entry(entry);
    }
}

pub fn save_clipboard_history(content: String) {
    // Try to get home directory, fallback to current directory
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
    history.push(entry.clone());

    // Serialize and write updated history
    let serialized_history =
        serde_json::to_string_pretty(&history).expect("Failed to serialize clipboard history");
    fs::write(&history_file, serialized_history).expect("Failed to write clipboard history");

    // Check if P2P sync is enabled and notify peers
    let p2p_enabled = env::var("ZP_P2P_SYNC")
        .map(|val| val == "1" || val.to_lowercase() == "true")
        .unwrap_or(false);

    if p2p_enabled {
        // Use a non-blocking approach to send the entry to peers
        let entry_clone = entry.clone();
        thread::spawn(move || {
            if let Ok(p2p_node) = P2PNode::new() {
                let _ = futures::executor::block_on(p2p_node.send_new_entry(entry_clone));
            }
        });
    }
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
    let result = run_app(&mut terminal, &entries);

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
) -> io::Result<()> {
    let mut clipboard = Clipboard::new().unwrap();
    let mut selected = entries.len().saturating_sub(1); // Start at the bottom
    let mut offset = 0; // Offset to manage scrolling

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let app_height = size.height / 2;
            let max_visible_items = app_height.saturating_sub(2) as usize; // Account for borders

            // Ensure offset keeps the selected item in view
            if selected >= offset + max_visible_items {
                offset = selected.saturating_sub(max_visible_items).saturating_add(1);
            } else if selected < offset {
                offset = selected;
            }

            // Calculate the visible entries
            let visible_entries = &entries[offset..entries.len().min(offset + max_visible_items)];

            // Render list items
            let items: Vec<ListItem> = visible_entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let actual_index = offset + i;
                    let elapsed = format_elapsed_time(&entry.timestamp);
                    let elapsed_styled = Span::styled(elapsed, Style::default().fg(Color::Green));
                    let content_styled =
                        Span::styled(entry.content.clone(), Style::default().fg(Color::White));

                    let highlight_symbol = if actual_index == selected {
                        Span::styled(
                            "> ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("  ")
                    };

                    let line = Line::from(vec![
                        highlight_symbol,
                        elapsed_styled,
                        Span::raw(" "),
                        content_styled,
                    ]);

                    if actual_index == selected {
                        ListItem::new(line).style(Style::default().bg(Color::DarkGray))
                    } else {
                        ListItem::new(line)
                    }
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" zp ")
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
            );

            f.render_widget(
                list,
                Rect {
                    y: size.height - app_height,
                    width: size.width,
                    height: app_height,
                    ..size
                },
            );
        })?;

        // Handle input
        if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected = selected.saturating_sub(1);
                    }
                }
                KeyCode::Down => {
                    if selected < entries.len().saturating_sub(1) {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    let content = &entries[selected].content;
                    clipboard.set_text(content.to_owned()).unwrap();
                    println!("Copied: {}", content);
                    break;
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    Ok(())
}
