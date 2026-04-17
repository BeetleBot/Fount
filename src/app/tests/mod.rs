use super::*;
use crate::app::{App, AppMode, BufferState};
use crate::types::LineType;

use ratatui::style::{Color, Modifier};
use crossterm::event::{KeyCode, KeyModifiers};

    pub(crate) fn create_empty_app() -> App {
        let mut app = App::new(crate::config::Cli::default());
        app.config = crate::config::Config::default();
        
        // Tests expect an initial empty buffer in Normal mode
        let buf = BufferState {
            lines: vec![String::new()],
            ..Default::default()
        };
        app.buffers.push(buf);
        app.switch_buffer(0);
        
        app.mode = AppMode::Normal;
        app.theme = crate::theme::Theme::adaptive();
        app.update_layout();
        app
    }











































































    fn send_key_press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyEventState};
        let mut update_target_x = false;
        let mut text_changed = false;
        let mut cursor_moved = false;

        let ev = Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });

        let _ = app.handle_event(
            ev,
            &mut update_target_x,
            &mut text_changed,
            &mut cursor_moved,
        );
    }




















































    // ── Structural Locking (Production Mode) Tests ──────────────────────














mod editing;
mod navigation;
mod ui;
mod ux;
mod core;
mod integration;
