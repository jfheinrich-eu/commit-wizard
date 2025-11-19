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
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Wrap,
};
use ratatui::Terminal;
use tui_framework_experiment::Button;

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
        draw_ui(terminal, app, ai_enabled)?;

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
    // If popup is active, handle popup-specific keys first
    if app.popup_active {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.clear_status();
                return Ok(false);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_popup_down();
                return Ok(false);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_popup_up();
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }

    // Normal mode key handling
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

        // Clear and redraw the terminal after returning from editor
        terminal.clear()?;

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
    let diff = Repository::discover(".").ok().and_then(|repo| {
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
            app.set_status(format!(
                "✗ AI generation failed: {}. Check GITHUB_TOKEN.",
                e
            ));
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
    ai_enabled: bool,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();

        // Main vertical layout: content area and shortcuts bar (3 lines for one text line)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(size);

        // Content area: left panel (50%) and right panel (50%)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vertical_chunks[0]);

        // Left panel: group list
        draw_groups_panel(f, app, content_chunks[0]);

        // Right panel split horizontally: commit message (50%) and files (50%)
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[1]);

        // Right top: commit message
        draw_commit_message_panel(f, app, right_chunks[0]);

        // Right bottom: files
        draw_files_panel(f, app, right_chunks[1]);

        // Bottom shortcuts bar
        draw_shortcuts_bar(f, ai_enabled, vertical_chunks[1]);

        // Draw status popup overlay if there's a status message
        if !app.status_message.is_empty() {
            draw_status_popup(f, app, size);
        }
    })?;

    Ok(())
}

/// Draws the left panel showing the list of commit groups.
fn draw_groups_panel(f: &mut ratatui::Frame, app: &AppState, area: ratatui::layout::Rect) {
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
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);

    // Add scrollbar on the right edge
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    let mut scrollbar_state =
        ScrollbarState::new(app.groups.len().saturating_sub(1)).position(app.selected_index);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        }),
        &mut scrollbar_state,
    );
}

/// Draws the commit message panel (right top).
fn draw_commit_message_panel(f: &mut ratatui::Frame, app: &AppState, area: ratatui::layout::Rect) {
    if let Some(group) = app.selected_group() {
        let msg = group.full_message();
        let line_count = msg.lines().count();
        let paragraph = Paragraph::new(msg)
            .block(
                Block::default()
                    .title(" Commit Message ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);

        // Add scrollbar if content is longer than visible area
        if line_count > area.height.saturating_sub(2) as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state = ScrollbarState::new(line_count.saturating_sub(1)).position(0);
            f.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    horizontal: 0,
                    vertical: 1,
                }),
                &mut scrollbar_state,
            );
        }
    } else {
        let empty = Paragraph::new("No group selected").block(
            Block::default()
                .title(" Commit Message ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        f.render_widget(empty, area);
    }
}

/// Draws the files panel (right bottom).
fn draw_files_panel(f: &mut ratatui::Frame, app: &AppState, area: ratatui::layout::Rect) {
    if let Some(group) = app.selected_group() {
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

        let files_paragraph = Paragraph::new(file_lines.clone())
            .block(
                Block::default()
                    .title(format!(" Files ({}) ", group.files.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(files_paragraph, area);

        // Add scrollbar if there are many files
        if file_lines.len() > area.height.saturating_sub(2) as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state =
                ScrollbarState::new(file_lines.len().saturating_sub(1)).position(0);
            f.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    horizontal: 0,
                    vertical: 1,
                }),
                &mut scrollbar_state,
            );
        }
    } else {
        let empty = Paragraph::new("No files").block(
            Block::default()
                .title(" Files ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
        f.render_widget(empty, area);
    }
}

/// Draws the keyboard shortcuts bar at the bottom.
fn draw_shortcuts_bar(f: &mut ratatui::Frame, ai_enabled: bool, area: ratatui::layout::Rect) {
    let shortcuts = if ai_enabled {
        vec![
            Span::styled(
                " ↑↓/jk ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate "),
            Span::styled(
                " e ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Edit "),
            Span::styled(
                " a ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("AI "),
            Span::styled(
                " c ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Commit "),
            Span::styled(
                " C ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Commit All "),
            Span::styled(
                " Ctrl+L ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Clear Status "),
            Span::styled(
                " q ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit"),
        ]
    } else {
        vec![
            Span::styled(
                " ↑↓/jk ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate "),
            Span::styled(
                " e ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Edit "),
            Span::styled(
                " c ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Commit "),
            Span::styled(
                " C ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Commit All "),
            Span::styled(
                " Ctrl+L ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Clear Status "),
            Span::styled(
                " q ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit"),
        ]
    };

    let shortcuts_line = Line::from(shortcuts);
    let shortcuts_paragraph = Paragraph::new(shortcuts_line)
        .block(
            Block::default()
                .title(" Keyboard Shortcuts ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);

    f.render_widget(shortcuts_paragraph, area);
}

/// Draws a centered status popup overlay.
fn draw_status_popup(f: &mut ratatui::Frame, app: &AppState, area: ratatui::layout::Rect) {
    // Calculate popup size (70% width, 10% height)
    let popup_width = (area.width as f32 * 0.7) as u16;
    let popup_height = 6; //(area.height as f32 * 0.1) as u16;

    // Center the popup
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area for the popup
    f.render_widget(Clear, popup_area);

    // Render popup border first - highlight if active
    let border_color = if app.popup_active {
        Color::Green
    } else {
        Color::Yellow
    };
    let title = if app.popup_active {
        " Status (↑↓ scroll, Enter/Esc close) "
    } else {
        " Status "
    };
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    f.render_widget(popup_block, popup_area);

    // Add scrollbar if there more text than fits
    let total_lines = app.status_message.lines().count();
    let visible_lines = popup_area.height.saturating_sub(3) as usize;
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(total_lines.saturating_sub(1)).position(app.popup_scroll_offset);
        f.render_stateful_widget(
            scrollbar,
            popup_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }

    // Inner area for content (inside borders)
    let inner_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width.saturating_sub(2),
        height: popup_area.height.saturating_sub(2),
    };

    // Split inner area into message area and button area
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(inner_area);

    // Status message with scroll offset applied
    let lines: Vec<_> = app.status_message.lines().collect();
    let visible_lines = popup_chunks[0].height as usize;
    let start_line = app.popup_scroll_offset;
    let end_line = (start_line + visible_lines).min(lines.len());
    let visible_text = lines[start_line..end_line].join("\n");

    let status_text = Paragraph::new(visible_text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    f.render_widget(status_text, popup_chunks[0]);

    // Close button (bottom right, inside popup)
    let button_width = 10u16;
    let button_height = 1u16;
    let button_area = Rect {
        x: popup_chunks[1].x + popup_chunks[1].width.saturating_sub(button_width + 1),
        y: popup_chunks[1].y,
        width: button_width,
        height: button_height,
    };

    // Create close button with default theme
    let close_button = Button::new(" Close ");
    f.render_widget(&close_button, button_area);
}
