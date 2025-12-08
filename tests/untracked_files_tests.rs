//! Tests for untracked files handling

use commit_wizard::git::collect_untracked_files;
use git2::Repository;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_test_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    {
        // Set user.name and user.email (fix for CI/tests)
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Test User").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();

        // Create initial commit
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
            .unwrap();
    }

    (temp_dir, repo)
}

#[test]
fn test_collect_untracked_files_empty() {
    let (_temp_dir, repo) = setup_test_repo();

    let untracked = collect_untracked_files(&repo).unwrap();
    assert_eq!(untracked.len(), 0);
}

#[test]
fn test_collect_untracked_files_with_new_files() {
    let (temp_dir, repo) = setup_test_repo();

    // Create untracked files
    fs::write(temp_dir.path().join("untracked1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("untracked2.txt"), "content2").unwrap();

    let untracked = collect_untracked_files(&repo).unwrap();
    assert_eq!(untracked.len(), 2);

    let paths: Vec<String> = untracked.iter().map(|f| f.path.clone()).collect();
    assert!(paths.contains(&"untracked1.txt".to_string()));
    assert!(paths.contains(&"untracked2.txt".to_string()));
}

#[test]
fn test_collect_untracked_files_ignores_gitignored() {
    let (temp_dir, repo) = setup_test_repo();

    // Create .gitignore
    fs::write(temp_dir.path().join(".gitignore"), "ignored.txt\n*.log\n").unwrap();

    // Add .gitignore to index
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(".gitignore")).unwrap();
    index.write().unwrap();

    // Create ignored and non-ignored files
    fs::write(temp_dir.path().join("ignored.txt"), "should be ignored").unwrap();
    fs::write(temp_dir.path().join("test.log"), "should be ignored").unwrap();
    fs::write(temp_dir.path().join("visible.txt"), "should be visible").unwrap();

    let untracked = collect_untracked_files(&repo).unwrap();

    // Only visible.txt should be collected
    let paths: Vec<String> = untracked.iter().map(|f| f.path.clone()).collect();
    assert!(!paths.contains(&"ignored.txt".to_string()));
    assert!(!paths.contains(&"test.log".to_string()));
    assert!(paths.contains(&"visible.txt".to_string()));
}

#[test]
fn test_collect_untracked_files_nested_directories() {
    let (temp_dir, repo) = setup_test_repo();

    // Create nested directories with untracked files
    fs::create_dir_all(temp_dir.path().join("src/nested")).unwrap();
    fs::write(temp_dir.path().join("src/file1.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("src/nested/file2.txt"), "content").unwrap();

    let untracked = collect_untracked_files(&repo).unwrap();
    assert!(untracked.len() >= 2);

    let paths: Vec<String> = untracked.iter().map(|f| f.path.clone()).collect();
    assert!(paths.iter().any(|p| p.contains("file1.txt")));
    assert!(paths.iter().any(|p| p.contains("file2.txt")));
}

#[test]
fn test_collect_untracked_does_not_include_staged() {
    let (temp_dir, repo) = setup_test_repo();

    // Create and stage a file
    fs::write(temp_dir.path().join("staged.txt"), "content").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("staged.txt")).unwrap();
    index.write().unwrap();

    // Create an untracked file
    fs::write(temp_dir.path().join("untracked.txt"), "content").unwrap();

    let untracked = collect_untracked_files(&repo).unwrap();

    // Should only contain untracked.txt
    let paths: Vec<String> = untracked.iter().map(|f| f.path.clone()).collect();
    assert!(!paths.contains(&"staged.txt".to_string()));
    assert!(paths.contains(&"untracked.txt".to_string()));
}
