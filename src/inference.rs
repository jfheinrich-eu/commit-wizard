//! Inference logic for commit types, scopes, and descriptions.
//!
//! This module analyzes file paths and content to automatically determine
//! appropriate commit types, scopes, and generate helpful descriptions.

use std::collections::BTreeMap;

use crate::types::{ChangedFile, ChangeGroup, CommitType};

/// Infers the appropriate commit type based on file path heuristics.
///
/// # Arguments
///
/// * `path` - The file path to analyze
///
/// # Returns
///
/// The most appropriate [`CommitType`] based on the file path.
///
/// # Heuristics
///
/// - Files in `test/` or `tests/` directories → [`CommitType::Test`]
/// - `.md`, `.rst` files or `docs/` directory → [`CommitType::Docs`]
/// - `.github/`, `.gitlab/`, pipeline files → [`CommitType::Ci`]
/// - `Dockerfile`, `package.json`, build files → [`CommitType::Build`]
/// - `.css`, `.scss`, style files → [`CommitType::Style`]
/// - Default → [`CommitType::Feat`]
///
/// # Examples
///
/// ```
/// use commit_wizard::inference::infer_commit_type;
/// use commit_wizard::types::CommitType;
///
/// assert_eq!(infer_commit_type("src/main.rs"), CommitType::Feat);
/// assert_eq!(infer_commit_type("tests/unit_test.rs"), CommitType::Test);
/// assert_eq!(infer_commit_type("README.md"), CommitType::Docs);
/// ```
pub fn infer_commit_type(path: &str) -> CommitType {
    let lower = path.to_lowercase();

    // Test files
    if lower.contains("test") || lower.contains("spec") {
        return CommitType::Test;
    }

    // Documentation
    if is_documentation_file(&lower) {
        return CommitType::Docs;
    }

    // CI/CD
    if is_ci_file(&lower) {
        return CommitType::Ci;
    }

    // Build system
    if is_build_file(&lower) {
        return CommitType::Build;
    }

    // Styling
    if is_style_file(&lower) {
        return CommitType::Style;
    }

    // Default to feature
    CommitType::Feat
}

/// Checks if a file is a documentation file.
fn is_documentation_file(path: &str) -> bool {
    path.ends_with(".md")
        || path.ends_with(".rst")
        || path.ends_with(".txt")
        || path.ends_with(".adoc")
        || path.contains("/docs/")
        || path.starts_with("docs/")
        || path == "readme"
        || path == "changelog"
        || path == "contributing"
}

/// Checks if a file is a CI/CD configuration file.
fn is_ci_file(path: &str) -> bool {
    path.contains(".github")
        || path.contains(".gitlab")
        || path.contains("jenkins")
        || path.contains("pipeline")
        || path.ends_with("ci.yml")
        || path.ends_with("ci.yaml")
        || path.ends_with(".travis.yml")
        || path.contains("circleci")
        || path.contains("azure-pipelines")
}

/// Checks if a file is a build system file.
fn is_build_file(path: &str) -> bool {
    path.contains("dockerfile")
        || path.ends_with("package.json")
        || path.ends_with("package-lock.json")
        || path.ends_with("yarn.lock")
        || path.ends_with("pnpm-lock.yaml")
        || path.ends_with("composer.json")
        || path.ends_with("composer.lock")
        || path.ends_with("cargo.toml")
        || path.ends_with("cargo.lock")
        || path.ends_with("build.gradle")
        || path.ends_with("pom.xml")
        || path.ends_with("go.mod")
        || path.ends_with("go.sum")
        || path.contains("cmake")
        || path.contains("makefile")
}

/// Checks if a file is a styling file.
fn is_style_file(path: &str) -> bool {
    path.ends_with(".css")
        || path.ends_with(".scss")
        || path.ends_with(".sass")
        || path.ends_with(".less")
        || path.ends_with(".styl")
        || path.contains("/styles/")
        || path.contains("/css/")
}

/// Extracts a scope from a file path.
///
/// # Arguments
///
/// * `path` - The file path to analyze
///
/// # Returns
///
/// The first directory segment as the scope, or [`None`] if not applicable.
///
/// # Examples
///
/// ```
/// use commit_wizard::inference::infer_scope;
///
/// assert_eq!(infer_scope("src/main.rs"), Some("src".to_string()));
/// assert_eq!(infer_scope("backend/api/users.rs"), Some("backend".to_string()));
/// assert_eq!(infer_scope("README.md"), None);
/// ```
pub fn infer_scope(path: &str) -> Option<String> {
    let first_segment = path.split('/').next()?;

    // Filter out non-meaningful scopes
    if first_segment.is_empty()
        || first_segment == "."
        || first_segment.starts_with('.')
        || first_segment.to_lowercase().ends_with(".md")
    {
        return None;
    }

    Some(first_segment.to_string())
}

/// Generates a descriptive commit message based on the files and context.
///
/// # Arguments
///
/// * `files` - The files in this commit group
/// * `commit_type` - The type of commit
/// * `scope` - The optional scope
///
/// # Returns
///
/// A human-readable description string.
pub fn infer_description(
    files: &[ChangedFile],
    commit_type: CommitType,
    scope: &Option<String>,
) -> String {
    // Generate contextual descriptions based on commit type
    let action = match commit_type {
        CommitType::Feat => "add",
        CommitType::Fix => "fix",
        CommitType::Docs => "update",
        CommitType::Style => "format",
        CommitType::Refactor => "refactor",
        CommitType::Perf => "optimize",
        CommitType::Test => "update tests for",
        CommitType::Chore => "maintain",
        CommitType::Ci => "update CI for",
        CommitType::Build => "update build for",
    };

    if let Some(scope_value) = scope {
        format!("{} {}", action, scope_value)
    } else if files.len() == 1 {
        let file = &files[0];
        let file_name = file
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&file.path);
        format!("{} {}", action, file_name)
    } else {
        format!("{} {} files", action, files.len())
    }
}

/// Generates bullet points for the commit body based on the files.
///
/// # Arguments
///
/// * `files` - The files in this commit group
///
/// # Returns
///
/// A vector of strings representing commit body lines.
pub fn infer_body_lines(files: &[ChangedFile]) -> Vec<String> {
    const MAX_BODY_LINES: usize = 20;

    let mut lines: Vec<String> = files
        .iter()
        .take(MAX_BODY_LINES)
        .map(|f| {
            let action = if f.is_new() {
                "add"
            } else if f.is_deleted() {
                "remove"
            } else if f.is_modified() {
                "modify"
            } else if f.is_renamed() {
                "rename"
            } else {
                "update"
            };
            format!("{} {}", action, f.path)
        })
        .collect();

    // Add note if there are more files than shown
    if files.len() > MAX_BODY_LINES {
        lines.push(format!(
            "... and {} more files",
            files.len() - MAX_BODY_LINES
        ));
    }

    lines
}

/// Groups changed files into logical commit groups.
///
/// # Arguments
///
/// * `files` - All changed files to group
/// * `ticket` - Optional ticket reference to include in all commits
///
/// # Returns
///
/// A vector of [`ChangeGroup`]s, sorted by commit type and scope.
///
/// # Algorithm
///
/// 1. Infer commit type and scope for each file
/// 2. Group files with identical type and scope
/// 3. Generate descriptions and body lines for each group
/// 4. Sort groups deterministically
pub fn build_groups(files: Vec<ChangedFile>, ticket: Option<String>) -> Vec<ChangeGroup> {
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct GroupKey {
        commit_type: CommitType,
        scope: Option<String>,
    }

    let mut map: BTreeMap<GroupKey, Vec<ChangedFile>> = BTreeMap::new();

    // Group files by type and scope
    for file in files {
        let commit_type = infer_commit_type(&file.path);
        let scope = infer_scope(&file.path);
        let key = GroupKey { commit_type, scope };
        map.entry(key).or_default().push(file);
    }

    // Convert groups to ChangeGroup structs
    let mut groups: Vec<ChangeGroup> = map
        .into_iter()
        .map(|(key, group_files)| {
            let description = infer_description(&group_files, key.commit_type, &key.scope);
            let body_lines = infer_body_lines(&group_files);

            ChangeGroup::new(
                key.commit_type,
                key.scope.clone(),
                group_files,
                ticket.clone(),
                description,
                body_lines,
            )
        })
        .collect();

    // Sort by commit type for consistent ordering
    groups.sort_by_key(|g| g.commit_type);

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Status;

    #[test]
    fn test_infer_commit_type() {
        assert_eq!(infer_commit_type("src/main.rs"), CommitType::Feat);
        assert_eq!(infer_commit_type("tests/unit.rs"), CommitType::Test);
        assert_eq!(infer_commit_type("README.md"), CommitType::Docs);
        assert_eq!(infer_commit_type(".github/workflows/ci.yml"), CommitType::Ci);
        assert_eq!(infer_commit_type("Dockerfile"), CommitType::Build);
        assert_eq!(infer_commit_type("styles/main.css"), CommitType::Style);
    }

    #[test]
    fn test_infer_scope() {
        assert_eq!(infer_scope("src/main.rs"), Some("src".to_string()));
        assert_eq!(infer_scope("backend/api.rs"), Some("backend".to_string()));
        assert_eq!(infer_scope("README.md"), None);
        assert_eq!(infer_scope(".github/ci.yml"), None);
    }

    #[test]
    fn test_infer_description() {
        let files = vec![ChangedFile::new("src/main.rs".to_string(), Status::INDEX_NEW)];

        let desc = infer_description(&files, CommitType::Feat, &Some("src".to_string()));
        assert_eq!(desc, "add src");

        let desc = infer_description(&files, CommitType::Fix, &None);
        assert_eq!(desc, "fix main.rs");
    }

    #[test]
    fn test_build_groups() {
        let files = vec![
            ChangedFile::new("src/main.rs".to_string(), Status::INDEX_MODIFIED),
            ChangedFile::new("src/lib.rs".to_string(), Status::INDEX_NEW),
            ChangedFile::new("tests/test.rs".to_string(), Status::INDEX_NEW),
            ChangedFile::new("README.md".to_string(), Status::INDEX_MODIFIED),
        ];

        let groups = build_groups(files, Some("TICKET-123".to_string()));

        // Should create separate groups for docs, src code, and tests
        assert!(groups.len() >= 2);
        assert!(groups.iter().any(|g| g.commit_type == CommitType::Docs));
        assert!(groups.iter().any(|g| g.commit_type == CommitType::Test));
    }
}
