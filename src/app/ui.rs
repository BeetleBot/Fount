use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;
use crate::{
    app::{App, AppMode},
    formatting::{RenderConfig, StringCaseExt, render_inline},
    layout::{find_visual_cursor, strip_sigils},
    types::{LineType, PAGE_WIDTH, base_style},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    f.render_widget(ratatui::widgets::Clear, area);

    let (mode_str, mode_bg) = match app.mode {
        AppMode::Normal => (" NORMAL ", Color::LightBlue),
        AppMode::Command => (" COMMAND ", Color::Yellow),
        AppMode::SceneNavigator => (" NAVIGATOR ", Color::LightCyan),
        AppMode::SettingsPane => (" SETTINGS ", Color::LightCyan),
        AppMode::ExportPane => (" EXPORT ", Color::LightCyan),
        AppMode::Search => (" SEARCH ", Color::LightMagenta),
        _ => (" PROMPT ", Color::LightRed),
    };


    let is_prompt = app.mode != AppMode::Normal;
    let has_status = app.status_msg.is_some();

    let show_bottom = !app.config.focus_mode || is_prompt || has_status;

    let in_command_mode = app.mode == AppMode::Command;
    let footer_height = if show_bottom { 1 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let (mut text_area, footer_area) = (chunks[0], chunks[1]);

    app.sidebar_area = Rect::default();
    if app.mode == AppMode::SceneNavigator {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(45), Constraint::Min(0)])
            .split(text_area);
        app.sidebar_area = side_chunks[0];
        text_area = side_chunks[1];

        let sidebar_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM));
        f.render_widget(sidebar_block, app.sidebar_area);
    }

    app.settings_area = Rect::default();
    if app.mode == AppMode::SettingsPane || app.mode == AppMode::Shortcuts || app.mode == AppMode::ExportPane {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(35)])
            .split(text_area);
        text_area = side_chunks[0];
        app.settings_area = side_chunks[1];

        let settings_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(mode_bg).add_modifier(Modifier::DIM));
        f.render_widget(settings_block, app.settings_area);
    }

    let height = text_area.height as usize;
    app.visible_height = height;
    let page_w = PAGE_WIDTH.min(text_area.width);
    let global_pad = text_area.width.saturating_sub(page_w) / 2;

    let (vis_row, vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

    let mut pad_top = 0;

    if app.config.strict_typewriter_mode {
        let absolute_center = area.height / 2;
        let center_offset = absolute_center.saturating_sub(text_area.y) as usize;
        if vis_row < center_offset {
            pad_top = center_offset - vis_row;
        }
        app.scroll = vis_row.saturating_sub(center_offset);
    } else if app.config.typewriter_mode {
        let absolute_center = area.height / 2;
        let center_offset = absolute_center.saturating_sub(text_area.y) as usize;
        app.scroll = vis_row.saturating_sub(center_offset);
    } else {
        if vis_row < app.scroll {
            app.scroll = vis_row;
        }
        if vis_row >= app.scroll + height {
            app.scroll = vis_row + 1 - height;
        }
    }

    let mut dark_gray_style = Style::default();
    if !app.config.no_color {
        dark_gray_style = dark_gray_style.add_modifier(Modifier::DIM);
    }

    let mut sug_style = Style::default();
    if !app.config.no_formatting {
        sug_style = sug_style.add_modifier(Modifier::DIM | Modifier::BOLD);
    }

    let mut page_num_style = Style::default();
    if !app.config.no_color {
        page_num_style = page_num_style.add_modifier(Modifier::DIM);
    }

    let mut visible: Vec<Line> = Vec::new();
    for _ in 0..pad_top {
        visible.push(Line::raw(""));
    }

    let mirror_scenes = app.config.mirror_scene_numbers == crate::config::MirrorOption::Always;

    visible.extend(
        app.layout
            .iter()
            .skip(app.scroll)
            .take(height.saturating_sub(pad_top))
            .map(|row| {
                let mut spans = Vec::new();
                let gap_size = 6u16;

                if let Some(ref snum) = row.scene_num {
                    let s_str = snum.to_string();
                    let s_len = UnicodeWidthStr::width(s_str.as_str()) as u16;

                    if global_pad >= s_len + gap_size {
                        let pad = global_pad - s_len - gap_size;
                        spans.push(Span::raw(" ".repeat(pad as usize)));
                        spans.push(Span::styled(s_str, dark_gray_style));
                        spans.push(Span::raw(" ".repeat(gap_size as usize)));
                    } else {
                        spans.push(Span::styled(s_str, dark_gray_style));
                        spans.push(Span::raw(" "));
                    }
                } else {
                    spans.push(Span::raw(" ".repeat(global_pad as usize)));
                }

                spans.push(Span::raw(" ".repeat(row.indent as usize)));

                let mut bst = base_style(row.line_type, &app.config);
                if let Some(c) = row.override_color
                    && !app.config.no_color
                {
                    bst.fg = Some(c);
                }

                let mut display = if row.is_active || !app.config.hide_markup {
                    row.raw_text.clone()
                } else {
                    strip_sigils(&row.raw_text, row.line_type).to_string()
                };

                let reveal_markup = !app.config.hide_markup || row.is_active;
                let skip_md = row.line_type == LineType::Boneyard;

                if matches!(
                    row.line_type,
                    LineType::SceneHeading | LineType::Transition | LineType::Shot
                ) {
                    display = display.to_uppercase_1to1();
                } else if matches!(
                    row.line_type,
                    LineType::Character | LineType::DualDialogueCharacter
                ) {
                    if let Some(idx) = display.find('(') {
                        let name = display[..idx].to_uppercase_1to1();
                        let ext = &display[idx..];
                        display = format!("{}{}", name, ext);
                    } else {
                        display = display.to_uppercase_1to1();
                    }
                }

                let empty_logical_line = String::new();
                let full_logical_line = app.lines.get(row.line_idx).unwrap_or(&empty_logical_line);

                let is_last_visual_row = row.char_end == full_logical_line.chars().count();
                let mut meta_key_end = 0;

                if (row.line_type == LineType::MetadataKey
                    || (row.line_type == LineType::MetadataTitle && row.is_active))
                    && let Some(idx) = full_logical_line.find(':')
                {
                    meta_key_end = full_logical_line[..=idx].chars().count() + 1;
                }

                let mut row_highlights = HashSet::new();
                if app.show_search_highlight
                    && let Some(re) = &app.compiled_search_regex
                {
                    for mat in re.find_iter(full_logical_line) {
                        let start_byte = mat.start();
                        let end_byte = mat.end();

                        let char_start = full_logical_line[..start_byte].chars().count();
                        let char_len = full_logical_line[start_byte..end_byte].chars().count();

                        for idx in char_start..(char_start + char_len) {
                            row_highlights.insert(idx);
                        }
                    }
                }

                // Selection highlight (overrides search)
                let mut sel_highlights = HashSet::new();
                if let Some(((sel_sl, sel_sc), (sel_el, sel_ec))) = app.selection_range() {
                    let li = row.line_idx;
                    if li >= sel_sl && li <= sel_el {
                        let line_len = full_logical_line.chars().count();
                        let from = if li == sel_sl { sel_sc } else { 0 };
                        let to = if li == sel_el { sel_ec.min(line_len) } else { line_len };
                        for idx in from..to {
                            sel_highlights.insert(idx);
                        }
                    }
                }

                // Merge: sel_highlights takes priority — remove those from row_highlights
                // so render_inline gets the clean search set, and we'll override selected below.
                for idx in &sel_highlights {
                    row_highlights.remove(idx);
                }

                spans.extend(render_inline(
                    &display,
                    bst,
                    &row.fmt,
                    RenderConfig {
                        reveal_markup,
                        skip_markdown: skip_md,
                        exclude_comments: false,
                        char_offset: row.char_start,
                        meta_key_end,
                        no_color: app.config.no_color,
                        no_formatting: app.config.no_formatting,
                    },
                    &row_highlights,
                    &sel_highlights,
                ));

                if row.is_active
                    && row.line_idx == app.cursor_y
                    && is_last_visual_row
                    && let Some(sug) = &app.suggestion
                {
                    spans.push(Span::styled(sug.clone(), sug_style));
                }

                let right_text = if mirror_scenes {
                    row.scene_num.clone()
                } else {
                    row.page_num.map(|pnum| format!("{}.", pnum))
                };

                if let Some(r_str) = right_text {
                    let current_line_width: usize = spans
                        .iter()
                        .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                        .sum();

                    let target_pos = global_pad as usize + page_w as usize + gap_size as usize;
                    if target_pos > current_line_width {
                        spans.push(Span::raw(" ".repeat(target_pos - current_line_width)));
                        spans.push(Span::styled(r_str, page_num_style));
                    }
                }

                Line::from(spans)
            }),
    );

    f.render_widget(Paragraph::new(visible), text_area);

    if app.mode == AppMode::SceneNavigator {
        let items: Vec<ListItem> = app
            .scenes
            .iter()
            .enumerate()
            .map(|(i, (_, heading, snum, synopses, _))| {
                let is_selected = i == app.selected_scene;
                let mut lines = Vec::new();
                
                // Navigator is monochrome per request
                let base_style = if is_selected {
                    Style::default().add_modifier(Modifier::REVERSED).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                let dim_style = if is_selected {
                    base_style
                } else {
                    Style::default().add_modifier(Modifier::DIM)
                };

                let prefix = if is_selected { " ⟫ " } else { "   " };
                let s_tag = if let Some(s) = snum {
                    format!("{} ", s)
                } else {
                    String::new()
                };

                // Line 1: Heading
                lines.push(Line::from(vec![
                    Span::styled(prefix, base_style),
                    Span::styled(s_tag, base_style),
                    Span::styled(heading.to_uppercase_1to1(), base_style),
                ]));

                // Line 2: Synopsis or placeholder
                if let Some(syn) = synopses.first() {
                    lines.push(Line::from(vec![
                        Span::styled("     ", dim_style),
                        Span::styled(syn, dim_style.add_modifier(Modifier::ITALIC)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("     ", dim_style),
                        Span::styled("no synopsis", dim_style.add_modifier(Modifier::ITALIC)),
                    ]));
                }
                
                // Line 3: Always a spacer line for equal spacing
                lines.push(Line::from(""));
                
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).highlight_style(Style::default());
        f.render_stateful_widget(list, app.sidebar_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }), &mut app.navigator_state);
    }



    if app.mode == AppMode::SettingsPane {
        let settings = vec![
            ("Typewriter Mode", &app.config.strict_typewriter_mode),
            ("Auto-Save", &app.config.auto_save),
            ("Autocomplete", &app.config.autocomplete),
            ("Auto-Breaks", &app.config.auto_paragraph_breaks),
            ("Focus Mode", &app.config.focus_mode),
        ];

        let items: Vec<ListItem> = settings
            .into_iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let is_selected = i == app.selected_setting;
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(mode_bg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                let (icon, icon_style) = if *value { 
                    ("󰄬 ", Style::default().fg(Color::Green)) 
                } else { 
                    ("󰄱 ", Style::default().fg(Color::DarkGray)) 
                };

                ListItem::new(Line::from(vec![
                    Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
                    Span::styled(icon, if is_selected { style } else { icon_style }),
                    Span::styled(label, style),
                ]))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, app.settings_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }));
    }

    if app.mode == AppMode::ExportPane {
        let export_options = vec![
            format!(" Format: {}", app.config.export_format.to_uppercase()),
            format!(" Paper: {}", app.config.paper_size.to_uppercase()),
            format!(" Bold Headings: {}", if app.config.export_bold_scene_headings { "[X]" } else { "[ ]" }),
            " [ EXPORT ]".to_string(),
        ];

        let items: Vec<ListItem> = export_options
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                let is_selected = i == app.selected_export_option;
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(mode_bg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
                    Span::styled(label, style),
                ]))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, app.settings_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }));
    }

    if app.mode == AppMode::Shortcuts {
        let categories = vec![
            (" NAVIGATION ", vec![
                ("^H        ", "Scene Navigator"),
                ("^P        ", "Settings Pane"),
                ("^E        ", "Export Options"),
                ("^. / ^>   ", "Next Buffer"),
                ("^, / ^<   ", "Prev Buffer"),
                ("Arrows    ", "Move Cursor"),
                ("^+Arrows  ", "Jump Words"),
                ("Home/End  ", "Quick Jump"),
            ]),
            (" EDITING ", vec![
                ("^A        ", "Select All"),
                ("^C        ", "Copy selection"),
                ("^X        ", "Cut selected/line"),
                ("^V        ", "Paste clipboard"),
                ("Shift+Arr ", "Select Text"),
                ("^Backspace", "Delete word back"),
                ("^Delete   ", "Delete word ahead"),
                ("Enter     ", "Add Line"),
                ("Shift+Ent ", "Hard Break"),
            ]),
            (" COMMANDS ", vec![
                ("/w [path] ", "Save Buffer"),
                ("/q / /wq  ", "Exit App"),
                ("/renum    ", "Renumber Scenes"),
                ("/clearnum ", "Clear numbers"),
                ("/injectnum", "Tag Scene"),
                ("/[line]   ", "Jump to Line"),
                ("/s[num]   ", "Jump to Scene"),
                ("/search   ", "Global Search"),
                ("/u / /redo", "Undo / Redo"),
                ("/set [opt]", "Change Settings"),
                ("/pos      ", "Cursor Position"),
            ]),
        ];

        let mut items = Vec::new();
        for (cat, shortcuts) in categories {
            items.push(ListItem::new(""));
            items.push(ListItem::new(Span::styled(format!("  ──{}──", cat), Style::default().fg(mode_bg).add_modifier(Modifier::DIM))));
            for (key, desc) in shortcuts {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("  {}  ", key), Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                    Span::styled(desc, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ])));
            }
        }

        let list = List::new(items);
        f.render_widget(list, app.settings_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }));
    }

    // ── Footer rendering ──────────────────────────────────────────────────
    if footer_area.height > 0 {
        // ── Single-line "Pure" Status Bar ─────────────────────────────────
        let mut spans = Vec::new();

        // 1. Mode (Accented)
        spans.push(Span::styled(mode_str, Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" │ "));

        // 2. Filename (Accented)
        let fname = app.file.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "[No Name]".to_string());
        let dirty_str = if app.dirty { " [+]" } else { "" };
        spans.push(Span::styled(format!("{}{}", fname, dirty_str), Style::default().fg(mode_bg)));
        spans.push(Span::raw(" │ "));

        // 3. Center Section: Command Input OR Status Message OR Hints
        if app.mode == AppMode::Command {
            let cmd_style = if app.command_error {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled("/", cmd_style));
            spans.push(Span::styled(&app.command_input, cmd_style));
            
            if app.command_input.is_empty() && !app.command_error {
                spans.push(Span::styled(" type a command...", Style::default().fg(Color::DarkGray)));
            }
        } else if let Some(msg) = &app.status_msg {
            let style = if app.command_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            };
            spans.push(Span::styled(msg, style));
        } else {
            match app.mode {
                AppMode::Search => {
                    let prompt_base = if app.last_search.is_empty() {
                        "SEARCH: ".to_string()
                    } else {
                        format!("SEARCH [{}]: ", app.last_search)
                    };
                    spans.push(Span::raw(format!("{}{}", prompt_base, app.search_query)));
                }
                AppMode::PromptSave => spans.push(Span::raw("SAVE MODIFIED SCRIPT? (Y/N/C) ")),
                AppMode::PromptFilename => spans.push(Span::raw(format!("FILENAME: {} ", app.filename_input))),
                AppMode::PromptExportFilename => spans.push(Span::raw(format!("EXPORT {}: {} ", app.config.export_format.to_uppercase(), app.filename_input))),
                _ => spans.push(Span::styled("COMMANDS [F1]", Style::default().fg(Color::DarkGray))),
            }
        }

        // 4. Right Side: Counts & Cursor (Plain text)
        let word_count = app.total_word_count();
        let line_count = app.lines.len();
        let pos_str = format!("{}:{}", app.cursor_y + 1, app.cursor_x + 1);

        let mut right_spans = Vec::new();
        right_spans.push(Span::raw(" │ "));
        right_spans.push(Span::styled(format!("{}W {}L", word_count, line_count), Style::default().fg(Color::DarkGray)));
        right_spans.push(Span::raw(" │ "));
        right_spans.push(Span::styled(pos_str, Style::default().add_modifier(Modifier::BOLD)));

        let left_width: usize = spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
        let right_width: usize = right_spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
        let total_width = footer_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            spans.push(Span::raw(" ".repeat(pad_len)));
        }
        
        spans.extend(right_spans);

        f.render_widget(Paragraph::new(Line::from(spans)), footer_area);

        if app.mode == AppMode::Command {
            let mode_w = UnicodeWidthStr::width(mode_str);
            let fname_w = UnicodeWidthStr::width(fname.as_str()) + UnicodeWidthStr::width(dirty_str);
            let cur_x = footer_area.x + (mode_w + 3 + fname_w + 3 + 1 + UnicodeWidthStr::width(app.command_input.as_str())) as u16;
            f.set_cursor_position((cur_x, footer_area.y));
        }
    }

    // ── Cursor for non-command modes ──────────────────────────────────────
    if !in_command_mode {
        match app.mode {
            AppMode::Search if footer_area.height > 0 => {
                let prompt_base = if app.last_search.is_empty() {
                    "Search: ".to_string()
                } else {
                    format!("Search [{}]: ", app.last_search)
                };
                let query_w = UnicodeWidthStr::width(prompt_base.as_str())
                    + UnicodeWidthStr::width(app.search_query.as_str());
                let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
                let cur_screen_x = footer_area.x
                    + center_start as u16
                    + UnicodeWidthStr::width(prompt_base.as_str()) as u16;
                f.set_cursor_position((cur_screen_x, footer_area.y));
            }
            AppMode::PromptFilename if footer_area.height > 0 => {
                let prompt_base = "File Name to Write: ";
                let query_w = UnicodeWidthStr::width(prompt_base)
                    + UnicodeWidthStr::width(app.filename_input.as_str());
                let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
                let cur_screen_x = footer_area.x
                    + center_start as u16
                    + UnicodeWidthStr::width(prompt_base) as u16;
                f.set_cursor_position((cur_screen_x, footer_area.y));
            }
            AppMode::PromptExportFilename if footer_area.height > 0 => {
                let prompt_base = format!("EXPORT {}: ", app.config.export_format.to_uppercase());
                let query_w = UnicodeWidthStr::width(prompt_base.as_str())
                    + UnicodeWidthStr::width(app.filename_input.as_str());
                let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
                let cur_screen_x = footer_area.x
                    + center_start as u16
                    + UnicodeWidthStr::width(prompt_base.as_str()) as u16;
                f.set_cursor_position((cur_screen_x, footer_area.y));
            }
            AppMode::Normal => {
                let cur_screen_y =
                    text_area.y + pad_top as u16 + (vis_row.saturating_sub(app.scroll)) as u16;
                let cur_screen_x = text_area.x + global_pad + vis_x;
                if cur_screen_y < text_area.y + text_area.height {
                    f.set_cursor_position((cur_screen_x, cur_screen_y));
                }
            }
            _ => {}
        }
    }
}
