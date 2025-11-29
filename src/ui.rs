//! Terminal user interface for the commit wizard.
//!
//! This module provides an interactive TUI using `ratatui` for selecting
//! and managing commit groups.

use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::crossterm::{execute, terminal};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Wrap,
};
use ratatui::Terminal;

use crate::git::commit_group;
use crate::types::{ActivePanel, AppState};

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
    // If editor help is shown, handle it first
    if app.show_editor_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                app.close_editor_help();
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }

    // If diff viewer is active, handle its keys
    if app.show_diff_viewer {
        match key.code {
            KeyCode::Esc => {
                app.close_diff();
                return Ok(false);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_diff_down();
                return Ok(false);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_diff_up();
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }

    // If editor is active, route all events to the editor
    if app.editor.is_active() {
        // Check for help toggle first
        if key.code == KeyCode::Char('?') && key.modifiers.is_empty() {
            app.toggle_editor_help();
            return Ok(false);
        }

        // Convert KeyEvent to CrosstermEvent for the editor
        use ratatui::crossterm::event::Event as CrosstermEvent;
        let editor_continues = app.editor.handle_event(CrosstermEvent::Key(key))?;

        if !editor_continues {
            // Editor was closed (Ctrl+S = save, Ctrl+C = cancel)
            // Check if it was a save (not a cancel)
            if key.code == KeyCode::Char('s') && key.modifiers == KeyModifiers::CONTROL {
                // Save: transfer text back to the selected group
                let text = app.editor.text();
                if let Some(group) = app.selected_group_mut() {
                    group.set_from_commit_text(&text);
                }
            }
            // For Ctrl+C, editor.cancel() already restored original text
        }

        return Ok(false); // Continue running
    }

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
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.activate_previous_panel();
            } else {
                app.activate_next_panel();
            }
        }
        KeyCode::BackTab => {
            app.activate_previous_panel();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // Navigate based on active panel
            match app.active_panel {
                ActivePanel::Groups => app.select_next(),
                ActivePanel::CommitMessage => app.scroll_commit_message_down(),
                ActivePanel::Files => app.select_next_file(),
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            // Navigate based on active panel
            match app.active_panel {
                ActivePanel::Groups => app.select_previous(),
                ActivePanel::CommitMessage => app.scroll_commit_message_up(),
                ActivePanel::Files => app.select_previous_file(),
            }
        }
        KeyCode::Char('e') => {
            handle_edit_action(app, terminal)?;
        }
        KeyCode::Char('d') => {
            handle_diff_action(app, repo_path)?;
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

/// Handles the edit action (activates integrated editor).
fn handle_edit_action<B: ratatui::backend::Backend + std::io::Write>(
    app: &mut AppState,
    _terminal: &mut Terminal<B>,
) -> Result<()> {
    // Check if selected group is already committed
    if let Some(group) = app.selected_group() {
        if group.is_committed() {
            app.set_status("✗ Cannot edit already committed group");
            return Ok(());
        }
    }

    // Get the current commit message first
    let message = app
        .selected_group()
        .map(|g| g.full_message())
        .unwrap_or_default();

    // Activate the integrated editor
    app.editor.activate(message);

    Ok(())
}

/// Handles the diff viewer action (shows diff for selected file).
fn handle_diff_action(app: &mut AppState, repo_path: &Path) -> Result<()> {
    use git2::Repository;

    // Only allow diff from Files panel
    if app.active_panel != ActivePanel::Files {
        app.set_status("ℹ Switch to Files panel (Tab) to view diffs");
        return Ok(());
    }

    // Check if group is committed first
    let is_committed = app
        .selected_group()
        .map(|g| g.is_committed())
        .unwrap_or(false);
    if is_committed {
        app.set_status("ℹ Viewing diff for already committed group");
    }

    // Get the selected file from the active group
    let file_path = match app.selected_file() {
        Some(file) => file.path.clone(),
        None => {
            app.set_status("✗ No files in selected group");
            return Ok(());
        }
    };

    // Get the repository
    let repo = Repository::discover(repo_path)?;

    // Get the diff for the file
    match crate::git::get_file_diff(&repo, &file_path) {
        Ok(diff_content) => {
            if diff_content.trim().is_empty() {
                app.set_status("✗ No staged changes for this file");
            } else {
                app.show_diff(file_path, diff_content);
            }
        }
        Err(e) => {
            app.set_status(format!("✗ Failed to get diff: {}", e));
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
    let selected_idx = app.selected_index;
    if let Some(group) = app.selected_group() {
        // Check if already committed
        if group.is_committed() {
            app.set_status("✗ Group already committed");
            return Ok(());
        }

        match commit_group(repo_path, group) {
            Ok(()) => {
                // Mark the group as committed
                if let Some(group) = app.groups.get_mut(selected_idx) {
                    group.mark_as_committed();
                }
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
    use crate::git::commit_group;

    // Filter out already committed groups
    let uncommitted_count = app.groups.iter().filter(|g| !g.is_committed()).count();

    if uncommitted_count == 0 {
        app.set_status("✗ All groups already committed");
        return Ok(());
    }

    let mut committed_count = 0;
    let mut failed = false;

    for group in &mut app.groups {
        if !group.is_committed() {
            match commit_group(repo_path, group) {
                Ok(()) => {
                    group.mark_as_committed();
                    committed_count += 1;
                }
                Err(e) => {
                    app.set_status(format!("✗ Failed to commit group: {}", e));
                    failed = true;
                    break;
                }
            }
        }
    }

    if !failed {
        app.set_status(format!(
            "✓ Successfully committed {} group(s)",
            committed_count
        ));
    }

    Ok(())
}

/// Draws the user interface.
fn draw_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
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
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(vertical_chunks[0]);

        // Left panel: group list
        let is_groups_active = app.active_panel == ActivePanel::Groups;
        draw_groups_panel(f, app, content_chunks[0], is_groups_active);

        // Right panel split horizontally: commit message (50%) and files (50%)
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[1]);

        // Right top: commit message
        let is_message_active = app.active_panel == ActivePanel::CommitMessage;
        draw_commit_message_panel(f, app, right_chunks[0], is_message_active);

        // Right bottom: files
        let is_files_active = app.active_panel == ActivePanel::Files;
        draw_files_panel(f, app, right_chunks[1], is_files_active);

        // Bottom shortcuts bar
        draw_shortcuts_bar(f, ai_enabled, vertical_chunks[1]);

        // Draw status popup overlay if there's a status message
        if !app.status_message.is_empty() {
            draw_status_popup(f, app, size);
        }

        // Draw editor overlay if editor is active
        if app.editor.is_active() {
            draw_editor_overlay(f, app, size);
        }

        // Draw diff viewer popup if active (higher z-order than editor)
        if app.show_diff_viewer {
            draw_diff_viewer_popup(f, app, size);
        }

        // Draw editor help popup if active (highest z-order)
        if app.show_editor_help {
            draw_editor_help_popup(f, app, size);
        }
    })?;

    Ok(())
}

/// Draws the editor overlay when the integrated editor is active.
fn draw_editor_overlay(f: &mut ratatui::Frame, app: &mut AppState, area: ratatui::layout::Rect) {
    use edtui::EditorView;
    use ratatui::widgets::{Block, Borders};

    // Create a block with borders and title
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Commit Message Editor (Ctrl+S=save, Ctrl+C=cancel) ")
        .border_style(Style::default().fg(Color::Cyan));

    // Get the inner area for the editor view
    let inner_area = block.inner(area);

    // Clear the area for the popup
    f.render_widget(Clear, area);

    // Render the block first
    f.render_widget(block, area);

    // Render the editor view inside
    let editor_view = EditorView::new(app.editor.state_mut());
    f.render_widget(editor_view, inner_area);
}

/// Draws the left panel showing the list of commit groups.
fn draw_groups_panel(
    f: &mut ratatui::Frame,
    app: &AppState,
    area: ratatui::layout::Rect,
    is_active: bool,
) {
    let items: Vec<ListItem> = app
        .groups
        .iter()
        .enumerate()
        .map(|(idx, group)| {
            let header = group.header();
            let is_selected = idx == app.selected_index;
            let is_committed = group.is_committed();

            let style = if is_committed {
                // Committed groups are grayed out
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM)
            } else if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_committed {
                "✓ "
            } else if is_selected {
                "▶ "
            } else {
                "  "
            };
            let content = format!("{}{}", prefix, header);

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();
    let border_color = if is_active { Color::Green } else { Color::Cyan };
    let title = format!(" Commit Groups ({}) ", app.groups.len());
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
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
fn draw_commit_message_panel(
    f: &mut ratatui::Frame,
    app: &AppState,
    area: ratatui::layout::Rect,
    is_active: bool,
) {
    if let Some(group) = app.selected_group() {
        let msg = group.full_message();
        let all_lines: Vec<&str> = msg.lines().collect();
        let line_count = all_lines.len();

        let border_color = if is_active {
            Color::Green
        } else {
            Color::White
        };

        // Calculate visible lines with scroll offset
        let visible_height = area.height.saturating_sub(2) as usize;
        let start_line = app.commit_message_scroll_offset;
        let end_line = (start_line + visible_height).min(line_count);
        let visible_text = all_lines[start_line..end_line].join("\n");

        let paragraph = Paragraph::new(visible_text)
            .block(
                Block::default()
                    .title(" Commit Message ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            // We intentionally use `trim: true` here to remove trailing whitespace from commit messages,
            // as trailing spaces are rarely meaningful in commit messages and can cause formatting issues.
            // Note: The files panel uses `trim: false` to preserve whitespace, which is important for file diffs.
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);

        // Add scrollbar if active and content is longer than visible area
        if is_active && line_count > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state = ScrollbarState::new(line_count.saturating_sub(1))
                .position(app.commit_message_scroll_offset);
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
        let border_color = if is_active {
            Color::Green
        } else {
            Color::White
        };
        let empty = Paragraph::new("No group selected").block(
            Block::default()
                .title(" Commit Message ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(empty, area);
    }
}

/// Draws the files panel (right bottom).
fn draw_files_panel(
    f: &mut ratatui::Frame,
    app: &AppState,
    area: ratatui::layout::Rect,
    is_active: bool,
) {
    if let Some(group) = app.selected_group() {
        let file_lines: Vec<Line> = group
            .files
            .iter()
            .enumerate()
            .map(|(idx, file)| {
                let is_selected = idx == app.selected_file_index;

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

                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(
                        format!("{} ", status_icon),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(&file.path, style),
                ])
            })
            .collect();

        let border_color = if is_active { Color::Green } else { Color::Blue };
        let file_lines_len = file_lines.len();
        let files_paragraph = Paragraph::new(file_lines)
            .block(
                Block::default()
                    .title(format!(" Files ({}) ", group.files.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(files_paragraph, area);

        // Add scrollbar if active and there are many files
        if is_active && file_lines_len > area.height.saturating_sub(2) as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state = ScrollbarState::new(file_lines_len.saturating_sub(1))
                .position(app.selected_file_index);
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
        let border_color = if is_active { Color::Green } else { Color::Blue };
        let empty = Paragraph::new("No files").block(
            Block::default()
                .title(" Files ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(empty, area);
    }
}

/// Draws the keyboard shortcuts bar at the bottom.
fn draw_shortcuts_bar(f: &mut ratatui::Frame, ai_enabled: bool, area: ratatui::layout::Rect) {
    let mut shortcuts = vec![
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
            " d ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Diff "),
    ];

    if ai_enabled {
        shortcuts.push(Span::styled(
            " a ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        shortcuts.push(Span::raw("AI "));
    }

    shortcuts.extend(vec![
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
    ]);

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
    // Fixed height provides consistent appearance across terminal sizes
    const STATUS_POPUP_HEIGHT: u16 = 6;

    // Calculate popup size (70% width, fixed height)
    let popup_width = (area.width as f32 * 0.7) as u16;
    let popup_height = STATUS_POPUP_HEIGHT;

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
    f.render_widget(popup_block.clone(), popup_area);

    // Inner area for content (inside borders)
    let inner_area = popup_block.inner(popup_area);

    // Split inner area into message area and button area
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(inner_area);

    // Add scrollbar if there is more text than fits
    let total_lines = app.status_message.lines().count();
    let visible_lines = popup_chunks[0].height as usize;
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(total_lines.saturating_sub(1)).position(app.popup_scroll_offset);
        f.render_stateful_widget(
            scrollbar,
            popup_area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }

    // Status message with scroll offset applied
    let lines: Vec<_> = app.status_message.lines().collect();
    let start_line = app.popup_scroll_offset;
    let end_line = (start_line + visible_lines).min(lines.len());
    let visible_text = lines[start_line..end_line].join("\n");

    let status_text = Paragraph::new(visible_text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    f.render_widget(status_text, popup_chunks[0]);

    // Close button (bottom right, inside popup) - visually highlighted as active
    let button_width = 12u16;
    let button_height = 1u16;
    let button_area = Rect {
        x: popup_chunks[1].x + popup_chunks[1].width.saturating_sub(button_width + 1),
        y: popup_chunks[1].y,
        width: button_width,
        height: button_height,
    };

    // Create close button highlighted to show it's active (Enter closes)
    let button_text = "[ Close ]";
    let button_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let button = Paragraph::new(button_text)
        .style(button_style)
        .alignment(Alignment::Center);
    f.render_widget(button, button_area);
}

/// Draws the editor help popup showing keyboard shortcuts.
fn draw_editor_help_popup(f: &mut ratatui::Frame, _app: &AppState, area: ratatui::layout::Rect) {
    // Calculate popup size (60% width, 70% height)
    let popup_width = (area.width as f32 * 0.6) as u16;
    let popup_height = (area.height as f32 * 0.7) as u16;

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

    // Render popup border
    let popup_block = Block::default()
        .title(" Editor Keyboard Shortcuts (Esc to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(popup_block.clone(), popup_area);

    // Inner area for content
    let inner_area = popup_block.inner(popup_area);

    // Help text content
    let help_text = vec![
        Line::from(vec![
            Span::styled(
                "Ctrl+S",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    Save and close"),
        ]),
        Line::from(vec![
            Span::styled(
                "Ctrl+C",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    Cancel without saving"),
        ]),
        Line::from(vec![
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Toggle this help"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "--- Vim-Style Navigation ---",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "h j k l",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   Left, Down, Up, Right"),
        ]),
        Line::from(vec![
            Span::styled(
                "0",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Start of line"),
        ]),
        Line::from(vec![
            Span::styled(
                "$",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         End of line"),
        ]),
        Line::from(vec![
            Span::styled(
                "w",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Next word"),
        ]),
        Line::from(vec![
            Span::styled(
                "b",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Previous word"),
        ]),
        Line::from(vec![
            Span::styled(
                "gg",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("        Go to first line"),
        ]),
        Line::from(vec![
            Span::styled(
                "G",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Go to last line"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "--- Vim-Style Editing ---",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "i",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Insert mode"),
        ]),
        Line::from(vec![
            Span::styled(
                "a",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Append after cursor"),
        ]),
        Line::from(vec![
            Span::styled(
                "A",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Append at end of line"),
        ]),
        Line::from(vec![
            Span::styled(
                "o",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Open line below"),
        ]),
        Line::from(vec![
            Span::styled(
                "O",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Open line above"),
        ]),
        Line::from(vec![
            Span::styled(
                "x",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Delete character"),
        ]),
        Line::from(vec![
            Span::styled(
                "dd",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("        Delete line"),
        ]),
        Line::from(vec![
            Span::styled(
                "yy",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("        Yank (copy) line"),
        ]),
        Line::from(vec![
            Span::styled(
                "p",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Paste"),
        ]),
        Line::from(vec![
            Span::styled(
                "u",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Undo"),
        ]),
        Line::from(vec![
            Span::styled(
                "Ctrl+R",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    Redo"),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);
    f.render_widget(paragraph, inner_area);
}

/// Draws the diff viewer popup showing file changes.
fn draw_diff_viewer_popup(f: &mut ratatui::Frame, app: &AppState, area: ratatui::layout::Rect) {
    // Calculate popup size (90% width, 80% height)
    let popup_width = (area.width as f32 * 0.9) as u16;
    let popup_height = (area.height as f32 * 0.8) as u16;

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

    // Render popup border
    let title = format!(
        " Diff Viewer: {} (↑↓ scroll, Esc close) ",
        app.diff_file_path
    );
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(popup_block.clone(), popup_area);

    // Inner area for content
    let inner_area = popup_block.inner(popup_area);

    // Add scrollbar if there is more text than fits
    let total_lines = app.diff_content.lines().count();
    let visible_lines = inner_area.height as usize;
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(total_lines.saturating_sub(1)).position(app.diff_scroll_offset);
        f.render_stateful_widget(
            scrollbar,
            popup_area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }

    // Parse and render diff with syntax highlighting
    let lines: Vec<_> = app.diff_content.lines().collect();
    let start_line = app.diff_scroll_offset;
    let end_line = (start_line + visible_lines).min(lines.len());

    let styled_lines: Vec<Line> = lines[start_line..end_line]
        .iter()
        .map(|line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                Style::default().fg(Color::Green)
            } else if line.starts_with('-') && !line.starts_with("---") {
                Style::default().fg(Color::Red)
            } else if line.starts_with("@@") {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if line.starts_with("diff") || line.starts_with("index") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Line::from(Span::styled(*line, style))
        })
        .collect();

    let paragraph = Paragraph::new(styled_lines)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);
    f.render_widget(paragraph, inner_area);
}
