use commit_wizard::editor::CommitMessageEditor;
use ratatui::crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_new_editor_with_text() {
    let editor = CommitMessageEditor::new("Hello\nWorld".to_string());
    assert!(!editor.is_active());
    assert_eq!(editor.text(), "Hello\nWorld");
}

#[test]
fn test_empty_editor() {
    let editor = CommitMessageEditor::empty();
    assert!(!editor.is_active());
    assert_eq!(editor.text(), "");
}

#[test]
fn test_activate_deactivate() {
    let mut editor = CommitMessageEditor::empty();
    assert!(!editor.is_active());

    editor.activate("test".to_string());
    assert!(editor.is_active());
    assert_eq!(editor.text(), "test");

    editor.deactivate();
    assert!(!editor.is_active());
}

#[test]
fn test_set_text() {
    let mut editor = CommitMessageEditor::empty();
    editor.set_text("New content".to_string());
    assert_eq!(editor.text(), "New content");
}

#[test]
fn test_cancel_restores_original() {
    let mut editor = CommitMessageEditor::new("original".to_string());
    editor.activate("original".to_string());

    // Simulate some edits by setting new state
    editor.set_text("modified".to_string());
    editor.activate("modified".to_string());

    editor.cancel();
    assert!(!editor.is_active());
    assert_eq!(editor.text(), "modified"); // State is restored to what was activated
}

#[test]
fn test_handle_event_ctrl_s() {
    let mut editor = CommitMessageEditor::new("test".to_string());
    editor.activate("test".to_string());

    let event = CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    let result = editor.handle_event(event);

    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false to signal exit
    assert!(!editor.is_active());
}

#[test]
fn test_handle_event_ctrl_c() {
    let mut editor = CommitMessageEditor::new("original".to_string());
    editor.activate("original".to_string());

    let event = CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    let result = editor.handle_event(event);

    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false to signal cancel
    assert!(!editor.is_active());
}

#[test]
fn test_handle_event_normal_char() {
    let mut editor = CommitMessageEditor::new("test".to_string());
    editor.activate("test".to_string());

    let event = CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    let result = editor.handle_event(event);

    assert!(result.is_ok());
    assert!(result.unwrap()); // Should return true to continue editing
    assert!(editor.is_active());
}

#[test]
fn test_state_access() {
    let mut editor = CommitMessageEditor::new("test".to_string());

    // Test immutable access
    let _state = editor.state();

    // Test mutable access
    let _state_mut = editor.state_mut();
}

#[test]
fn test_multiline_text() {
    let text = "Line 1\nLine 2\nLine 3";
    let editor = CommitMessageEditor::new(text.to_string());
    assert_eq!(editor.text(), text);
}

#[test]
fn test_empty_lines() {
    let text = "Line 1\n\nLine 3";
    let editor = CommitMessageEditor::new(text.to_string());
    assert_eq!(editor.text(), text);
}
