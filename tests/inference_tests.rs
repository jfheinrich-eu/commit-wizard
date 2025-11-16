//! Integration tests for the inference module.
//!
//! Tests commit type inference, scope extraction, description generation, and grouping logic.

use git2::Status;

// Import inference functions and types from the library
use commit_wizard::inference::{
    build_groups, infer_body_lines, infer_commit_type, infer_description, infer_scope,
};
use commit_wizard::types::{ChangedFile, CommitType};

#[test]
fn test_infer_commit_type_code_files() {
    assert_eq!(infer_commit_type("src/main.rs"), CommitType::Feat);
    assert_eq!(infer_commit_type("backend/api.rs"), CommitType::Feat);
    assert_eq!(infer_commit_type("lib/utils.rs"), CommitType::Feat);
}

#[test]
fn test_infer_commit_type_test_files() {
    assert_eq!(infer_commit_type("tests/unit.rs"), CommitType::Test);
    assert_eq!(infer_commit_type("test/integration.rs"), CommitType::Test);
    assert_eq!(infer_commit_type("src/api_test.rs"), CommitType::Test);
    assert_eq!(infer_commit_type("specs/user_spec.rs"), CommitType::Test);
}

#[test]
fn test_infer_commit_type_documentation() {
    assert_eq!(infer_commit_type("README.md"), CommitType::Docs);
    assert_eq!(infer_commit_type("CHANGELOG.md"), CommitType::Docs);
    assert_eq!(infer_commit_type("docs/guide.md"), CommitType::Docs);
    assert_eq!(infer_commit_type("docs/api.rst"), CommitType::Docs);
    assert_eq!(infer_commit_type("CONTRIBUTING.txt"), CommitType::Docs);
}

#[test]
fn test_infer_commit_type_ci_files() {
    assert_eq!(infer_commit_type(".github/workflows/ci.yml"), CommitType::Ci);
    assert_eq!(infer_commit_type(".gitlab-ci.yml"), CommitType::Ci);
    assert_eq!(infer_commit_type(".travis.yml"), CommitType::Ci);
    assert_eq!(infer_commit_type("jenkins/Jenkinsfile"), CommitType::Ci);
    assert_eq!(infer_commit_type("azure-pipelines.yml"), CommitType::Ci);
    assert_eq!(infer_commit_type(".circleci/config.yml"), CommitType::Ci);
}

#[test]
fn test_infer_commit_type_build_files() {
    assert_eq!(infer_commit_type("Dockerfile"), CommitType::Build);
    assert_eq!(infer_commit_type("package.json"), CommitType::Build);
    assert_eq!(infer_commit_type("package-lock.json"), CommitType::Build);
    assert_eq!(infer_commit_type("cargo.toml"), CommitType::Build);
    assert_eq!(infer_commit_type("cargo.lock"), CommitType::Build);
    assert_eq!(infer_commit_type("pom.xml"), CommitType::Build);
    assert_eq!(infer_commit_type("build.gradle"), CommitType::Build);
    assert_eq!(infer_commit_type("composer.json"), CommitType::Build);
    assert_eq!(infer_commit_type("go.mod"), CommitType::Build);
    // Note: CMakeLists.txt and Makefile match cmake/makefile patterns (contains check, case-insensitive)
    assert_eq!(infer_commit_type("CMakeLists.txt"), CommitType::Build);
    assert_eq!(infer_commit_type("Makefile"), CommitType::Build);
}

#[test]
fn test_infer_commit_type_style_files() {
    assert_eq!(infer_commit_type("styles/main.css"), CommitType::Style);
    assert_eq!(infer_commit_type("app.scss"), CommitType::Style);
    assert_eq!(infer_commit_type("theme.sass"), CommitType::Style);
    assert_eq!(infer_commit_type("layout.less"), CommitType::Style);
    assert_eq!(infer_commit_type("components.styl"), CommitType::Style);
    assert_eq!(infer_commit_type("css/normalize.css"), CommitType::Style);
}

#[test]
fn test_infer_scope_with_directory() {
    assert_eq!(infer_scope("src/main.rs"), Some("src".to_string()));
    assert_eq!(infer_scope("backend/api.rs"), Some("backend".to_string()));
    assert_eq!(infer_scope("frontend/ui.js"), Some("frontend".to_string()));
    assert_eq!(infer_scope("docs/guide.md"), Some("docs".to_string()));
}

#[test]
fn test_infer_scope_without_directory() {
    // Files without directory prefix get their filename as "scope" but are filtered
    // if they end with .md or are top-level files
    assert_eq!(infer_scope("README.md"), None); // Filtered: ends with .md
    assert_eq!(infer_scope("LICENSE"), Some("LICENSE".to_string())); // Has no extension
    // Note: Implementation returns first segment even for top-level files
}

#[test]
fn test_infer_scope_hidden_directories() {
    assert_eq!(infer_scope(".github/workflows/ci.yml"), None);
    assert_eq!(infer_scope(".vscode/settings.json"), None);
}

#[test]
fn test_infer_scope_nested_paths() {
    // Should only return first segment
    assert_eq!(infer_scope("src/api/users.rs"), Some("src".to_string()));
    assert_eq!(infer_scope("backend/db/schema.sql"), Some("backend".to_string()));
}

#[test]
fn test_infer_description_with_scope() {
    let files = vec![ChangedFile::new("src/main.rs".to_string(), Status::INDEX_NEW)];

    assert_eq!(
        infer_description(&files, CommitType::Feat, &Some("src".to_string())),
        "add src"
    );
    assert_eq!(
        infer_description(&files, CommitType::Fix, &Some("api".to_string())),
        "fix api"
    );
    assert_eq!(
        infer_description(&files, CommitType::Test, &Some("backend".to_string())),
        "update tests for backend"
    );
    assert_eq!(
        infer_description(&files, CommitType::Docs, &Some("readme".to_string())),
        "update readme" // Docs type uses "update" not "update docs for"
    );
}

#[test]
fn test_infer_description_single_file_no_scope() {
    let files = vec![ChangedFile::new("README.md".to_string(), Status::INDEX_MODIFIED)];

    assert_eq!(
        infer_description(&files, CommitType::Docs, &None),
        "update README.md"
    );
}

#[test]
fn test_infer_description_multiple_files() {
    let files = vec![
        ChangedFile::new("file1.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("file2.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("file3.rs".to_string(), Status::INDEX_NEW),
    ];

    assert_eq!(
        infer_description(&files, CommitType::Feat, &None),
        "add 3 files"
    );
}

#[test]
fn test_infer_body_lines() {
    let files = vec![
        ChangedFile::new("src/main.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("src/lib.rs".to_string(), Status::INDEX_MODIFIED),
        ChangedFile::new("tests/test.rs".to_string(), Status::INDEX_DELETED),
    ];

    let body_lines = infer_body_lines(&files);

    assert_eq!(body_lines.len(), 3);
    assert!(body_lines[0].contains("add") && body_lines[0].contains("src/main.rs"));
    assert!(body_lines[1].contains("modify") && body_lines[1].contains("src/lib.rs"));
    assert!(body_lines[2].contains("remove") && body_lines[2].contains("tests/test.rs"));
}

#[test]
fn test_infer_body_lines_truncation() {
    // Create more than 20 files
    let files: Vec<ChangedFile> = (0..25)
        .map(|i| ChangedFile::new(format!("file{}.rs", i), Status::INDEX_NEW))
        .collect();

    let body_lines = infer_body_lines(&files);

    // Should be truncated to 20 + 1 summary line
    assert_eq!(body_lines.len(), 21);
    assert!(body_lines[20].contains("and 5 more files"));
}

#[test]
fn test_build_groups_single_type() {
    let files = vec![
        ChangedFile::new("src/main.rs".to_string(), Status::INDEX_MODIFIED),
        ChangedFile::new("src/lib.rs".to_string(), Status::INDEX_NEW),
    ];

    let groups = build_groups(files, None);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].commit_type, CommitType::Feat);
    assert_eq!(groups[0].scope, Some("src".to_string()));
    assert_eq!(groups[0].files.len(), 2);
}

#[test]
fn test_build_groups_multiple_types() {
    let files = vec![
        ChangedFile::new("src/main.rs".to_string(), Status::INDEX_MODIFIED),
        ChangedFile::new("tests/test.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("README.md".to_string(), Status::INDEX_MODIFIED),
    ];

    let groups = build_groups(files, Some("TICKET-123".to_string()));

    // Should have multiple groups
    assert!(groups.len() >= 2);
    
    // Verify we have different commit types
    let has_docs = groups.iter().any(|g| g.commit_type == CommitType::Docs);
    let has_test = groups.iter().any(|g| g.commit_type == CommitType::Test);
    assert!(has_docs || has_test, "Should have docs or test commits");
    
    // All should have the ticket
    assert_eq!(groups[0].ticket, Some("TICKET-123".to_string()));
    assert_eq!(groups[1].ticket, Some("TICKET-123".to_string()));
    assert_eq!(groups[2].ticket, Some("TICKET-123".to_string()));
}

#[test]
fn test_build_groups_same_type_different_scopes() {
    let files = vec![
        ChangedFile::new("src/main.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("backend/api.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("frontend/ui.js".to_string(), Status::INDEX_NEW),
    ];

    let groups = build_groups(files, None);

    // Should create 3 groups (different scopes)
    assert_eq!(groups.len(), 3);
    
    let scopes: Vec<_> = groups.iter().map(|g| g.scope.as_ref()).collect();
    assert!(scopes.contains(&Some(&"src".to_string())));
    assert!(scopes.contains(&Some(&"backend".to_string())));
    assert!(scopes.contains(&Some(&"frontend".to_string())));
}

#[test]
fn test_build_groups_empty() {
    let groups = build_groups(vec![], None);
    assert!(groups.is_empty());
}

#[test]
fn test_build_groups_ordering_deterministic() {
    let files = vec![
        ChangedFile::new("Dockerfile".to_string(), Status::INDEX_NEW),
        ChangedFile::new("src/main.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("tests/test.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("README.md".to_string(), Status::INDEX_NEW),
        ChangedFile::new(".github/ci.yml".to_string(), Status::INDEX_NEW),
    ];

    let groups1 = build_groups(files.clone(), None);
    let groups2 = build_groups(files, None);

    // Should produce identical ordering
    assert_eq!(groups1.len(), groups2.len());
    for (g1, g2) in groups1.iter().zip(groups2.iter()) {
        assert_eq!(g1.commit_type, g2.commit_type);
        assert_eq!(g1.scope, g2.scope);
    }
}
