use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{
    self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use git2::{Repository, Status, StatusOptions};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use regex::Regex;
use tempfile::NamedTempFile;

/// CLI options (extend later if needed).
#[derive(Parser, Debug)]
#[command(author, version, about = "Conventional Commit helper", long_about = None)]
struct Cli {
    /// Optional path to the git repository (defaults to current directory)
    #[arg(short, long)]
    repo: Option<PathBuf>,
}

/// Logical commit type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Chore,
    Ci,
    Build,
}

impl CommitType {
    fn as_str(&self) -> &'static str {
        match self {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Style => "style",
            CommitType::Refactor => "refactor",
            CommitType::Perf => "perf",
            CommitType::Test => "test",
            CommitType::Chore => "chore",
            CommitType::Ci => "ci",
            CommitType::Build => "build",
        }
    }
}

/// Single changed file.
#[derive(Debug, Clone)]
struct ChangedFile {
    path: String,
    status: Status,
}

/// A logical group of changes, representing one potential commit.
#[derive(Debug, Clone)]
struct ChangeGroup {
    commit_type: CommitType,
    scope: Option<String>,
    files: Vec<ChangedFile>,
    ticket: Option<String>,
    description: String,
    body_lines: Vec<String>,
}

impl ChangeGroup {
    fn header(&self) -> String {
        let ctype = self.commit_type.as_str();
        let scope_part = self.scope.as_ref().map(|s| format!("({})", s)).unwrap_or_default();
        let ticket_part = self.ticket.clone().unwrap_or_else(|| "NO-TICKET".to_string());

        // Base header prefix without description.
        let base_prefix = if scope_part.is_empty() {
            format!("{ctype}: {ticket_part}: ")
        } else {
            format!("{ctype}{scope_part}: {ticket_part}: ")
        };

        let max_len = 72usize;
        let available_for_desc = max_len.saturating_sub(base_prefix.len());
        let mut desc = self.description.clone();
        if desc.len() > available_for_desc {
            desc.truncate(available_for_desc.saturating_sub(3));
            desc.push_str("...");
        }

        format!("{base_prefix}{desc}")
    }

    fn full_message(&self) -> String {
        let mut msg = String::new();
        msg.push_str(&self.header());
        if !self.body_lines.is_empty() {
            msg.push('\n');
            msg.push('\n');
            for line in &self.body_lines {
                msg.push_str("- ");
                msg.push_str(line);
                msg.push('\n');
            }
        }
        msg
    }

    fn set_from_commit_text(&mut self, text: &str) {
        // This is intentionally simple: we keep the header, and body lines starting with "- ".
        let mut lines = text.lines();
        if let Some(header) = lines.next() {
            // Trust user. Do not re-parse header, just take it.
            // If user made it non-conventional, that is their responsibility.
            // For display purpose only, we keep description and body separately.
            let header_trimmed = header.trim().to_string();
            // We try to extract description part after last ": ".
            if let Some(idx) = header_trimmed.rfind(": ") {
                self.description = header_trimmed[idx + 2..].to_string();
            }
        }

        let mut body = Vec::new();
        for line in lines {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- ") {
                body.push(trimmed[2..].to_string());
            } else if !trimmed.is_empty() {
                // Treat non-bullet line as one bullet item.
                body.push(trimmed.to_string());
            }
        }
        self.body_lines = body;
    }
}

/// Application state for the TUI.
struct AppState {
    groups: Vec<ChangeGroup>,
    selected_index: usize,
    status_message: String,
}

impl AppState {
    fn new(groups: Vec<ChangeGroup>) -> Self {
        Self {
            groups,
            selected_index: 0,
            status_message: "↑/↓ select, e edit, c commit group, C commit all, q quit".to_string(),
        }
    }

    fn selected_group_mut(&mut self) -> Option<&mut ChangeGroup> {
        self.groups.get_mut(self.selected_index)
    }

    fn selected_group(&self) -> Option<&ChangeGroup> {
        self.groups.get(self.selected_index)
    }
}

/// Determine commit type heuristically from file path.
fn infer_commit_type(path: &str) -> CommitType {
    let lower = path.to_lowercase();

    if lower.contains("test") {
        return CommitType::Test;
    }

    if lower.ends_with(".md")
        || lower.ends_with(".rst")
        || lower.contains("/docs/")
        || lower.starts_with("docs/")
    {
        return CommitType::Docs;
    }

    if lower.contains(".github")
        || lower.contains(".gitlab")
        || lower.contains("pipeline")
        || lower.contains("ci.yml")
        || lower.contains("ci.yaml")
    {
        return CommitType::Ci;
    }

    if lower.contains("dockerfile")
        || lower.ends_with("package.json")
        || lower.ends_with("composer.json")
        || lower.ends_with("build.gradle")
        || lower.ends_with("pom.xml")
    {
        return CommitType::Build;
    }

    if lower.ends_with(".css")
        || lower.ends_with(".scss")
        || lower.ends_with(".less")
        || lower.contains("style")
    {
        return CommitType::Style;
    }

    // Default heuristic: feature.
    CommitType::Feat
}

/// Extract scope from a file path (first segment, e.g. "src", "backend", "docs").
fn infer_scope(path: &str) -> Option<String> {
    let mut parts = path.split('/');
    let first = parts.next()?;
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

/// Infer a short description from group content (very simplistic).
fn infer_description(files: &[ChangedFile], commit_type: CommitType, scope: &Option<String>) -> String {
    if let Some(scope_value) = scope {
        match commit_type {
            CommitType::Test => format!("update tests for {scope_value}"),
            CommitType::Docs => format!("update docs for {scope_value}"),
            CommitType::Ci => format!("update CI for {scope_value}"),
            CommitType::Build => format!("update build for {scope_value}"),
            _ => format!("update {scope_value}"),
        }
    } else if files.len() == 1 {
        format!("update {}", files[0].path)
    } else {
        "update project files".to_string()
    }
}

/// Extract reasonable bullet points from file list.
fn infer_body_lines(files: &[ChangedFile]) -> Vec<String> {
    files
        .iter()
        .map(|f| format!("touch {}", f.path))
        .collect()
}

/// Group changed files into logical commit groups.
fn build_groups(files: Vec<ChangedFile>, ticket: Option<String>) -> Vec<ChangeGroup> {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct GroupKey {
        commit_type: CommitType,
        scope: Option<String>,
    }

    let mut map: BTreeMap<GroupKey, Vec<ChangedFile>> = BTreeMap::new();

    for file in files {
        let ctype = infer_commit_type(&file.path);
        let scope = infer_scope(&file.path);
        let key = GroupKey {
            commit_type: ctype,
            scope,
        };
        map.entry(key).or_default().push(file);
    }

    let mut groups = Vec::new();
    for (key, group_files) in map {
        let description = infer_description(&group_files, key.commit_type, &key.scope);
        let body_lines = infer_body_lines(&group_files);
        groups.push(ChangeGroup {
            commit_type: key.commit_type,
            scope: key.scope.clone(),
            files: group_files,
            ticket: ticket.clone(),
            description,
            body_lines,
        });
    }

    groups
}

/// Extract ticket number like "LU-1234" from branch name.
fn extract_ticket_from_branch(branch: &str) -> Option<String> {
    let re = Regex::new(r"([A-Z]+-\d+)").ok()?;
    let caps = re.captures(branch)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// Collect staged files from git, using libgit2.
fn collect_staged_files(repo: &Repository) -> Result<Vec<ChangedFile>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut result = Vec::new();
    for entry in statuses.iter() {
        let s = entry.status();
        // Only care about staged changes.
        if s.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            if let Some(path) = entry.head_to_index()
                .and_then(|diff| diff.new_file().path())
                .or_else(|| entry.index_to_workdir().and_then(|diff| diff.new_file().path()))
                .or_else(|| entry.index_to_workdir().and_then(|diff| diff.old_file().path()))
                .or_else(|| entry.head_to_index().and_then(|diff| diff.old_file().path()))
                .or_else(|| entry.index_to_workdir().and_then(|diff| diff.new_file().path()))
                .or_else(|| entry.path().map(Path::new))
            {
                let path_str = path.to_string_lossy().to_string();
                result.push(ChangedFile {
                    path: path_str,
                    status: s,
                });
            }
        }
    }

    Ok(result)
}

/// Get current branch name via libgit2.
fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    let shorthand = head
        .shorthand()
        .context("Cannot get branch shorthand")?;
    Ok(shorthand.to_string())
}

/// Open text in external editor (from $EDITOR or default).
fn edit_text_in_editor(initial: &str) -> Result<String> {
    let mut tmp = NamedTempFile::new()?;
    std::io::Write::write_all(&mut tmp, initial.as_bytes())?;
    tmp.flush()?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(editor)
        .arg(tmp.path())
        .status()
        .context("Failed to spawn editor")?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    let content = std::fs::read_to_string(tmp.path())?;
    Ok(content)
}

/// Execute git commit for a single group.
fn commit_group(repo_path: &Path, group: &ChangeGroup) -> Result<()> {
    let msg = group.full_message();
    let mut tmp = NamedTempFile::new()?;
    std::io::Write::write_all(&mut tmp, msg.as_bytes())?;
    tmp.flush()?;

    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_path);
    cmd.arg("commit");
    cmd.arg("-F");
    cmd.arg(tmp.path());

    // Restrict commit to files in this group.
    for f in &group.files {
        cmd.arg(&f.path);
    }

    let status = cmd.status().context("Failed to run git commit")?;
    if !status.success() {
        anyhow::bail!("git commit failed with status: {}", status);
    }

    Ok(())
}

/// Commit all groups one after another.
fn commit_all_groups(repo_path: &Path, groups: &[ChangeGroup]) -> Result<()> {
    for group in groups {
        commit_group(repo_path, group)?;
    }
    Ok(())
}

/// Event wrapper: input or tick.
enum Event<I> {
    Input(I),
    Tick,
}

/// Simple event loop to handle key events and ticks.
fn events_loop(tick_rate: Duration) -> impl Iterator<Item = Event<KeyEvent>> {
    struct EventIter {
        last_tick: Instant,
        tick_rate: Duration,
    }

    impl Iterator for EventIter {
        type Item = Event<KeyEvent>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let timeout = self
                    .tick_rate
                    .checked_sub(self.last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap_or(false) {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        return Some(Event::Input(key));
                    }
                }

                if self.last_tick.elapsed() >= self.tick_rate {
                    self.last_tick = Instant::now();
                    return Some(Event::Tick);
                }
            }
        }
    }

    EventIter {
        last_tick: Instant::now(),
        tick_rate,
    }
}

/// Draw the UI.
fn draw_ui<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &AppState) -> io::Result<()> {
    let _ = terminal.draw(|f| {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ]
                .as_ref(),
            )
            .split(size);

        // Left: group list.
        let groups_items: Vec<ListItem> = app
            .groups
            .iter()
            .enumerate()
            .map(|(idx, g)| {
                let header = g.header();
                let prefix = if idx == app.selected_index { "> " } else { "  " };
                let text = format!("{prefix}{header}");
                ListItem::new(Line::from(Span::raw(text)))
            })
            .collect();

        let groups_list = List::new(groups_items)
            .block(Block::default().title("Groups").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(groups_list, chunks[0]);

        // Right layout: message top, files bottom.
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(55),
                    Constraint::Percentage(35),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(chunks[1]);

        if let Some(group) = app.selected_group() {
            let msg = group.full_message();
            let paragraph = Paragraph::new(msg)
                .block(Block::default().title("Commit message").borders(Borders::ALL));
            f.render_widget(paragraph, right_chunks[0]);

            let files_lines: Vec<Line> = group
                .files
                .iter()
                .map(|f| Line::from(Span::raw(format!("- {}", f.path))))
                .collect();

            let files_paragraph = Paragraph::new(files_lines)
                .block(Block::default().title("Files").borders(Borders::ALL));
            f.render_widget(files_paragraph, right_chunks[1]);
        } else {
            let empty = Paragraph::new("No group selected")
                .block(Block::default().title("Commit message").borders(Borders::ALL));
            f.render_widget(empty, right_chunks[0]);
        }

        let status_paragraph = Paragraph::new(app.status_message.clone())
            .block(Block::default().title("Status").borders(Borders::ALL));
        f.render_widget(status_paragraph, right_chunks[2]);
    });
    Ok(())
}

/// Run the TUI event loop.
fn run_tui(mut app: AppState, repo_path: &Path) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(250);

    for evt in events_loop(tick_rate) {
        match evt {
            Event::Input(key) => match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Down => {
                    if !app.groups.is_empty() {
                        app.selected_index = (app.selected_index + 1).min(app.groups.len() - 1);
                    }
                }
                KeyCode::Up => {
                    if !app.groups.is_empty() {
                        if app.selected_index > 0 {
                            app.selected_index -= 1;
                        }
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(group) = app.selected_group_mut() {
                        // Temporarily leave raw mode.
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
                        terminal.show_cursor()?;

                        let edited = edit_text_in_editor(&group.full_message());

                        // Back to raw mode and alternate screen.
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;
                        terminal.hide_cursor()?;

                        match edited {
                            Ok(text) => {
                                group.set_from_commit_text(&text);
                                app.status_message = "Updated commit message from editor".to_string();
                            }
                            Err(e) => {
                                app.status_message =
                                    format!("Editor error: {e}. Keeping old message.");
                            }
                        }
                    }
                }
                KeyCode::Char('c') => {
                    if let Some(group) = app.selected_group() {
                        let result = commit_group(repo_path, group);
                        match result {
                            Ok(()) => {
                                app.status_message =
                                    "Committed selected group. You may quit now.".to_string();
                            }
                            Err(e) => {
                                app.status_message =
                                    format!("Commit failed: {e}");
                            }
                        }
                    }
                }
                KeyCode::Char('C') if key.modifiers.is_empty() => {
                    let result = commit_all_groups(repo_path, &app.groups);
                    match result {
                        Ok(()) => {
                            app.status_message =
                                "Committed all groups. You may quit now.".to_string();
                        }
                        Err(e) => {
                            app.status_message =
                                format!("Commit all failed: {e}");
                        }
                    }
                }
                KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+L clear status message.
                    app.status_message.clear();
                }
                _ => {}
            },
            Event::Tick => {
                // Nothing special on tick for now.
            }
        }

        draw_ui(&mut terminal, &app)?;
    }

    // Restore terminal state.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_path = cli
        .repo
        .unwrap_or_else(|| env::current_dir().expect("Cannot get current dir"));

    let repo = Repository::open(&repo_path)
        .with_context(|| format!("Not a git repository: {}", repo_path.display()))?;

    let branch = get_current_branch(&repo)?;
    let ticket = extract_ticket_from_branch(&branch);

    let staged_files = collect_staged_files(&repo)?;
    if staged_files.is_empty() {
        anyhow::bail!("No staged changes found. Stage files before running this tool.");
    }

    let groups = build_groups(staged_files, ticket);
    let app = AppState::new(groups);

    run_tui(app, &repo_path)?;

    Ok(())
}
