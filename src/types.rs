//! Core data types and structures for the commit wizard.
//!
//! This module defines the fundamental types used throughout the application,
//! including commit types, changed files, commit groups, and application state.

use git2::Status;

/// Conventional commit types following the Conventional Commits specification.
///
/// See: <https://www.conventionalcommits.org/>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommitType {
    /// A new feature
    Feat,
    /// A bug fix
    Fix,
    /// Documentation changes
    Docs,
    /// Code style changes (formatting, whitespace, etc.)
    Style,
    /// Code refactoring without changing functionality
    Refactor,
    /// Performance improvements
    Perf,
    /// Adding or updating tests
    Test,
    /// Maintenance tasks
    Chore,
    /// CI/CD pipeline changes
    Ci,
    /// Build system or dependency changes
    Build,
}

impl CommitType {
    /// Returns the conventional commit type prefix as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use commit_wizard::types::CommitType;
    ///
    /// assert_eq!(CommitType::Feat.as_str(), "feat");
    /// assert_eq!(CommitType::Fix.as_str(), "fix");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Feat => "feat",
            Self::Fix => "fix",
            Self::Docs => "docs",
            Self::Style => "style",
            Self::Refactor => "refactor",
            Self::Perf => "perf",
            Self::Test => "test",
            Self::Chore => "chore",
            Self::Ci => "ci",
            Self::Build => "build",
        }
    }

    /// Returns all available commit types.
    pub fn all() -> &'static [Self] {
        &[
            Self::Feat,
            Self::Fix,
            Self::Docs,
            Self::Style,
            Self::Refactor,
            Self::Perf,
            Self::Test,
            Self::Chore,
            Self::Ci,
            Self::Build,
        ]
    }
}

/// Represents a single file that has been changed in the git repository.
#[derive(Debug, Clone)]
pub struct ChangedFile {
    /// Relative path to the file from the repository root
    pub path: String,
    /// Git status flags for this file
    pub status: Status,
}

impl ChangedFile {
    /// Creates a new changed file entry.
    pub fn new(path: String, status: Status) -> Self {
        Self { path, status }
    }

    /// Checks if the file was newly added.
    pub fn is_new(&self) -> bool {
        self.status.is_index_new()
    }

    /// Checks if the file was modified.
    pub fn is_modified(&self) -> bool {
        self.status.is_index_modified()
    }

    /// Checks if the file was deleted.
    pub fn is_deleted(&self) -> bool {
        self.status.is_index_deleted()
    }

    /// Checks if the file was renamed.
    pub fn is_renamed(&self) -> bool {
        self.status.is_index_renamed()
    }
}

/// A logical group of changes representing a single potential commit.
///
/// Files are grouped by commit type and scope to create cohesive,
/// well-structured commits following conventional commit standards.
#[derive(Debug, Clone)]
pub struct ChangeGroup {
    /// The type of this commit (feat, fix, docs, etc.)
    pub commit_type: CommitType,
    /// Optional scope (e.g., component or module name)
    pub scope: Option<String>,
    /// Files included in this commit
    pub files: Vec<ChangedFile>,
    /// Optional ticket/issue reference (e.g., "LU-1234")
    pub ticket: Option<String>,
    /// Short description of the changes
    pub description: String,
    /// Detailed bullet points for the commit body
    pub body_lines: Vec<String>,
}

impl ChangeGroup {
    /// Maximum recommended length for a commit header line.
    pub const MAX_HEADER_LENGTH: usize = 72;

    /// Creates a new change group.
    pub fn new(
        commit_type: CommitType,
        scope: Option<String>,
        files: Vec<ChangedFile>,
        ticket: Option<String>,
        description: String,
        body_lines: Vec<String>,
    ) -> Self {
        Self {
            commit_type,
            scope,
            files,
            ticket,
            description,
            body_lines,
        }
    }

    /// Generates the commit message header line.
    ///
    /// Format: `<type>[(<scope>)]: <ticket>: <description>`
    ///
    /// The header is automatically truncated if it exceeds [`MAX_HEADER_LENGTH`].
    pub fn header(&self) -> String {
        let ctype = self.commit_type.as_str();
        let scope_part = self
            .scope
            .as_ref()
            .map(|s| format!("({})", s))
            .unwrap_or_default();
        let ticket_part = self
            .ticket
            .as_ref()
            .map(|t| format!("{}: ", t))
            .unwrap_or_default();

        let base_prefix = if scope_part.is_empty() {
            format!("{}: {}", ctype, ticket_part)
        } else {
            format!("{}{}: {}", ctype, scope_part, ticket_part)
        };

        let available_for_desc = Self::MAX_HEADER_LENGTH.saturating_sub(base_prefix.len());
        let mut desc = self.description.clone();

        if desc.len() > available_for_desc {
            desc.truncate(available_for_desc.saturating_sub(3));
            desc.push_str("...");
        }

        format!("{}{}", base_prefix, desc)
    }

    /// Generates the full commit message including header and body.
    ///
    /// # Format
    ///
    /// ```text
    /// <header>
    ///
    /// - <body line 1>
    /// - <body line 2>
    /// ```
    pub fn full_message(&self) -> String {
        let mut msg = String::new();
        msg.push_str(&self.header());

        if !self.body_lines.is_empty() {
            msg.push_str("\n\n");
            for line in &self.body_lines {
                msg.push_str("- ");
                msg.push_str(line);
                msg.push('\n');
            }
        }

        msg
    }

    /// Updates the group from user-edited commit text.
    ///
    /// Parses the first line as the new description and subsequent
    /// lines starting with "- " as body lines.
    pub fn set_from_commit_text(&mut self, text: &str) {
        let mut lines = text.lines();

        // Extract description from the first line
        if let Some(header) = lines.next() {
            let header_trimmed = header.trim();
            // Try to extract description after the last ": "
            if let Some(idx) = header_trimmed.rfind(": ") {
                self.description = header_trimmed[idx + 2..].trim().to_string();
            } else {
                // If no colon found, use entire header as description
                self.description = header_trimmed.to_string();
            }
        }

        // Extract body lines
        let mut body = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("- ") {
                body.push(stripped.to_string());
            } else if !trimmed.is_empty() {
                // Non-empty, non-bullet lines are treated as bullet items
                body.push(trimmed.to_string());
            }
        }
        self.body_lines = body;
    }
}

/// Application state for the terminal user interface.
pub struct AppState {
    /// All commit groups available for processing
    pub groups: Vec<ChangeGroup>,
    /// Index of the currently selected group
    pub selected_index: usize,
    /// Status message to display to the user
    pub status_message: String,
}

impl AppState {
    /// Creates a new application state with the given commit groups.
    pub fn new(groups: Vec<ChangeGroup>) -> Self {
        Self {
            groups,
            selected_index: 0,
            status_message: "↑/↓ select, e edit, c commit group, C commit all, q quit".to_string(),
        }
    }

    /// Returns a mutable reference to the currently selected group.
    pub fn selected_group_mut(&mut self) -> Option<&mut ChangeGroup> {
        self.groups.get_mut(self.selected_index)
    }

    /// Returns a reference to the currently selected group.
    pub fn selected_group(&self) -> Option<&ChangeGroup> {
        self.groups.get(self.selected_index)
    }

    /// Moves selection to the next group (wraps around).
    pub fn select_next(&mut self) {
        if !self.groups.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.groups.len();
        }
    }

    /// Moves selection to the previous group (wraps around).
    pub fn select_previous(&mut self) {
        if !self.groups.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.groups.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Sets the status message.
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    /// Clears the status message.
    pub fn clear_status(&mut self) {
        self.status_message.clear();
    }
}
