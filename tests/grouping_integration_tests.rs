//! Integration test for duplicate file detection in real grouping scenarios.

use commit_wizard::copilot::validate_no_duplicate_files;
use commit_wizard::inference::build_groups;
use commit_wizard::types::ChangedFile;
use git2::Status;

#[test]
fn test_heuristic_grouping_produces_no_duplicates() {
    // Create a realistic set of changed files
    let files = vec![
        ChangedFile {
            path: "src/api/users.rs".to_string(),
            status: Status::INDEX_MODIFIED,
        },
        ChangedFile {
            path: "src/api/posts.rs".to_string(),
            status: Status::INDEX_MODIFIED,
        },
        ChangedFile {
            path: "src/models/user.rs".to_string(),
            status: Status::INDEX_NEW,
        },
        ChangedFile {
            path: "src/ui/button.rs".to_string(),
            status: Status::INDEX_MODIFIED,
        },
        ChangedFile {
            path: "tests/api_tests.rs".to_string(),
            status: Status::INDEX_NEW,
        },
        ChangedFile {
            path: "README.md".to_string(),
            status: Status::INDEX_MODIFIED,
        },
        ChangedFile {
            path: ".github/workflows/ci.yml".to_string(),
            status: Status::INDEX_NEW,
        },
    ];

    // Build groups using heuristic inference
    let groups = build_groups(files, Some("PROJ-123".to_string()));

    // Verify no duplicates
    assert!(validate_no_duplicate_files(&groups).is_ok());

    // Verify all files are grouped
    let total_files: usize = groups.iter().map(|g| g.files.len()).sum();
    assert_eq!(total_files, 7, "All files should be grouped exactly once");
}

#[test]
fn test_heuristic_grouping_with_many_similar_files() {
    // Test with many files in same directory to ensure no duplicates
    let mut files = vec![];

    for i in 0..20 {
        files.push(ChangedFile {
            path: format!("src/api/endpoint_{}.rs", i),
            status: Status::INDEX_MODIFIED,
        });
    }

    for i in 0..15 {
        files.push(ChangedFile {
            path: format!("tests/test_{}.rs", i),
            status: Status::INDEX_NEW,
        });
    }

    let groups = build_groups(files.clone(), None);

    // Verify no duplicates
    assert!(validate_no_duplicate_files(&groups).is_ok());

    // Verify all files are accounted for
    let total_files: usize = groups.iter().map(|g| g.files.len()).sum();
    assert_eq!(
        total_files,
        files.len(),
        "All {} files should be grouped exactly once",
        files.len()
    );
}

#[test]
fn test_edge_case_single_file() {
    let files = vec![ChangedFile {
        path: "src/main.rs".to_string(),
        status: Status::INDEX_MODIFIED,
    }];

    let groups = build_groups(files, None);

    assert!(validate_no_duplicate_files(&groups).is_ok());
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].files.len(), 1);
}

#[test]
fn test_edge_case_empty_files() {
    let files = vec![];
    let groups = build_groups(files, None);

    assert!(validate_no_duplicate_files(&groups).is_ok());
    assert!(groups.is_empty());
}
