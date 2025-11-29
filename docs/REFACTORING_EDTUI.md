# Technical Specification: Editor Refactoring (External → edtui)

## Overview

Replace external editor spawning with integrated `edtui` TUI widget for in-app text editing.

## Current Implementation Analysis

### Current Flow (`src/editor.rs`)

```rust
pub fn edit_text_in_editor(initial_text: &str) -> Result<String> {
    1. Get editor from EDITOR env variable (fallback: vi)
    2. Validate editor command for security
    3. Create temporary file with initial text
    4. Spawn external editor process
    5. Wait for process completion
    6. Read back edited content
    7. Clean up temp file
}
```

**Problems:**
- Terminal raw mode must be disabled/re-enabled
- External process dependency (not testable)
- Security validation complexity
- Poor UX (context switch, different keybindings)
- Platform-specific issues (Windows editor paths)

### Usage in Code

1. **src/ui.rs** - `handle_edit_action()`:
   - Triggered by 'e' key
   - Disables raw mode
   - Calls `edit_text_in_editor()`
   - Re-enables raw mode
   - Updates commit message

2. **Tests** - 12 tests in `tests/editor_tests.rs`:
   - Editor validation tests
   - Path safety tests
   - Environment variable tests
   - **All need to be rewritten or removed**

## Target Implementation: edtui Integration

### Dependencies

```toml
[dependencies]
edtui = "0.9.9"
# Already have: ratatui = "0.29.0"
```

### Architecture Design

#### 1. New Editor State Structure

```rust
// src/editor.rs

use edtui::{EditorState, EditorView};
use ratatui::layout::Rect;
use ratatui::Frame;

pub struct CommitMessageEditor {
    /// Editor state (content, cursor position, etc.)
    state: EditorState,
    /// Whether editor is currently active
    active: bool,
    /// Original text before editing (for cancel operation)
    original_text: String,
}

impl CommitMessageEditor {
    pub fn new(initial_text: String) -> Self {
        let mut state = EditorState::default();
        state.set_text(&initial_text);

        Self {
            state,
            active: false,
            original_text: initial_text,
        }
    }

    /// Activate editor for editing
    pub fn activate(&mut self) {
        self.active = true;
        self.original_text = self.state.text().to_string();
    }

    /// Deactivate editor and save changes
    pub fn save(&mut self) -> String {
        self.active = false;
        self.state.text().to_string()
    }

    /// Deactivate editor and discard changes
    pub fn cancel(&mut self) {
        self.active = false;
        self.state.set_text(&self.original_text);
    }

    /// Render editor widget
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let view = EditorView::new(&mut self.state);
        f.render_widget(view, area);
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> EditorAction {
        // Map vim-style keys to actions
        match (key.code, key.modifiers) {
            // Save and close: Esc or Ctrl+S
            (KeyCode::Esc, _) | (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                EditorAction::Save
            }
            // Cancel: Ctrl+C or Ctrl+Q
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                EditorAction::Cancel
            }
            // Forward to edtui
            _ => {
                self.state.handle_key_event(key);
                EditorAction::Continue
            }
        }
    }
}

pub enum EditorAction {
    Save,
    Cancel,
    Continue,
}
```

#### 2. Integration in AppState

```rust
// src/types.rs

pub struct AppState {
    // ... existing fields ...

    /// Optional editor instance (None when not editing)
    pub editor: Option<CommitMessageEditor>,
}

impl AppState {
    /// Start editing the selected group's commit message
    pub fn start_editing(&mut self) {
        if let Some(group) = self.selected_group() {
            let initial_text = group.full_message();
            let mut editor = CommitMessageEditor::new(initial_text);
            editor.activate();
            self.editor = Some(editor);
        }
    }

    /// Check if editor is active
    pub fn is_editing(&self) -> bool {
        self.editor.as_ref().map_or(false, |e| e.active)
    }

    /// Save editor content to current group
    pub fn save_editor(&mut self) {
        if let Some(mut editor) = self.editor.take() {
            let edited_text = editor.save();
            if let Some(group) = self.selected_group_mut() {
                group.set_from_commit_text(&edited_text);
            }
        }
    }

    /// Cancel editing
    pub fn cancel_editor(&mut self) {
        if let Some(mut editor) = self.editor.take() {
            editor.cancel();
        }
    }
}
```

#### 3. UI Integration

```rust
// src/ui.rs

fn draw_ui<B: Backend>(terminal: &mut Terminal<B>, app: &AppState, ai_enabled: bool) {
    terminal.draw(|f| {
        let size = f.area();

        // If editor is active, show full-screen editor
        if app.is_editing() {
            if let Some(editor) = &mut app.editor {
                // Full screen or large centered overlay
                let editor_area = centered_rect(90, 90, size);

                // Clear area
                f.render_widget(Clear, editor_area);

                // Border with instructions
                let block = Block::default()
                    .title(" Edit Commit Message (Esc: Save, Ctrl+C: Cancel) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow));
                f.render_widget(block, editor_area);

                // Inner editor
                let inner = editor_area.inner(&Margin { horizontal: 1, vertical: 1 });
                editor.render(f, inner);
            }
            return;
        }

        // Normal UI rendering
        // ... existing code ...
    })?;
}

fn handle_key_event(...) -> Result<bool> {
    // If editor is active, route keys to editor
    if app.is_editing() {
        if let Some(editor) = &mut app.editor {
            match editor.handle_key(key) {
                EditorAction::Save => {
                    app.save_editor();
                }
                EditorAction::Cancel => {
                    app.cancel_editor();
                }
                EditorAction::Continue => {
                    // Continue editing
                }
            }
        }
        return Ok(false); // Don't quit
    }

    // Normal key handling
    match key.code {
        KeyCode::Char('e') => {
            app.start_editing();
        }
        // ... existing handlers ...
    }

    Ok(false)
}

// Helper for centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### Testing Strategy

#### Unit Tests (src/editor.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_initialization() {
        let editor = CommitMessageEditor::new("Initial text".to_string());
        assert!(!editor.active);
        assert_eq!(editor.state.text(), "Initial text");
    }

    #[test]
    fn test_editor_activation() {
        let mut editor = CommitMessageEditor::new("Test".to_string());
        editor.activate();
        assert!(editor.active);
    }

    #[test]
    fn test_editor_save() {
        let mut editor = CommitMessageEditor::new("Old".to_string());
        editor.state.set_text("New");
        let result = editor.save();
        assert_eq!(result, "New");
        assert!(!editor.active);
    }

    #[test]
    fn test_editor_cancel() {
        let mut editor = CommitMessageEditor::new("Original".to_string());
        editor.activate();
        editor.state.set_text("Modified");
        editor.cancel();
        assert_eq!(editor.state.text(), "Original");
        assert!(!editor.active);
    }

    #[test]
    fn test_editor_key_handling() {
        let mut editor = CommitMessageEditor::new("Test".to_string());

        // Test save key
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(editor.handle_key(key), EditorAction::Save));

        // Test cancel key
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(editor.handle_key(key), EditorAction::Cancel));
    }
}
```

#### Integration Tests (tests/editor_tests.rs)

```rust
// Most existing tests can be removed
// New tests focus on editor state management

#[test]
fn test_appstate_editor_workflow() {
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("test".to_string()),
        vec![],
        None,
        "initial message".to_string(),
        vec![],
    );
    let mut app = AppState::new(vec![group]);

    // Not editing initially
    assert!(!app.is_editing());

    // Start editing
    app.start_editing();
    assert!(app.is_editing());

    // Modify and save
    if let Some(editor) = &mut app.editor {
        editor.state.set_text("new message");
    }
    app.save_editor();

    assert!(!app.is_editing());
    assert!(app.selected_group().unwrap().description.contains("new message"));
}

#[test]
fn test_appstate_editor_cancel() {
    let group = ChangeGroup::new(
        CommitType::Fix,
        None,
        vec![],
        None,
        "original".to_string(),
        vec![],
    );
    let mut app = AppState::new(vec![group]);

    app.start_editing();
    if let Some(editor) = &mut app.editor {
        editor.state.set_text("modified");
    }
    app.cancel_editor();

    assert!(!app.is_editing());
    assert_eq!(app.selected_group().unwrap().description, "original");
}
```

## Migration Steps

### Phase 1: Preparation
1. ✅ Add `edtui = "0.9.9"` to Cargo.toml
2. ✅ Study edtui API documentation
3. ✅ Create feature branch: `feature/edtui-editor`

### Phase 2: Implementation
1. Implement `CommitMessageEditor` in src/editor.rs
2. Add `editor: Option<CommitMessageEditor>` to AppState
3. Implement `start_editing()`, `save_editor()`, `cancel_editor()` methods
4. Update UI to show editor overlay when active
5. Update event handling to route keys to editor

### Phase 3: Testing
1. Write unit tests for CommitMessageEditor
2. Write integration tests for AppState editor workflow
3. Manual testing of full edit flow
4. Test edge cases (empty message, very long text, Unicode)

### Phase 4: Cleanup
1. Remove old `edit_text_in_editor()` function
2. Remove `get_editor()` and `validate_editor_command()` functions
3. Remove EDITOR env variable handling
4. Remove all old editor-related tests (12 tests)
5. Update documentation

### Phase 5: Documentation
1. Update README with new editing instructions
2. Document vim-style keybindings
3. Update CHANGELOG

## Breaking Changes & Migration

### For Users

**Before:**
- Used EDITOR environment variable
- Spawned external editor (vim, nano, etc.)
- Required editor installed on system

**After:**
- Built-in vim-style editor
- No external dependencies
- Consistent experience across platforms

**Migration Guide:**
- EDITOR environment variable is ignored
- Learn basic vim commands: `i` (insert), `Esc` (save), `Ctrl+C` (cancel)
- No external editor installation needed

### For Developers

**Removed APIs:**
```rust
// REMOVED
pub fn edit_text_in_editor(initial_text: &str) -> Result<String>
pub fn get_editor() -> Result<String>
pub fn validate_editor_command(cmd: &str) -> Result<()>
```

**New APIs:**
```rust
// NEW
pub struct CommitMessageEditor { ... }
impl CommitMessageEditor {
    pub fn new(initial_text: String) -> Self
    pub fn activate(&mut self)
    pub fn save(&mut self) -> String
    pub fn cancel(&mut self)
    pub fn render(&mut self, f: &mut Frame, area: Rect)
    pub fn handle_key(&mut self, key: KeyEvent) -> EditorAction
}

// AppState extensions
impl AppState {
    pub fn start_editing(&mut self)
    pub fn is_editing(&self) -> bool
    pub fn save_editor(&mut self)
    pub fn cancel_editor(&mut self)
}
```

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Users prefer their own editor | High | Document clearly, consider adding toggle flag |
| Vim keybindings learning curve | Medium | Provide clear in-app help, support common keys |
| edtui bugs or limitations | Medium | Thorough testing, consider fallback option |
| Performance with large texts | Low | Test with large commit messages, optimize if needed |

## Timeline Estimate

- **Phase 1 (Preparation):** 1 hour
- **Phase 2 (Implementation):** 4-6 hours
- **Phase 3 (Testing):** 2-3 hours
- **Phase 4 (Cleanup):** 1-2 hours
- **Phase 5 (Documentation):** 1 hour

**Total:** ~9-13 hours

## Success Criteria

- ✅ Editor appears as full-screen overlay when 'e' pressed
- ✅ Text editing works smoothly with vim-style keys
- ✅ Esc saves changes, Ctrl+C cancels
- ✅ All tests pass
- ✅ Code coverage maintained (editor is testable)
- ✅ No external process spawning
- ✅ Consistent UX across platforms
