//! Git repository operations and utilities.
//!
//! This module provides functions for interacting with git repositories,
//! including collecting staged files, extracting branch information, and
//! executing commits.

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use git2::{Repository, Status, StatusOptions};
use regex::Regex;
use tempfile::NamedTempFile;

use crate::types::{ChangeGroup, ChangedFile};
use log::{debug, error};

/// Collects all changed files from the git repository (staged and unstaged).
///
/// This function collects:
/// - Modified files (staged or unstaged)
/// - New files (tracked or untracked)
/// - Deleted files
/// - Renamed files
///
/// # Arguments
///
/// * `repo` - A reference to the git repository
/// * `include_untracked` - Whether to include untracked (new) files
///
/// # Returns
///
/// A vector of [`ChangedFile`] representing all changes.
///
/// # Errors
///
/// Returns an error if the git status operation fails.
///
/// # Examples
///
/// ```no_run
/// use git2::Repository;
/// use commit_wizard::git::collect_changed_files;
///
/// let repo = Repository::open(".").unwrap();
/// let files = collect_changed_files(&repo, true).unwrap();
/// println!("Found {} changed files", files.len());
/// ```
pub fn collect_changed_files(
    repo: &Repository,
    include_untracked: bool,
) -> Result<Vec<ChangedFile>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(include_untracked)
        .include_ignored(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .context("Failed to get git status")?;

    let mut result = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();

        // Process both staged and unstaged changes, including untracked if requested
        let relevant_flags = if include_untracked {
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE
                | Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE
        } else {
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE
        };

        if !status.intersects(relevant_flags) {
            continue;
        }

        // Try multiple sources to get the file path
        let path = entry
            .head_to_index()
            .and_then(|diff| diff.new_file().path())
            .or_else(|| {
                entry
                    .index_to_workdir()
                    .and_then(|diff| diff.new_file().path())
            })
            .or_else(|| {
                entry
                    .index_to_workdir()
                    .and_then(|diff| diff.old_file().path())
            })
            .or_else(|| {
                entry
                    .head_to_index()
                    .and_then(|diff| diff.old_file().path())
            })
            .or_else(|| entry.path().map(Path::new));

        if let Some(path) = path {
            let path_str = path.to_string_lossy().to_string();

            // Validate path (security: prevent directory traversal)
            if is_valid_path(&path_str) {
                result.push(ChangedFile::new(path_str, status));
            }
        }
    }

    Ok(result)
}

/// Collects only untracked files that are not ignored by gitignore.
///
/// # Arguments
///
/// * `repo` - A reference to the git repository
///
/// # Returns
///
/// A vector of [`ChangedFile`] representing untracked files.
pub fn collect_untracked_files(repo: &Repository) -> Result<Vec<ChangedFile>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .context("Failed to get git status")?;

    let mut result = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();

        // Only collect untracked files (WT_NEW)
        if !status.contains(Status::WT_NEW) {
            continue;
        }

        if let Some(path) = entry.path() {
            let path_str = path.to_string();

            // Validate path (security: prevent directory traversal)
            if is_valid_path(&path_str) {
                result.push(ChangedFile::new(path_str, status));
            }
        }
    }

    Ok(result)
}

/// Validates that a path doesn't contain dangerous patterns.
///
/// # Security
///
/// This prevents directory traversal attacks and ensures paths are
/// relative to the repository root.
fn is_valid_path(path: &str) -> bool {
    // Reject absolute paths
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }

    // Reject paths with parent directory references
    if path.contains("..") {
        return false;
    }

    // Reject paths with null bytes
    if path.contains('\0') {
        return false;
    }

    // On Windows, reject paths with drive letters
    #[cfg(windows)]
    {
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            return false;
        }
    }

    true
}

/// Gets the git diff for a specific file.
///
/// # Arguments
///
/// * `repo` - A reference to the git repository
/// * `file_path` - The path to the file
///
/// # Returns
///
/// A string containing the diff output for the file.
///
/// # Errors
///
/// Returns an error if the diff operation fails.
pub fn get_file_diff(repo: &Repository, file_path: &str) -> Result<String> {
    let workdir = repo
        .workdir()
        .context("Repository has no working directory")?;

    let output = Command::new("git")
        .args(["diff", "--cached", "--", file_path])
        .current_dir(workdir)
        .output()
        .context("Failed to execute git diff")?;

    if !output.status.success() {
        bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Gets the current branch name from the repository.
///
/// # Arguments
///
/// * `repo` - A reference to the git repository
///
/// # Returns
///
/// The name of the current branch.
///
/// # Errors
///
/// Returns an error if the HEAD is detached or the branch name cannot be determined.
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head().context("Failed to get repository HEAD")?;

    let shorthand = head
        .shorthand()
        .context("Cannot get branch shorthand (HEAD may be detached)")?;

    Ok(shorthand.to_string())
}

/// Extracts a ticket reference from a branch name.
///
/// # Arguments
///
/// * `branch` - The branch name to parse
///
/// # Returns
///
/// The ticket reference if found (e.g., "LU-1234", "JIRA-456").
///
/// # Pattern
///
/// Matches uppercase letters followed by a dash and digits: `[A-Z]+-\d+`
///
/// # Examples
///
/// ```
/// use commit_wizard::git::extract_ticket_from_branch;
///
/// assert_eq!(
///     extract_ticket_from_branch("feature/LU-1234-add-login"),
///     Some("LU-1234".to_string())
/// );
/// assert_eq!(extract_ticket_from_branch("main"), None);
/// ```
pub fn extract_ticket_from_branch(branch: &str) -> Option<String> {
    let re = Regex::new(r"([A-Z]+-\d+)").ok()?;
    let caps = re.captures(branch)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// Commits a single change group to the repository.
///
/// Commits a single change group to the repository.
///
/// This function performs the following steps:
/// 1. Validates all file paths for security
/// 2. Stages the files in the group with `git add`
/// 3. Commits only those specific files with the group's message
///
/// # Behavior
///
/// This function will stage ALL files in the group, including files that were
/// previously unstaged. This is a behavior change from earlier versions that
/// relied on files already being staged. Users should be aware that running
/// this function will automatically stage any unstaged files in the group.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
/// * `group` - The change group to commit
///
/// # Errors
///
/// Returns an error if:
/// - Any file path in the group is invalid
/// - The git add (staging) command fails
/// - The commit message file cannot be created
/// - The git commit command fails
///
/// # Security
///
/// - Validates all file paths before passing to git
/// - Uses temporary files for commit messages
/// - Sets a timeout to prevent hanging
pub fn commit_group(repo_path: &Path, group: &ChangeGroup) -> Result<String> {
    // Validate all file paths first
    for file in &group.files {
        if !is_valid_path(&file.path) {
            bail!("Invalid file path: {}", file.path);
        }
    }

    // Stage the files in this group
    debug!("Staging {} file(s) for commit", group.files.len());

    let mut stage_cmd = Command::new("git");
    stage_cmd.arg("-C").arg(repo_path).arg("add").arg("--");

    for file in &group.files {
        stage_cmd.arg(&file.path);
    }

    let stage_output = execute_with_timeout(&mut stage_cmd, Duration::from_secs(10))
        .context("Failed to stage files")?;

    if !stage_output.status.success() {
        let stderr = String::from_utf8_lossy(&stage_output.stderr);
        error!("git add failed: {}", stderr);
        bail!("Failed to stage files: {}", stderr);
    }

    // Note: We stage files here to ensure all group files are committed,
    // even if they were previously unstaged. This is intentional behavior.

    // Create commit message
    let msg = group.full_message();
    let mut tmp = NamedTempFile::new().context("Failed to create temporary file")?;

    std::io::Write::write_all(&mut tmp, msg.as_bytes())
        .context("Failed to write commit message")?;
    tmp.flush().context("Failed to flush commit message")?;

    // Commit the staged files
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo_path)
        .arg("commit")
        .arg("-F")
        .arg(tmp.path())
        .arg("--");

    // Add specific files to this commit
    for file in &group.files {
        cmd.arg(&file.path);
    }

    // Execute with timeout for robustness
    debug!(
        "Committing group with args: {}\n",
        cmd.get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let output = execute_with_timeout(&mut cmd, Duration::from_secs(30))
        .context("Failed to execute git commit")?;

    // Capture both stdout and stderr for display
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    if !output.status.success() {
        error!("git commit failed: {}", stderr);
        bail!("git commit failed: {}", stderr);
    }

    Ok(combined_output)
}

/// Commits all change groups sequentially.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
/// * `groups` - Slice of change groups to commit
///
/// # Errors
///
/// Returns an error if any individual commit fails. Already committed
/// groups will remain committed; this function does not perform rollback.
pub fn commit_all_groups(repo_path: &Path, groups: &[ChangeGroup]) -> Result<()> {
    for (idx, group) in groups.iter().enumerate() {
        commit_group(repo_path, group)
            .with_context(|| format!("Failed to commit group {} of {}", idx + 1, groups.len()))?;
    }
    Ok(())
}

/// Executes a command with a timeout.
///
/// # Security & Robustness
///
/// This prevents commands from hanging indefinitely, which could:
/// - Freeze the UI
/// - Consume system resources
/// - Enable DoS attacks
fn execute_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<std::process::Output> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // Clone command for execution in thread
    let mut cmd_clone = Command::new(cmd.get_program());
    cmd_clone.args(cmd.get_args());
    if let Some(dir) = cmd.get_current_dir() {
        cmd_clone.current_dir(dir);
    }

    thread::spawn(move || {
        let result = cmd_clone.output();
        let _ = tx.send(result);
    });

    rx.recv_timeout(timeout)
        .context("Command execution timed out")?
        .context("Command execution failed")
}
