use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph, Wrap},
};

pub fn draw_index_cards(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;

    // Apply dim modifier to the background of the editor behind the cards
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                let current_style = cell.style();
                cell.set_style(current_style.add_modifier(Modifier::DIM));
            }
        }
    }

    let accent = Color::from(theme.ui.normal_mode_bg.clone());
    let dim = Color::from(theme.ui.dim.clone());
    let normal_fg = theme.ui.foreground.clone().map(Color::from).unwrap_or(Color::White);
    let selection_bg = Color::from(theme.ui.selection_bg.clone());
    let selection_fg = Color::from(theme.ui.selection_fg.clone());

    let cards = app.extract_scene_cards();
    
    // Header for the mode
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);
        
    let header_area = chunks[0];
    let grid_area = chunks[1];
    let footer_area = chunks[2];
    
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" 󱉟 STORY ARCHITECT ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" • {} Scenes ", cards.len()), Style::default().fg(dim)),
    ])).alignment(Alignment::Center), header_area);

    // Grid details
    let columns = 3;
    let card_w = (grid_area.width.saturating_sub(4)) / columns; // Accounting for gaps
    let card_h = 10;
    let visible_rows = (grid_area.height / card_h) as usize;
    
    // Auto-scrolling logic
    let selected_row = (app.selected_card_idx / columns as usize) as usize;
    if selected_row < app.card_row_offset {
        app.card_row_offset = selected_row;
    } else if selected_row >= app.card_row_offset + visible_rows {
        app.card_row_offset = selected_row.saturating_sub(visible_rows) + 1;
    }

    for (i, card) in cards.iter().enumerate() {
        let card_row = i / columns as usize;
        if card_row < app.card_row_offset || card_row >= app.card_row_offset + visible_rows {
            continue;
        }
        
        let row_in_view = card_row - app.card_row_offset;
        let col = i % columns as usize;
        
        let x = grid_area.x + 2 + (col as u16 * (card_w + 1));
        let y = grid_area.y + (row_in_view as u16 * card_h);
        
        let card_rect = Rect::new(x, y, card_w, card_h - 1); // -1 for vertical gap
        let is_selected = i == app.selected_card_idx;
        
        // --- DRAW SHADOW ---
        if card_rect.width > 2 && card_rect.height > 2 {
            let shadow_rect = Rect::new(card_rect.x + 1, card_rect.y + 1, card_rect.width, card_rect.height);
            for sy in shadow_rect.top()..shadow_rect.bottom() {
                for sx in shadow_rect.left()..shadow_rect.right() {
                    if let Some(cell) = f.buffer_mut().cell_mut((sx, sy)) {
                        cell.set_char(' '); // Shadow background
                        cell.set_style(Style::default().bg(Color::Rgb(15, 15, 15)));
                    }
                }
            }
        }

        // --- DRAW CARD CONTENT ---
        f.render_widget(Clear, card_rect);
        
        let mut border_style = Style::default().fg(dim);
        let mut body_bg = Color::Rgb(30, 30, 30);
        
        if is_selected {
            border_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
            if !app.is_card_editing {
                body_bg = Color::Rgb(40, 40, 45); // Subtle lift
            } else {
                body_bg = Color::Rgb(25, 25, 35); // Sunken/Focus look
            }
        }
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .bg(body_bg);
            
        f.render_widget(block, card_rect);
        
        // Header Bar (Tab)
        let header_bar_rect = Rect::new(card_rect.x + 1, card_rect.y + 1, card_rect.width - 2, 1);
        let header_style = if is_selected {
            Style::default().bg(accent).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(dim).fg(Color::Black)
        };
        
        let header_text = if let Some(ref num) = card.scene_num {
            format!(" SCENE {} ", num)
        } else {
            format!(" SCENE {} ", i + 1)
        };
        
        f.render_widget(Paragraph::new(header_text).style(header_style), header_bar_rect);

        // Content Area
        let inner = Rect::new(card_rect.x + 1, card_rect.y + 2, card_rect.width - 2, card_rect.height - 3);
        
        let mut card_lines = Vec::new();
        
        // Heading
        let heading_content = if is_selected && app.is_card_editing && app.is_heading_editing {
            format!("{}█", app.card_input_buffer)
        } else {
            let h = card.heading.trim_start_matches('.').to_string();
            if h.is_empty() { "[No Heading]".to_string() } else { h }
        };
        
        let heading_style = if is_selected && app.is_card_editing && app.is_heading_editing {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(normal_fg).add_modifier(Modifier::BOLD)
        };
        
        card_lines.push(Line::from(Span::styled(heading_content, heading_style)));
        card_lines.push(Line::from(Span::styled(" ", Style::default()))); // Spacer
        
        // Synopsis
        let syn_content = if is_selected && app.is_card_editing && !app.is_heading_editing {
            format!("{}█", app.card_input_buffer)
        } else if !card.synopsis.is_empty() {
            card.synopsis.clone()
        } else if !card.preview.is_empty() {
            card.preview.clone()
        } else {
            "Tap Enter to plan...".to_string()
        };
        
        let syn_style = if is_selected && app.is_card_editing && !app.is_heading_editing {
            Style::default().fg(selection_fg).bg(selection_bg)
        } else if !card.synopsis.is_empty() {
            Style::default().fg(normal_fg).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(dim).add_modifier(Modifier::ITALIC)
        };
        
        card_lines.push(Line::from(Span::styled(syn_content, syn_style)));
        
        let p = Paragraph::new(card_lines)
            .wrap(Wrap { trim: true });
            
        f.render_widget(p, inner);
    }

    // Footer hints
    let footer_text = if app.is_card_editing {
        if app.is_heading_editing { " [Enter] Move to Synopsis | [Esc] Cancel Edit " } else { " [Enter] Save & Finish | [Esc] Cancel Edit " }
    } else {
        " [Arrows] Navigate | [Enter] Edit Card | [n] New Scene | [Shift+Arw] Move | [/ed] Exit "
    };
    
    let footer_style = if app.is_card_editing {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(dim)
    };
    
    f.render_widget(Paragraph::new(Span::styled(footer_text, footer_style)).alignment(Alignment::Center), footer_area);
}
