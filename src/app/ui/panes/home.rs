use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph},
};

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 { return (128, 128, 128); }
    (
        u8::from_str_radix(&hex[0..2], 16).unwrap_or(128),
        u8::from_str_radix(&hex[2..4], 16).unwrap_or(128),
        u8::from_str_radix(&hex[4..6], 16).unwrap_or(128),
    )
}

fn gradient_color(stops: &[(u8, u8, u8)], t: f32) -> Color {
    if stops.len() < 2 { return Color::White; }
    let t = t.clamp(0.0, 1.0);
    let seg = stops.len() - 1;
    let scaled = t * seg as f32;
    let idx = (scaled as usize).min(seg - 1);
    let lt = scaled - idx as f32;
    let (a, b) = (stops[idx], stops[idx + 1]);
    Color::Rgb(
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * lt) as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * lt) as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * lt) as u8,
    )
}

pub fn draw_home(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;

    let accent = Color::from(theme.ui.normal_mode_bg.clone());
    let sel_bg = Color::from(theme.ui.selection_bg.clone());
    let sel_fg = Color::from(theme.ui.selection_fg.clone());
    let normal_fg = theme.primary_fg();
    let dim = Color::from(theme.ui.dim.clone());

    let stops = vec![
        hex_to_rgb(&theme.ui.normal_mode_bg.0),
        hex_to_rgb(&theme.ui.tree_mode_bg.0),
        hex_to_rgb(&theme.ui.search_mode_bg.0),
    ];

    // Main Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Header/Logo
            Constraint::Min(0),      // Content
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    // ── HEADER ──
    let logo = [
        "      ░██     ░████                                     ░██    ",
        "     ░██     ░██                                        ░██    ",
        "    ░██   ░████████  ░███████  ░██    ░██ ░████████  ░████████ ",
        "   ░██       ░██    ░██    ░██ ░██    ░██ ░██    ░██    ░██    ",
        "  ░██        ░██    ░██    ░██ ░██    ░██ ░██    ░██    ░██    ",
        " ░██         ░██    ░██    ░██ ░██   ░███ ░██    ░██    ░██    ",
        "░██          ░██     ░███████   ░█████░██ ░██    ░██     ░████ ",
    ];

    let mut logo_lines = Vec::new();
    let max_logo_w = logo.iter().map(|r| r.chars().count()).max().unwrap_or(1);
    for row in &logo {
        let mut spans = Vec::new();
        for (ci, ch) in row.chars().enumerate() {
            let t = ci as f32 / max_logo_w.max(1) as f32;
            if ch == ' ' {
                spans.push(Span::raw(" "));
            } else {
                spans.push(Span::styled(ch.to_string(), Style::default().fg(gradient_color(&stops, t))));
            }
        }
        logo_lines.push(Line::from(spans));
    }
    logo_lines.push(Line::from(Span::styled(
        format!("v{} — Blockbusters in Terminal", env!("CARGO_PKG_VERSION")),
        Style::default().fg(accent).add_modifier(Modifier::ITALIC),
    )));

    f.render_widget(
        Paragraph::new(logo_lines)
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(dim))),
        chunks[0]
    );

    // ── CONTENT ──
    let menu = ["New File", "New file with Structure", "Open File", "Tutorial", "Exit"];
    let mut menu_lines = Vec::new();
    menu_lines.push(Line::from(""));

    for (i, label) in menu.iter().enumerate() {
        let is_sel = i == app.home_selected;
        if is_sel {
            menu_lines.push(Line::from(vec![
                Span::styled(format!("  {}  ", if app.config.use_nerd_fonts { "󰁔" } else { "▸" }), Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD)),
                Span::styled(label.to_string(), Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            menu_lines.push(Line::from(Span::styled(format!("    {}  ", label), Style::default().fg(normal_fg))));
        }
        menu_lines.push(Line::from(""));
    }

    if !app.recent_files.is_empty() {
        menu_lines.push(Line::from(Span::styled("Recent Files", Style::default().fg(accent).add_modifier(Modifier::BOLD))));
        menu_lines.push(Line::from(""));
        for (i, path) in app.recent_files.iter().take(4).enumerate() {
            let idx = menu.len() + i;
            let is_sel = idx == app.home_selected;
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Unknown".to_string());
            if is_sel {
                menu_lines.push(Line::from(vec![
                    Span::styled(format!("  {}  ", if app.config.use_nerd_fonts { "󰁔" } else { "▸" }), Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD)),
                    Span::styled(name, Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                menu_lines.push(Line::from(Span::styled(format!("    {}  ", name), Style::default().fg(normal_fg))));
            }
            menu_lines.push(Line::from(""));
        }
    }

    f.render_widget(Paragraph::new(menu_lines).alignment(Alignment::Center), chunks[1]);

    // ── FOOTER ──
    let footer_start_idx = menu.len() + app.recent_files.len().min(4);
    let wiki_sel = app.home_selected == footer_start_idx;
    let github_sel = app.home_selected == footer_start_idx + 1;

    let wiki_label = if app.config.use_nerd_fonts { " 󰖟 Wiki " } else { " [Wiki] " };
    let github_label = if app.config.use_nerd_fonts { " 󰊤 GitHub " } else { " [GitHub] " };

    let wiki_style = if wiki_sel { Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD) } else { Style::default().fg(accent) };
    let github_style = if github_sel { Style::default().fg(sel_fg).bg(sel_bg).add_modifier(Modifier::BOLD) } else { Style::default().fg(accent) };

    let footer_spans = vec![
        Span::styled("Check out the ", theme.secondary_style()),
        Span::styled(wiki_label, wiki_style),
        Span::styled(" for documentation and the ", theme.secondary_style()),
        Span::styled(github_label, github_style),
        Span::styled(" repository for updates.", theme.secondary_style()),
    ];

    f.render_widget(
        Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(dim))
                .border_type(BorderType::Rounded)),
        chunks[2],
    );
}
