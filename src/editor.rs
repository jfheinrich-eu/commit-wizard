//! Integrated text editor for commit message editing.
//!
//! This module provides an embedded text editor using the edtui widget,
//! eliminating the need for external editor processes.

use anyhow::Result;
use edtui::{EditorEventHandler, EditorState, Lines};
use ratatui::crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};

/// Editor for commit messages with vim-style keybindings.
///
/// This editor provides an integrated text editing experience without
/// spawning external processes. It uses vim-style navigation and editing.
#[derive(Clone)]
pub struct CommitMessageEditor {
    /// The edtui editor state managing the text buffer and cursor
    state: EditorState,
    /// Event handler for processing key events
    event_handler: EditorEventHandler,
    /// Original text for cancel functionality
    original_text: String,
    /// Whether the editor is currently active
    active: bool,
}

impl CommitMessageEditor {
    /// Creates a new editor with the given initial text.
    pub fn new(initial_text: String) -> Self {
        let state = EditorState::new(Lines::from(initial_text.as_str()));
        let event_handler = EditorEventHandler::default();

        Self {
            state,
            event_handler,
            original_text: initial_text,
            active: false,
        }
    }

    /// Creates an empty editor.
    pub fn empty() -> Self {
        Self::new(String::new())
    }

    /// Returns a reference to the editor state for rendering.
    pub fn state(&self) -> &EditorState {
        &self.state
    }

    /// Returns a mutable reference to the editor state for updates.
    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    /// Returns a reference to the event handler.
    pub fn event_handler_mut(&mut self) -> &mut EditorEventHandler {
        &mut self.event_handler
    }

    /// Gets the current text content.
    pub fn text(&self) -> String {
        // Convert Lines (Jagged<char>) to String using the From trait
        String::from(self.state.lines.clone())
    }

    /// Sets the text content.
    pub fn set_text(&mut self, text: String) {
        self.state = EditorState::new(Lines::from(text.as_str()));
        self.original_text = text;
    }

    /// Returns whether the editor is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates the editor with the given text.
    pub fn activate(&mut self, text: String) {
        self.original_text = text.clone();
        self.state = EditorState::new(Lines::from(text.as_str()));
        self.active = true;
    }

    /// Deactivates the editor.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Cancels editing and restores original text.
    pub fn cancel(&mut self) {
        self.state = EditorState::new(Lines::from(self.original_text.as_str()));
        self.deactivate();
    }

    pub fn save(&mut self) {
        self.state = EditorState::new(Lines::from(self.text().as_str()));
        self.original_text = self.text();
        self.deactivate();
    }

    /// Handles a crossterm event and returns whether it should exit.
    ///
    /// Returns `Ok(true)` to continue editing, `Ok(false)` to exit.
    /// Special keys:
    /// - Esc: Cancel without saving
    /// - Ctrl+C: Cancel without saving
    /// - Ctrl+S: Save and close
    pub fn handle_event(&mut self, event: CrosstermEvent) -> Result<bool> {
        // Check for exit keys first
        if let CrosstermEvent::Key(key) = event {
            match (key.code, key.modifiers) {
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    // Save and close
                    self.save();
                    return Ok(false);
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    // Cancel without saving
                    self.cancel();
                    return Ok(false);
                }
                _ => {}
            }
        }

        // Forward event to edtui handler
        self.event_handler.on_event(event, &mut self.state);
        Ok(true)
    }
}
