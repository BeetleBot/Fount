use crate::app::App;
use crate::types::LineType;

#[derive(Clone, Debug)]
pub struct SceneCard {
    pub start_line: usize,
    pub end_line: usize,
    pub heading: String,
    pub synopsis: String,
    pub preview: String,
    pub scene_num: Option<String>,
}

impl App {
    pub fn extract_scene_cards(&self) -> Vec<SceneCard> {
        let mut cards = Vec::new();
        let mut current_card: Option<SceneCard> = None;

        for (i, (&lt, line)) in self.types.iter().zip(self.lines.iter()).enumerate() {
            if lt == LineType::SceneHeading {
                if let Some(mut card) = current_card.take() {
                    card.end_line = i.saturating_sub(1);
                    cards.push(card);
                }

                let (clean_heading, scene_num) = if let Some(caps) = crate::layout::SCENE_NUM_RE.captures(line) {
                    (caps[1].to_string(), Some(caps[2].to_string()))
                } else {
                    (line.clone(), None)
                };

                current_card = Some(SceneCard {
                    start_line: i,
                    end_line: self.lines.len().saturating_sub(1),
                    heading: clean_heading,
                    synopsis: String::new(),
                    preview: String::new(),
                    scene_num,
                });
            } else if let Some(ref mut card) = current_card {
                if lt == LineType::Synopsis {
                    if !card.synopsis.is_empty() {
                        card.synopsis.push('\n');
                    }
                    card.synopsis.push_str(crate::layout::strip_sigils(line, lt));
                } else if card.preview.is_empty() && (lt == LineType::Action || lt == LineType::Dialogue) {
                    card.preview = line.clone();
                }
            }
        }

        if let Some(card) = current_card {
            cards.push(card);
        }

        cards
    }

    pub fn swap_cards(&mut self, i: usize, j: usize) {
        let cards = self.extract_scene_cards();
        if i >= cards.len() || j >= cards.len() || i == j {
            return;
        }

        self.save_state(true);

        // Ensure i is before j
        let (first_idx, second_idx) = if i < j { (i, j) } else { (j, i) };
        let card_a = &cards[first_idx];
        let card_b = &cards[second_idx];

        let block_a: Vec<String> = self.lines[card_a.start_line..=card_a.end_line].to_vec();
        let block_b: Vec<String> = self.lines[card_b.start_line..=card_b.end_line].to_vec();

        if first_idx + 1 == second_idx {
            // Adjacent: [A][B] -> [B][A]
            let start = card_a.start_line;
            let end = card_b.end_line;
            self.lines.splice(start..=end, [block_b, block_a].concat());
            self.selected_card_idx = second_idx;
        } else {
            // Non-adjacent: This is more complex because moving A to B shifts indices.
            // For now, we'll stick to adjacent swaps which is what Arrows support.
        }

        self.dirty = true;
        self.parse_document();
        self.update_layout();
    }

    pub fn add_card(&mut self, after_idx: usize) {
        self.save_state(true);
        let cards = self.extract_scene_cards();
        
        let insert_line = if cards.is_empty() {
            self.lines.len()
        } else if after_idx < cards.len() {
            cards[after_idx].end_line + 1
        } else {
            self.lines.len()
        };

        // Insert a blank scene
        let new_lines = vec![
            String::new(),
            ". ".to_string(), // The dot forces it to be a scene heading even if empty
            "= ".to_string(),
            String::new(),
        ];
        
        for (i, line) in new_lines.into_iter().enumerate() {
            self.lines.insert(insert_line + i, line);
        }
        
        self.parse_document();
        self.update_layout();
        
        self.selected_card_idx = if cards.is_empty() { 0 } else { after_idx + 1 };
        self.is_card_editing = true;
        self.is_heading_editing = true;
        self.card_input_buffer = String::new();
    }

    pub fn delete_card(&mut self, idx: usize) {
        let cards = self.extract_scene_cards();
        if idx >= cards.len() {
            return;
        }
        self.save_state(true);
        let card = &cards[idx];
        self.lines.drain(card.start_line..=card.end_line);
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.parse_document();
        self.update_layout();
        self.selected_card_idx = idx.saturating_sub(1);
    }

    pub fn update_card_content(&mut self, idx: usize, heading: String, synopsis: String) {
        let cards = self.extract_scene_cards();
        if idx >= cards.len() {
            return;
        }
        self.save_state(true);
        let card = &cards[idx];
        
        // Update Heading
        self.lines[card.start_line] = if heading.starts_with('.') { heading } else { format!(".{}", heading) };
        
        // Update Synopsis
        // Find existing synopsis line or insert one
        let mut syn_found = false;
        for i in card.start_line + 1..=card.end_line {
            if self.types[i] == LineType::Synopsis {
                self.lines[i] = format!("= {}", synopsis);
                syn_found = true;
                break;
            }
        }
        
        if !syn_found {
            self.lines.insert(card.start_line + 1, format!("= {}", synopsis));
        }
        
        self.parse_document();
        self.update_layout();
    }
}
