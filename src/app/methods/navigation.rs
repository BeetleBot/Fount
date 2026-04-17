use crate::app::{App, LastEdit, AppMode};
use crate::layout::{find_visual_cursor, build_layout};

impl App {
    pub fn update_search_regex(&mut self) {
        let active_query = if self.search_query.is_empty() {
            &self.last_search
        } else {
            &self.search_query
        };

        if active_query.is_empty() {
            self.compiled_search_regex = None;
        } else {
            self.compiled_search_regex = regex::RegexBuilder::new(&regex::escape(active_query))
                .case_insensitive(true)
                .build()
                .ok();
        }
    }

    pub fn report_cursor_position(&mut self) {
        if self.lines.is_empty() {
            self.set_status("line 1/1 (100%), col 1/1 (100%), char 1/1 (100%)");
            return;
        }

        let total_lines = self.lines.len().max(1);
        let cur_line = self.cursor_y + 1;
        let line_pct = (cur_line as f64 / total_lines as f64 * 100.0) as usize;

        let current_line_text = self
            .lines
            .get(self.cursor_y)
            .map(|s| s.as_str())
            .unwrap_or("");
        let total_cols = current_line_text.chars().count() + 1;
        let cur_col = self.cursor_x + 1;
        let col_pct = (cur_col as f64 / total_cols as f64 * 100.0) as usize;

        let total_chars: usize = self
            .lines
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            .max(1);

        let cur_char = self.lines[..self.cursor_y]
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            + self.cursor_x
            + 1;

        let char_pct = (cur_char as f64 / total_chars as f64 * 100.0) as usize;

        let msg = format!(
            "line {}/{} ({}%), col {}/{} ({}%), char {}/{} ({}%)",
            cur_line,
            total_lines,
            line_pct,
            cur_col,
            total_cols,
            col_pct,
            cur_char,
            total_chars,
            char_pct
        );
        self.set_status(&msg);
    }

    pub fn execute_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_query = self.last_search.clone();
        }
        if self.search_query.is_empty() {
            self.mode = AppMode::Normal;
            self.set_status("Cancelled");
            self.show_search_highlight = false;
            self.compiled_search_regex = None;
            return;
        }
        self.last_search = self.search_query.clone();
        self.update_search_regex();

        let re = self.compiled_search_regex.as_ref().unwrap();

        let mut wrapped = false;
        let mut found = false;
        let start_y = self.cursor_y;
        let start_char_x = self.cursor_x;

        for i in 0..=self.lines.len() {
            let y = (start_y + i) % self.lines.len();
            let line = &self.lines[y];

            for mat in re.find_iter(line) {
                let char_idx = line[..mat.start()].chars().count();

                if i == 0 && char_idx <= start_char_x {
                    continue;
                }

                if i == self.lines.len() && char_idx > start_char_x {
                    continue;
                }

                self.cursor_y = y;
                self.cursor_x = char_idx;
                found = true;

                if y < start_y || (y == start_y && i > 0) {
                    wrapped = true;
                }
                break;
            }
            if found {
                break;
            }
        }

        self.mode = AppMode::Normal;

        if !found {
            self.set_status(&format!("\"{}\" not found", self.search_query));
            self.show_search_highlight = false;
        } else {
            self.show_search_highlight = true;
            if wrapped {
                self.set_status("Search Wrapped");
            } else {
                self.clear_status();
            }
        }

        self.search_query.clear();
    }

    pub fn update_layout(&mut self) {
        self.layout = build_layout(
            &self.lines,
            &self.types,
            self.cursor_y,
            &self.config,
            &self.theme,
        );
    }

    pub fn move_up(&mut self) {
        self.last_edit = LastEdit::Other;
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row > 0 {
            let mut target_vi = vis_row - 1;
            while target_vi > 0 && self.layout[target_vi].is_phantom {
                target_vi -= 1;
            }
            self.jump_to_visual_row(target_vi, Some(false));
        } else {
            self.cursor_y = 0;
            self.cursor_x = 0;
        }
    }

    pub fn move_down(&mut self) {
        self.last_edit = LastEdit::Other;
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row + 1 < self.layout.len() {
            let mut target_vi = vis_row + 1;
            while target_vi + 1 < self.layout.len() && self.layout[target_vi].is_phantom {
                target_vi += 1;
            }
            self.jump_to_visual_row(target_vi, Some(true));
        } else {
            self.cursor_y = self.lines.len().saturating_sub(1);
            self.cursor_x = self.line_len(self.cursor_y);
        }
    }

    pub fn move_left(&mut self) {
        self.last_edit = LastEdit::Other;
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.line_len(self.cursor_y);
        }
    }

    pub fn move_right(&mut self) {
        self.last_edit = LastEdit::Other;
        let max = self.line_len(self.cursor_y);
        if self.cursor_x < max {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn move_word_left(&mut self) {
        self.last_edit = LastEdit::Other;
        if self.cursor_x == 0 {
            self.move_left();
            return;
        }
        let chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
        while self.cursor_x > 0 && chars[self.cursor_x - 1].is_whitespace() {
            self.cursor_x -= 1;
        }
        while self.cursor_x > 0 && !chars[self.cursor_x - 1].is_whitespace() {
            self.cursor_x -= 1;
        }
    }

    pub fn move_word_right(&mut self) {
        self.last_edit = LastEdit::Other;
        let chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
        let max = chars.len();
        if self.cursor_x == max {
            self.move_right();
            return;
        }
        while self.cursor_x < max && chars[self.cursor_x].is_whitespace() {
            self.cursor_x += 1;
        }
        while self.cursor_x < max && !chars[self.cursor_x].is_whitespace() {
            self.cursor_x += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.last_edit = LastEdit::Other;
        self.cursor_x = 0;
    }

    pub fn move_end(&mut self) {
        self.last_edit = LastEdit::Other;
        self.cursor_x = self.line_len(self.cursor_y);
    }

    pub fn move_page_up(&mut self) {
        self.last_edit = LastEdit::Other;
        let height = self.visible_height.max(1);
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row > 0 {
            let mut target_vi = vis_row.saturating_sub(height);
            while target_vi > 0 && self.layout[target_vi].is_phantom {
                target_vi -= 1;
            }
            self.jump_to_visual_row(target_vi, None);
        } else {
            self.cursor_y = 0;
            self.cursor_x = 0;
        }
    }

    pub fn move_page_down(&mut self) {
        self.last_edit = LastEdit::Other;
        let height = self.visible_height.max(1);
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row + 1 < self.layout.len() {
            let mut target_vi = (vis_row + height).min(self.layout.len().saturating_sub(1));
            while target_vi + 1 < self.layout.len() && self.layout[target_vi].is_phantom {
                target_vi += 1;
            }
            self.jump_to_visual_row(target_vi, None);
        } else {
            self.cursor_y = self.lines.len().saturating_sub(1);
            self.cursor_x = self.line_len(self.cursor_y);
        }
    }

    pub fn jump_to_visual_row(&mut self, target_vi: usize, snap_edge: Option<bool>) {
        let target_line_idx = self.layout[target_vi].line_idx;
        let changed_line = self.cursor_y != target_line_idx;

        let mut offset = 0;
        for i in (0..target_vi).rev() {
            if self.layout[i].line_idx == target_line_idx && !self.layout[i].is_phantom {
                offset += 1;
            } else if self.layout[i].line_idx != target_line_idx {
                break;
            }
        }

        self.cursor_y = target_line_idx;
        let mut final_vi = target_vi;

        if changed_line {
            self.update_layout();

            let new_rows: Vec<usize> = self
                .layout
                .iter()
                .enumerate()
                .filter(|(_, r)| !r.is_phantom && r.line_idx == target_line_idx)
                .map(|(i, _)| i)
                .collect();

            if !new_rows.is_empty() {
                if let Some(moving_down) = snap_edge {
                    if moving_down {
                        final_vi = *new_rows.first().unwrap();
                    } else {
                        final_vi = *new_rows.last().unwrap();
                    }
                } else {
                    final_vi = new_rows[offset.min(new_rows.len().saturating_sub(1))];
                }
            }
        }

        if final_vi < self.layout.len() {
            let target_row = &self.layout[final_vi];
            let is_last = target_row.char_end == self.line_len(target_row.line_idx);
            self.cursor_x = target_row
                .visual_to_logical_x(self.target_visual_x, is_last)
                .min(self.line_len(self.cursor_y));
        }
    }
}


impl crate::app::App {
    pub fn current_visual_x(&self) -> u16 {
        let (_, vis_x) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        vis_x
    }
}
