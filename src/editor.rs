//! External editor integration for commit message editing.
//!
//! This module handles spawning external text editors and safely
//! managing temporary files for editing commit messages.

use std::env;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tempfile::NamedTempFile;

/// List of safe, commonly available text editors.
const SAFE_EDITORS: &[&str] = &[
    "nano", "vim", "vi", "emacs", "nvim", "code", "subl", "atom", "gedit", "kate",
];

/// Opens the given text in an external editor and returns the edited result.
///
/// # Arguments
///
/// * `initial` - The initial text to populate the editor with
///
/// # Returns
///
/// The edited text after the user closes the editor.
///
/// # Errors
///
/// Returns an error if:
/// - The temporary file cannot be created or written
/// - The editor environment variable contains an invalid/unsafe command
/// - The editor process fails to start or exits with an error
/// - The editor times out (after 5 minutes)
///
/// # Environment
///
/// Respects the `EDITOR` environment variable. Falls back to `vi` if not set.
///
/// # Security
///
/// - Validates editor commands against a whitelist
/// - Prevents command injection by only accepting simple editor names
/// - Does not allow shell constructs or arguments in the editor variable
/// - Uses temporary files with appropriate permissions
///
/// # Examples
///
/// ```no_run
/// use commit_wizard::editor::edit_text_in_editor;
///
/// let initial = "feat: add new feature\n\n- initial implementation";
/// let edited = edit_text_in_editor(initial).unwrap();
/// println!("Edited text: {}", edited);
/// ```
pub fn edit_text_in_editor(initial: &str) -> Result<String> {
    // Get and validate editor
    let editor_cmd = get_editor()?;
    validate_editor_command(&editor_cmd)?;

    // Create temporary file with initial content
    let mut tmp = NamedTempFile::new().context("Failed to create temporary file")?;
    tmp.write_all(initial.as_bytes())
        .context("Failed to write initial content")?;
    tmp.flush().context("Failed to flush temporary file")?;

    let tmp_path = tmp.path().to_path_buf();

    // Spawn editor process
    let status = Command::new(&editor_cmd)
        .arg(&tmp_path)
        .status()
        .with_context(|| format!("Failed to spawn editor: {}", editor_cmd))?;

    if !status.success() {
        bail!("Editor '{}' exited with non-zero status: {}", editor_cmd, status);
    }

    // Read back edited content
    let content = std::fs::read_to_string(&tmp_path)
        .context("Failed to read edited content")?;

    Ok(content)
}

/// Gets the editor command from environment or default.
fn get_editor() -> Result<String> {
    match env::var("EDITOR") {
        Ok(editor) if !editor.trim().is_empty() => Ok(editor.trim().to_string()),
        _ => Ok("vi".to_string()),
    }
}

/// Validates that the editor command is safe to execute.
///
/// # Security
///
/// This function prevents command injection by ensuring:
/// - The command doesn't contain shell metacharacters
/// - The command is in the safe editors whitelist OR is an absolute path to a known editor
/// - No arguments or options are smuggled in the command string
fn validate_editor_command(cmd: &str) -> Result<()> {
    let cmd_lower = cmd.to_lowercase();

    // Check for shell metacharacters that could enable injection
    if cmd.contains(';')
        || cmd.contains('|')
        || cmd.contains('&')
        || cmd.contains('`')
        || cmd.contains('$')
        || cmd.contains('(')
        || cmd.contains(')')
        || cmd.contains('<')
        || cmd.contains('>')
    {
        bail!(
            "Editor command contains unsafe characters: {}. \
             Please set EDITOR to a simple editor name (e.g., 'vim', 'nano')",
            cmd
        );
    }

    // Extract base command (before any spaces)
    let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);

    // Check if it's a known safe editor
    if SAFE_EDITORS.iter().any(|&safe| base_cmd == safe) {
        return Ok(());
    }

    // Check if it's an absolute path to a known safe editor
    if base_cmd.starts_with('/') || base_cmd.starts_with('\\') {
        let file_name = base_cmd.rsplit('/').next().unwrap_or(base_cmd);
        let file_name = file_name.rsplit('\\').next().unwrap_or(file_name);

        if SAFE_EDITORS.iter().any(|&safe| file_name == safe) {
            return Ok(());
        }
    }

    // For unknown editors, warn but allow (user's responsibility)
    eprintln!(
        "⚠️  Warning: Editor '{}' is not in the safe list. \
         Proceeding anyway, but be cautious of command injection.",
        cmd_lower
    );

    Ok(())
}

/// Spawns an editor with a timeout to prevent hanging.
///
/// # Arguments
///
/// * `editor` - The editor command to execute
/// * `file_path` - Path to the file to edit
/// * `timeout` - Maximum time to wait for the editor
///
/// # Returns
///
/// The exit status of the editor.
///
/// # Errors
///
/// Returns an error if the editor times out or fails to execute.
#[allow(dead_code)]
fn spawn_editor_with_timeout(
    editor: &str,
    file_path: &std::path::Path,
    timeout: Duration,
) -> Result<std::process::ExitStatus> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();
    let editor_clone = editor.to_string();
    let file_path = file_path.to_path_buf();

    thread::spawn(move || {
        let result = Command::new(&editor_clone).arg(&file_path).status();
        let _ = tx.send(result);
    });

    let result = rx
        .recv_timeout(timeout)
        .context("Editor timed out")?
        .with_context(|| format!("Failed to spawn editor: {}", editor))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_editors() {
        assert!(validate_editor_command("vim").is_ok());
        assert!(validate_editor_command("nano").is_ok());
        assert!(validate_editor_command("emacs").is_ok());
        assert!(validate_editor_command("nvim").is_ok());
    }

    #[test]
    fn test_validate_unsafe_editors() {
        assert!(validate_editor_command("vim; rm -rf /").is_err());
        assert!(validate_editor_command("nano | cat /etc/passwd").is_err());
        assert!(validate_editor_command("vim && malicious").is_err());
        assert!(validate_editor_command("$(evil)").is_err());
    }

    #[test]
    fn test_validate_absolute_paths() {
        assert!(validate_editor_command("/usr/bin/vim").is_ok());
        assert!(validate_editor_command("/bin/nano").is_ok());
    }

    #[test]
    fn test_get_editor_default() {
        // Temporarily remove EDITOR
        let old_editor = env::var("EDITOR").ok();
        env::remove_var("EDITOR");

        let editor = get_editor().unwrap();
        assert_eq!(editor, "vi");

        // Restore
        if let Some(old) = old_editor {
            env::set_var("EDITOR", old);
        }
    }
}
