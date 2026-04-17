use crate::formatting::StringCaseExt;
use crate::app::{App, AppMode, EnsembleItem, CharacterItem, NavigatorItem};
use crate::layout::{strip_sigils, find_visual_cursor};
use ratatui::style::Color;
use crate::types::LineType;
use crate::parser::Parser;

impl App {
    pub fn total_word_count(&self) -> usize {
        self.lines
            .iter()
            .map(|l| l.split_whitespace().count())
            .sum()
    }

    pub fn total_page_count(&self) -> usize {
        self.layout
            .iter()
            .filter_map(|r| r.page_num)
            .next_back()
            .unwrap_or(1)
    }

    pub fn current_page_number(&self) -> usize {
        let (vis_row_idx, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        for i in (0..=vis_row_idx).rev() {
            if let Some(p) = self.layout[i].page_num {
                return p;
            }
        }
        1
    }

    pub fn open_scene_navigator(&mut self) {
        self.scenes.clear();
        let mut current_scene: Option<NavigatorItem> = None;
        let mut last_color: Option<Color> = None;

        for row in &self.layout {
            if row.line_type == LineType::Note {
                last_color = row.override_color;
            }

            if row.line_type == LineType::Section {
                if let Some(s) = current_scene.take() {
                    self.scenes.push(s);
                }
                let label = strip_sigils(&row.raw_text, row.line_type)
                    .trim()
                    .to_string();
                self.scenes.push(NavigatorItem {
                    line_idx: row.line_idx,
                    label,
                    is_section: true,
                    ..Default::default()
                });
                last_color = None;
            } else if row.line_type == LineType::SceneHeading {
                if let Some(s) = current_scene.take() {
                    self.scenes.push(s);
                }
                let mut raw_heading = strip_sigils(&row.raw_text, row.line_type).to_string();
                while let Some(start) = raw_heading.find("[[") {
                    if let Some(end_offset) = raw_heading[start..].find("]]") {
                        raw_heading.replace_range(start..start + end_offset + 2, "");
                    } else {
                        break;
                    }
                }
                let label = raw_heading.trim().to_uppercase_1to1();
                let color = row.override_color.or(last_color);
                current_scene = Some(NavigatorItem {
                    line_idx: row.line_idx,
                    label,
                    is_section: false,
                    scene_num: row.scene_num.clone(),
                    synopses: Vec::new(),
                    color,
                });
                last_color = None;
            } else if row.line_type == LineType::Synopsis {
                if let Some(ref mut s) = current_scene {
                    let note_text = strip_sigils(&row.raw_text, row.line_type).to_string();
                    if !note_text.is_empty() {
                        s.synopses.push(note_text);
                    }
                }
                last_color = None;
            } else if !matches!(
                row.line_type,
                LineType::Empty | LineType::Note | LineType::Synopsis
            ) {
                last_color = None;
            }

            if let Some(ref mut s) = current_scene
                && s.color.is_none() {
                    if let Some(c) = row.override_color {
                        s.color = Some(c);
                    } else if let Some(c) = row.fmt.note_color.values().next() {
                        s.color = Some(*c);
                    }
                }
        }
        if let Some(s) = current_scene {
            self.scenes.push(s);
        }

        if self.scenes.is_empty() {
            self.set_status("No scenes found");
        } else {
            self.mode = AppMode::SceneNavigator;
            self.selected_scene = 0;
            for (idx, item) in self.scenes.iter().enumerate() {
                if item.line_idx <= self.cursor_y {
                    self.selected_scene = idx;
                } else {
                    break;
                }
            }
            self.navigator_state.select(Some(self.selected_scene));
        }
    }

    pub fn open_character_sidebar(&mut self) {
        use std::collections::HashMap;
        self.character_stats.clear();
        let mut stats_map: HashMap<String, CharacterItem> = HashMap::new();
        let mut current_scene = "Untitled Scene".to_string();
        let mut current_character: Option<String> = None;

        for row in &self.layout {
            if row.line_type == LineType::SceneHeading {
                let mut raw_heading = strip_sigils(&row.raw_text, row.line_type).to_string();
                while let Some(start) = raw_heading.find("[[") {
                    if let Some(end_offset) = raw_heading[start..].find("]]") {
                        raw_heading.replace_range(start..start + end_offset + 2, "");
                    } else {
                        break;
                    }
                }
                current_scene = raw_heading.trim().to_uppercase_1to1();
            }

            if row.line_type == LineType::Character {
                let name = strip_sigils(&row.raw_text, row.line_type)
                    .trim()
                    .to_uppercase_1to1();
                let entry = stats_map
                    .entry(name.clone())
                    .or_insert_with(|| CharacterItem {
                        name: name.clone(),
                        ..Default::default()
                    });
                entry.dialogue_blocks += 1;
                if !entry
                    .appears_in_scenes
                    .iter()
                    .any(|(s, _)| s == &current_scene)
                {
                    entry
                        .appears_in_scenes
                        .push((current_scene.clone(), row.line_idx));
                    entry.scenes_count += 1;
                }
                current_character = Some(name);
            } else if row.line_type == LineType::Dialogue {
                if let Some(name) = &current_character
                    && let Some(entry) = stats_map.get_mut(name) {
                        let words = row.raw_text.split_whitespace().count();
                        entry.word_count += words;
                    }
            } else if row.line_type != LineType::Parenthetical {
                current_character = None;
            }
        }

        let mut stats: Vec<CharacterItem> = stats_map.into_values().collect();
        // Sort by dialogue prominence
        stats.sort_by(|a, b| {
            (b.dialogue_blocks * 10 + b.word_count).cmp(&(a.dialogue_blocks * 10 + a.word_count))
        });

        self.character_stats = stats;
        self.selected_character = 0;
        self.refresh_ensemble_list();
        self.selected_ensemble_idx = 0;
        self.ensemble_state.select(Some(0));
        self.mode = AppMode::CharacterNavigator;
    }

    pub fn refresh_ensemble_list(&mut self) {
        self.ensemble_items.clear();
        for i in 0..self.character_stats.len() {
            let item = self.character_stats[i].clone();

            // Character Header
            self.ensemble_items.push(EnsembleItem::CharacterHeader(i));

            // Stat: Scenes (with Hint)
            let scene_hint = if item.is_expanded {
                Some("(Cast in Scenes ↓)".to_string())
            } else {
                Some("(Tab to show)".to_string())
            };
            self.ensemble_items.push(EnsembleItem::Stat(
                format!("Scenes: {}", item.scenes_count),
                scene_hint,
                false,
            ));

            // Scene Links (if expanded)
            if item.is_expanded {
                for (j, (scene_name, line_idx)) in item.appears_in_scenes.iter().enumerate() {
                    let is_last_scene = j == item.appears_in_scenes.len() - 1;
                    self.ensemble_items.push(EnsembleItem::SceneLink(
                        scene_name.clone(),
                        *line_idx,
                        is_last_scene,
                    ));
                }
            }

            // Stat: Dialogues
            self.ensemble_items.push(EnsembleItem::Stat(
                format!("Dialogues: {}", item.dialogue_blocks),
                None,
                false,
            ));

            // Stat: Words (Last stat in tree)
            self.ensemble_items.push(EnsembleItem::Stat(
                format!("Words: {}", item.word_count),
                None,
                true,
            ));

            // Separator
            self.ensemble_items.push(EnsembleItem::Separator);
        }
    }

    pub fn parse_document(&mut self) {
        self.types = Parser::parse(&self.lines);

        // Forced Uppercase Transformation for key elements
        for i in 0..self.lines.len() {
            let lt = self.types[i];
            if matches!(
                lt,
                LineType::SceneHeading
                    | LineType::Character
                    | LineType::DualDialogueCharacter
                    | LineType::Transition
            ) {
                // Determine the clean upper version to avoid unnecessary updates
                let current = &self.lines[i];
                let upper = current.to_uppercase_1to1();
                if *current != upper {
                    self.lines[i] = upper;
                    self.dirty = true;
                }
            }
        }

        // Production lock: auto-assign suffixed numbers to new scenes
        if self.config.production_lock {
            self.auto_number_locked_scenes();
        }

        self.characters.clear();
        self.locations.clear();

        for (i, t) in self.types.iter().enumerate() {
            if *t == LineType::Character || *t == LineType::DualDialogueCharacter {
                let full_name = self.lines[i]
                    .trim_start_matches('@')
                    .trim_end_matches('^')
                    .trim();
                let name = if let Some(idx) = full_name.find('(') {
                    full_name[..idx].trim()
                } else {
                    full_name
                };
                if !name.is_empty() {
                    self.characters.insert(name.to_uppercase_1to1());
                }
            } else if *t == LineType::SceneHeading {
                let scene = self.lines[i].trim().to_uppercase_1to1();
                let mut loc_str = scene.as_str();
                let mut matched = false;

                if loc_str.starts_with('.') && !loc_str.starts_with("..") {
                    loc_str = &loc_str[1..];
                } else {
                    let prefixes = [
                        "INT. ",
                        "EXT. ",
                        "EST. ",
                        "INT/EXT. ",
                        "I/E. ",
                        "E/I. ",
                        "I./E. ",
                        "E./I. ",
                        "INT ",
                        "EXT ",
                        "EST ",
                        "INT/EXT ",
                        "I/E ",
                        "E/I ",
                    ];
                    for p in prefixes {
                        if let Some(rest) = loc_str.strip_prefix(p) {
                            loc_str = rest;
                            matched = true;
                            break;
                        }
                    }
                    if !matched && let Some((_, rest)) = loc_str.split_once(". ") {
                        loc_str = rest;
                    }
                }

                let mut final_loc = loc_str.trim().to_string();

                if final_loc.ends_with('#')
                    && let Some(idx) = final_loc.rfind(" #")
                {
                    final_loc.truncate(idx);
                    final_loc = final_loc.trim().to_string();
                }

                if !final_loc.is_empty() {
                    self.locations.insert(final_loc);
                }
            }
        }
    }
}


impl crate::app::App {
    pub fn calculate_scene_height(&self, item: &NavigatorItem) -> usize {
        if item.is_section {
            return 2; // Section name + spacer
        }

        let max_w: usize = 45; // Match the wider navigator sidebar
        let mut height: usize = 0;

        // Heading wrapping
        let mut current_line_len: usize = 0;
        let heading_indent: usize = 5; // prefix(3) + connector(2)
        for word in item.label.split_whitespace() {
            if current_line_len + word.len() + heading_indent + 1 > max_w {
                height += 1;
                current_line_len = 0;
            }
            if current_line_len > 0 {
                current_line_len += 1;
            }
            current_line_len += word.len();
        }
        if current_line_len > 0 || height == 0 {
            height += 1;
        }

        // Synopsis wrapping
        for syn in &item.synopses {
            let mut current_line_len: usize = 0;
            let max_syn_w = 34; // Sync with UI
            let mut syn_lines: usize = 0;
            for word in syn.split_whitespace() {
                if current_line_len + word.len() + 1 > max_syn_w {
                    syn_lines += 1;
                    current_line_len = word.len();
                } else {
                    if current_line_len > 0 {
                        current_line_len += 1;
                    }
                    current_line_len += word.len();
                }
            }
            if current_line_len > 0 {
                syn_lines += 1;
            }
            height += syn_lines;
        }

        if item.synopses.is_empty() {
            height += 1; // "no synopsis" placeholder height
        }

        height += 1; // Empty separator line or ending spacer
        height
    }
}
