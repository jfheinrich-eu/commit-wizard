//! Terminal user interface for the commit wizard.
//!
//! This module provides an interactive TUI using `ratatui` for selecting
//! and managing commit groups.

use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Terminal;

use crate::editor::edit_text_in_editor;
use crate::git::{commit_all_groups, commit_group};
use crate::types::AppState;



/// Runs the terminal user interface event loop.
///
/// # Arguments
///
/// * `app` - The application state containing commit groups
/// * `repo_path` - Path to the git repository
/// * `ai_enabled` - Whether AI-powered message generation is enabled
///
/// # Returns
///
/// Ok if the user quits normally, Err on terminal errors.
///
/// # Keyboard Controls
///
/// - `↑`/`↓` or `k`/`j` - Navigate between commit groups
/// - `e` - Edit the selected commit message in external editor
/// - `a` - Generate commit message using AI (if enabled)
/// - `c` - Commit the selected group
/// - `C` - Commit all groups
/// - `Ctrl+L` - Clear status message
/// - `q` or `Esc` - Quit
pub fn run_tui(mut app: AppState, repo_path: &Path, ai_enabled: bool) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let result = run_event_loop(&mut terminal, &mut app, repo_path, ai_enabled);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Runs the main event loop.
fn run_event_loop<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    repo_path: &Path,
    ai_enabled: bool,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        // Draw UI
        draw_ui(terminal, app)?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if handle_key_event(key, app, repo_path, terminal, ai_enabled)? {
                    break; // User wants to quit
                }
            }
        }

        // Tick
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

/// Handles a keyboard event.
///
/// Returns `true` if the application should quit.
fn handle_key_event<B: ratatui::backend::Backend + std::io::Write>(
    key: KeyEvent,
    app: &mut AppState,
    repo_path: &Path,
    terminal: &mut Terminal<B>,
    ai_enabled: bool,
) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            return Ok(true);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Char('e') => {
            handle_edit_action(app, terminal)?;
        }
        KeyCode::Char('a') => {
            if ai_enabled {
                handle_ai_generate_action(app, terminal)?;
            } else {
                app.set_status("✗ AI mode not enabled. Use --ai or --copilot flag.");
            }
        }
        KeyCode::Char('c') => {
            handle_commit_action(app, repo_path)?;
        }
        KeyCode::Char('C') if key.modifiers.is_empty() => {
            handle_commit_all_action(app, repo_path)?;
        }
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_status();
        }
        _ => {}
    }

    Ok(false)
}

/// Handles the edit action (opens external editor).
fn handle_edit_action<B: ratatui::backend::Backend + std::io::Write>(
    app: &mut AppState,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    if let Some(group) = app.selected_group_mut() {
        // Temporarily leave raw mode for editor
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        let result = edit_text_in_editor(&group.full_message());

        // Restore raw mode
        enable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;
        terminal.hide_cursor()?;

        match result {
            Ok(text) => {
                group.set_from_commit_text(&text);
                app.set_status("✓ Updated commit message from editor");
            }
            Err(e) => {
                app.set_status(format!("✗ Editor error: {}. Kept old message.", e));
            }
        }
    }

    Ok(())
}

/// Handles AI-powered commit message generation.
fn handle_ai_generate_action<B: ratatui::backend::Backend + std::io::Write>(
    app: &mut AppState,
    _terminal: &mut Terminal<B>,
) -> Result<()> {
    use crate::ai::generate_commit_message;
    use git2::Repository;

    // Clone data we need before borrowing mutably
    let (group_clone, files_clone) = if let Some(group) = app.selected_group() {
        (group.clone(), group.files.clone())
    } else {
        return Ok(());
    };

    app.set_status("🤖 Generating commit message with AI...");

    // Try to get git diff for better context
    let diff = Repository::discover(".")
        .ok()
        .and_then(|repo| {
            let mut diff_text = String::new();
            for file in &files_clone {
                if let Ok(diff) = crate::git::get_file_diff(&repo, &file.path) {
                    diff_text.push_str(&diff);
                }
            }
            if diff_text.is_empty() {
                None
            } else {
                Some(diff_text)
            }
        });

    match generate_commit_message(&group_clone, &files_clone, diff.as_deref()) {
        Ok((description, body)) => {
            // Now update the actual group
            if let Some(group) = app.selected_group_mut() {
                group.description = description;
                // Convert optional body string to Vec<String> of lines
                group.body_lines = body
                    .map(|b| b.lines().map(String::from).collect())
                    .unwrap_or_default();
            }
            app.set_status("✓ AI generated commit message successfully");
        }
        Err(e) => {
            app.set_status(format!("✗ AI generation failed: {}. Check GITHUB_TOKEN.", e));
        }
    }

    Ok(())
}

/// Handles committing a single group.
fn handle_commit_action(app: &mut AppState, repo_path: &Path) -> Result<()> {
    if let Some(group) = app.selected_group() {
        match commit_group(repo_path, group) {
            Ok(()) => {
                app.set_status("✓ Committed selected group successfully");
            }
            Err(e) => {
                app.set_status(format!("✗ Commit failed: {}", e));
            }
        }
    }
    Ok(())
}

/// Handles committing all groups.
fn handle_commit_all_action(app: &mut AppState, repo_path: &Path) -> Result<()> {
    match commit_all_groups(repo_path, &app.groups) {
        Ok(()) => {
            app.set_status("✓ Successfully committed all groups");
        }
        Err(e) => {
            app.set_status(format!("✗ Failed to commit all: {}", e));
        }
    }
    Ok(())
}

/// Draws the user interface.
fn draw_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &AppState,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();

        // Main layout: left panel (groups) and right panel (details)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(size);

        // Left panel: group list
        draw_groups_panel(f, app, main_chunks[0]);

        // Right panel: message and files
        draw_details_panel(f, app, main_chunks[1]);
    })?;

    Ok(())
}

/// Draws the left panel showing the list of commit groups.
fn draw_groups_panel(
    f: &mut ratatui::Frame,
    app: &AppState,
    area: ratatui::layout::Rect,
) {
    let items: Vec<ListItem> = app
        .groups
        .iter()
        .enumerate()
        .map(|(idx, group)| {
            let header = group.header();
            let is_selected = idx == app.selected_index;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let content = format!("{}{}", prefix, header);

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let title = format!(" Commit Groups ({}) ", app.groups.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(list, area);
}

/// Draws the right panel showing commit message and files.
fn draw_details_panel(
    f: &mut ratatui::Frame,
    app: &AppState,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(40),
            Constraint::Length(3),
        ])
        .split(area);

    // Top: commit message
    if let Some(group) = app.selected_group() {
        let msg = group.full_message();
        let paragraph = Paragraph::new(msg)
            .block(
                Block::default()
                    .title(" Commit Message ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, chunks[0]);

        // Middle: file list
        let file_lines: Vec<Line> = group
            .files
            .iter()
            .map(|file| {
                let status_icon = if file.is_new() {
                    "+"
                } else if file.is_deleted() {
                    "-"
                } else if file.is_modified() {
                    "~"
                } else if file.is_renamed() {
                    "→"
                } else {
                    "•"
                };

                Line::from(vec![
                    Span::styled(
                        format!(" {} ", status_icon),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::raw(&file.path),
                ])
            })
            .collect();

        let files_paragraph = Paragraph::new(file_lines)
            .block(
                Block::default()
                    .title(format!(" Files ({}) ", group.files.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(files_paragraph, chunks[1]);
    } else {
        let empty = Paragraph::new("No group selected")
            .block(
                Block::default()
                    .title(" Commit Message ")
                    .borders(Borders::ALL),
            );
        f.render_widget(empty, chunks[0]);
    }

    // Bottom: status bar
    let status = Paragraph::new(app.status_message.as_str())
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[2]);
}
