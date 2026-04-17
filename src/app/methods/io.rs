use std::{fs, io};
use std::path::PathBuf;
use crate::app::App;

impl App {
    pub fn emergency_save(&mut self) {
        let mut to_save = Vec::new();
        to_save.push((self.file.clone(), &self.lines, self.dirty));

        for (i, buf) in self.buffers.iter().enumerate() {
            if i != self.current_buf_idx {
                to_save.push((buf.file.clone(), &buf.lines, buf.dirty));
            }
        }

        for (file, lines, dirty) in to_save {
            if !dirty || lines.is_empty() || (lines.len() == 1 && lines[0].is_empty()) {
                continue;
            }

            let dir = file
                .as_ref()
                .and_then(|p| p.parent())
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let base_name = file
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "fount".to_string());

            let mut backup_path = dir.join(format!("{}.save", base_name));
            let mut counter = 1;

            while backup_path.exists() && counter <= 1000 {
                backup_path = dir.join(format!("{}.save.{}", base_name, counter));
                counter += 1;
            }

            if counter <= 1000 {
                let content = lines.join("\n");
                let _ = std::fs::write(&backup_path, content);
            }
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some(msg.to_string());
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn load_recent_files(&mut self) {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Fount") {
            let path = proj_dirs.data_dir().join("recent.txt");
            if let Ok(content) = fs::read_to_string(path) {
                self.recent_files = content
                    .lines()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect();
            }
        }
    }

    pub fn save_recent_files(&self) {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Fount") {
            let path = proj_dirs.data_dir().join("recent.txt");
            let content = self.recent_files
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n");
            let _ = fs::write(path, content);
        }
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        let path = path.canonicalize().unwrap_or(path);
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
        self.save_recent_files();
    }

    pub fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        if self.is_tutorial {
            self.set_status("Cannot save the tutorial buffer. Press Ctrl+X to exit.");
            return Ok(());
        }
        let content = self.lines.join("\n");
        fs::write(&path, content)?;
        self.file = Some(path.clone());
        self.dirty = false;
        self.add_recent_file(path);
        self.set_status(&format!(
            "Saved as {}",
            self.file.as_ref().unwrap().display()
        ));
        Ok(())
    }

    pub fn export_fountain(&self, path: &std::path::Path) -> std::io::Result<()> {
        let content = self.lines.join("\n");
        std::fs::write(path, content)
    }

    pub fn export_scene_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Scene Number,Int/Ext,Location,Time,Estimated Length (8ths)\n");

        let mut current_scene = None;
        let mut scene_lines = 0;
        let mut scenes_data = Vec::new();

        for row in &self.layout {
            if row.line_type == crate::types::LineType::SceneHeading {
                if let Some((s_num, heading)) = current_scene.take() {
                    scenes_data.push((s_num, heading, scene_lines));
                }

                let s_num = row.scene_num.clone().unwrap_or_default();
                let heading = crate::layout::strip_sigils(&row.raw_text, row.line_type).to_string();
                current_scene = Some((s_num, heading));
                scene_lines = 1;
            } else if current_scene.is_some() {
                scene_lines += 1;
            }
        }

        if let Some((s_num, heading)) = current_scene.take() {
            scenes_data.push((s_num, heading, scene_lines));
        }

        for (s_num, heading, visual_lines) in scenes_data {
            let eights_total = visual_lines as f32 / 7.0;
            let eights_rounded = eights_total.round() as usize;

            let full_pages = eights_rounded / 8;
            let remaining_eighths = eights_rounded % 8;

            let length_str = if full_pages > 0 && remaining_eighths > 0 {
                format!("{} {}/8", full_pages, remaining_eighths)
            } else if full_pages > 0 {
                format!("{}", full_pages)
            } else if remaining_eighths > 0 {
                format!("{}/8", remaining_eighths)
            } else {
                "1/8".to_string()
            };

            let mut int_ext = String::new();
            let loc;
            let mut time = String::new();
            let h = heading.to_uppercase();
            if let Some((ie, rest)) = h.split_once('.') {
                int_ext = ie.trim().to_string();
                if let Some((l, t)) = rest.split_once('-') {
                    loc = l.trim().to_string();
                    time = t.trim().to_string();
                } else {
                    loc = rest.trim().to_string();
                }
            } else {
                loc = h;
            }

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                s_num, int_ext, loc, time, length_str
            ));
        }

        std::fs::write(path, csv)
    }

    pub fn export_character_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Character,Dialogue Words,Scenes\n");

        let mut char_word_counts = std::collections::HashMap::new();
        let mut char_scenes = std::collections::HashMap::new();

        let mut current_scene = String::new();
        let mut current_char = String::new();

        for row in &self.layout {
            match row.line_type {
                crate::types::LineType::SceneHeading => {
                    if let Some(snum) = &row.scene_num {
                        current_scene = snum.clone();
                    } else {
                        current_scene = String::new();
                    }
                }
                crate::types::LineType::Character
                | crate::types::LineType::DualDialogueCharacter => {
                    let mut name = crate::layout::strip_sigils(&row.raw_text, row.line_type)
                        .trim()
                        .to_string();
                    if let Some(idx) = name.find('(') {
                        name = name[..idx].trim().to_string(); // Strip (V.O.) and (CONT'D)
                    }
                    current_char = name.to_uppercase();
                    if !current_scene.is_empty() {
                        let scenes: &mut std::collections::HashSet<String> =
                            char_scenes.entry(current_char.clone()).or_default();
                        scenes.insert(current_scene.clone());
                    }
                }
                crate::types::LineType::Dialogue => {
                    let text = crate::layout::strip_sigils(&row.raw_text, row.line_type);
                    if !current_char.is_empty() {
                        let words = text.split_whitespace().count();
                        *char_word_counts.entry(current_char.clone()).or_insert(0) += words;
                    }
                }
                _ => {
                    if row.line_type != crate::types::LineType::Parenthetical {
                        current_char = String::new();
                    }
                }
            }
        }

        let mut sorted_chars: Vec<_> = char_word_counts.into_iter().collect();
        sorted_chars.sort_by(|a, b| b.1.cmp(&a.1));

        for (ch, words) in sorted_chars {
            let scenes = char_scenes.get(&ch).cloned().unwrap_or_default();
            let mut scene_list: Vec<_> = scenes.into_iter().collect();
            scene_list.sort();
            let scenes_str = scene_list.join(", ");
            csv.push_str(&format!("\"{}\",{},\"{}\"\n", ch, words, scenes_str));
        }

        std::fs::write(path, csv)
    }

    pub fn set_error(&mut self, msg: &str) {
        self.status_msg = Some(msg.to_string());
        self.command_error = true;
    }

    pub fn save(&mut self) -> io::Result<()> {
        if self.is_tutorial {
            self.set_status("Cannot save the tutorial buffer. Press Ctrl+X to exit.");
            return Ok(());
        }
        if let Some(ref p) = self.file {
            let mut content = self.lines.join("\n");
            if !content.ends_with('\n') {
                content.push('\n');
            }
            fs::write(p, content)?;
            self.dirty = false;
            self.set_status(&format!("Wrote {} lines", self.lines.len()));

            // Trigger snapshot on manual save
            self.trigger_snapshot();
        }
        Ok(())
    }
}
