use crate::app::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

impl App {
    pub fn handle_command(&mut self, key: KeyEvent, update_target_x: &mut bool, text_changed: &mut bool, cursor_moved: &mut bool) -> io::Result<bool> {
        let _ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let _shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match self.mode {
                AppMode::Command => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = self.previous_mode;
                            self.command_input.clear();
                            self.command_error = false;
                        }
                        KeyCode::Tab => {
                            let commands = self.get_command_completions();
                            let matches: Vec<&String> = commands.iter()
                                .filter(|c| c.starts_with(&self.command_input))
                                .collect();
                            
                            if !matches.is_empty() {
                                // Basic cycling
                                let current = &self.command_input;
                                if let Some(pos) = matches.iter().position(|m| *m == current) {
                                    self.command_input = matches[(pos + 1) % matches.len()].to_string();
                                } else {
                                    self.command_input = matches[0].to_string();
                                }
                            }
                        }
                        KeyCode::Right => {
                            if !self.command_input.is_empty() {
                                let commands = self.get_command_completions();
                                if let Some(first_match) = commands.iter().find(|&c| c.starts_with(&self.command_input) && c != &self.command_input) {
                                    self.command_input = first_match.to_string();
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            self.command_input.pop();
                            if self.command_input.is_empty() {
                                self.mode = self.previous_mode;
                            }
                            self.command_error = false;
                        }
                        KeyCode::Enter => {
                            if self.execute_command(text_changed, cursor_moved, update_target_x)? {
                                return Ok(true);
                            }
                        }
                        KeyCode::Char(c) => {
                            self.command_input.push(c);
                            self.command_error = false;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
            _ => {}
        }
        Ok(false)
    }
}
