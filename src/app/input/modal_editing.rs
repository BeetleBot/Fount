use crate::app::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

impl App {
    // Returns Some(result) when the key is handled by the vim layer, None to
    // fall through to the regular editor handler.
    pub fn handle_vim_key(
        &mut self,
        key: KeyEvent,
        update_target_x: &mut bool,
        text_changed: &mut bool,
        cursor_moved: &mut bool,
    ) -> Option<io::Result<bool>> {
        if !self.config.modal_editing || self.mode != AppMode::Normal {
            return None;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        if self.vim_mode_insert {
            // Esc exits insert and nudges cursor left, just like vim.
            if key.code == KeyCode::Esc && !ctrl {
                self.vim_mode_insert = false;
                self.vim_pending_key = None;
                if self.cursor_x > 0 {
                    self.move_left();
                    *update_target_x = true;
                    *cursor_moved = true;
                }
                return Some(Ok(false));
            }
            // Every other key falls through to the normal insert handler.
            return None;
        }

        // Resolve pending operator (dd, dw, …).
        if let Some(pending) = self.vim_pending_key.take() {
            let handled = match (pending, &key.code) {
                ('d', KeyCode::Char('d')) if !ctrl => {
                    self.cut_line();
                    *update_target_x = true;
                    *text_changed = true;
                    *cursor_moved = true;
                    true
                }
                ('d', KeyCode::Char('w')) if !ctrl => {
                    self.delete_word_forward();
                    *update_target_x = true;
                    *text_changed = true;
                    *cursor_moved = true;
                    true
                }
                _ => false,
            };
            if handled {
                return Some(Ok(false));
            }
            // Unknown combo — pending is already cleared; fall through to
            // process the current key as a fresh command.
        }

        match key.code {
            // Navigation
            KeyCode::Char('h') if !ctrl => {
                self.clear_selection();
                self.move_left();
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('l') if !ctrl => {
                self.clear_selection();
                self.move_right();
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('j') if !ctrl => {
                self.clear_selection();
                self.move_down();
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('k') if !ctrl => {
                self.clear_selection();
                self.move_up();
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('w') if !ctrl => {
                self.clear_selection();
                self.move_word_right();
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('b') if !ctrl => {
                self.clear_selection();
                self.move_word_left();
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('e') if !ctrl => {
                self.clear_selection();
                let chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
                let max = chars.len();
                let mut x = self.cursor_x;
                if x < max && !chars[x].is_whitespace() {
                    x += 1;
                }
                while x < max && chars[x].is_whitespace() {
                    x += 1;
                }
                while x + 1 < max && !chars[x + 1].is_whitespace() {
                    x += 1;
                }
                if x < max {
                    self.cursor_x = x;
                }
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('0') if !ctrl => {
                self.clear_selection();
                self.move_home();
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('$') if !ctrl => {
                self.clear_selection();
                self.move_end();
                *update_target_x = true;
                *cursor_moved = true;
                Some(Ok(false))
            }

            // Enter insert sub-mode
            KeyCode::Char('i') if !ctrl => {
                self.vim_mode_insert = true;
                Some(Ok(false))
            }
            KeyCode::Char('I') if !ctrl => {
                self.move_home();
                *update_target_x = true;
                *cursor_moved = true;
                self.vim_mode_insert = true;
                Some(Ok(false))
            }
            KeyCode::Char('a') if !ctrl => {
                self.move_right();
                *update_target_x = true;
                *cursor_moved = true;
                self.vim_mode_insert = true;
                Some(Ok(false))
            }
            KeyCode::Char('A') if !ctrl => {
                self.move_end();
                *update_target_x = true;
                *cursor_moved = true;
                self.vim_mode_insert = true;
                Some(Ok(false))
            }
            KeyCode::Char('o') if !ctrl => {
                self.move_end();
                self.insert_newline(false);
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                self.vim_mode_insert = true;
                Some(Ok(false))
            }
            KeyCode::Char('O') if !ctrl => {
                self.cursor_x = 0;
                self.insert_newline(false);
                self.move_up();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                self.vim_mode_insert = true;
                Some(Ok(false))
            }

            // Editing without entering insert mode
            KeyCode::Char('x') if !ctrl => {
                self.delete_forward();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('p') if !ctrl => {
                self.move_right();
                self.paste_line();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('P') if !ctrl => {
                self.paste_line();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('d') if !ctrl => {
                self.vim_pending_key = Some('d');
                Some(Ok(false))
            }

            KeyCode::Char('u') if !ctrl => {
                self.undo();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                Some(Ok(false))
            }
            KeyCode::Char('U') if !ctrl => {
                self.redo();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
                Some(Ok(false))
            }

            // : opens the command panel (equivalent to / in regular mode).
            KeyCode::Char(':') if !ctrl => {
                self.previous_mode = self.mode;
                self.mode = AppMode::Command;
                self.command_input.clear();
                self.command_error = false;
                Some(Ok(false))
            }

            // Ctrl-chords and arrow keys fall through to the regular handler.
            KeyCode::Char(_) if !ctrl => Some(Ok(false)),
            _ => None,
        }
    }
}
