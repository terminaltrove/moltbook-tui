use crate::app::App;

use super::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use super::fonts::SPINNER_FRAMES;
use super::header::LOGO_ART;
use super::utils::{centered_fixed_rect, format_number_with_commas, humanize_date, humanize_number, parse_simple_markdown};

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_spinner(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let spinner_char = SPINNER_FRAMES[app.spinner_frame];
    let text = format!("{} Loading...", spinner_char);

    let width = text.len() as u16 + 4;
    let height = 3;

    // Center in the content area (accounting for header ~12 lines, footer ~3 lines)
    let header_height = 12;
    let footer_height = 3;
    let content_area = Rect::new(
        area.x,
        area.y + header_height,
        area.width,
        area.height.saturating_sub(header_height + footer_height),
    );
    let popup_area = centered_fixed_rect(width, height, content_area);

    frame.render_widget(Clear, popup_area);

    let spinner_widget = Paragraph::new(text)
        .style(Style::default().fg(MOLTBOOK_TEAL))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_RED)),
        );

    frame.render_widget(spinner_widget, popup_area);
}

pub fn render_help(frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_fixed_rect(45, 26, area);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  j / ↓     Move down"),
        Line::from("  k / ↑     Move up"),
        Line::from("  Enter     Open post"),
        Line::from("  Esc       Go back"),
        Line::from(""),
        Line::from(Span::styled(
            "  Sorting",
            Style::default().fg(MOLTBOOK_TEAL),
        )),
        Line::from("  n         Sort by New"),
        Line::from("  t         Sort by Top"),
        Line::from("  d         Sort by Discussed"),
        Line::from("  R         Sort by Random"),
        Line::from("  f / ←/→   Cycle time filter"),
        Line::from(""),
        Line::from("  r         Refresh"),
        Line::from("  o         Open in browser"),
        Line::from("  a         Toggle auto-refresh"),
        Line::from("  +/-       Adjust refresh interval"),
        Line::from("  N         Next page"),
        Line::from("  P         Previous page"),
        Line::from("  1-8       Navigate screens"),
        Line::from("  `         Toggle debug panel"),
        Line::from("  ?         Toggle help"),
        Line::from("  q         Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help_widget = Paragraph::new(help_text).block(
        Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );

    frame.render_widget(help_widget, popup_area);
}

pub fn render_error(frame: &mut Frame, app: &App) {
    let error = match &app.error_message {
        Some(e) => e,
        None => return,
    };

    let area = frame.area();
    let popup_area = centered_fixed_rect(50, 10, area);

    // Clear background so content doesn't show through
    frame.render_widget(Clear, popup_area);

    // Show friendly or technical message based on toggle
    let display_text = if app.show_technical_error {
        error.clone()
    } else {
        "Something went wrong.".to_string()
    };

    let help_text = "Esc: dismiss    e: toggle details    r: retry";

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(&display_text, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let error_widget = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Error ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(error_widget, popup_area);
}

pub fn render_debug(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Debug panel on the right side (40% width)
    let debug_width = (area.width * 40) / 100;
    let debug_area = Rect::new(
        area.width.saturating_sub(debug_width),
        0,
        debug_width,
        area.height,
    );

    frame.render_widget(Clear, debug_area);

    // Show last N debug entries that fit
    let available_lines = debug_area.height.saturating_sub(2) as usize; // -2 for borders
    let start_idx = app.debug_log.len().saturating_sub(available_lines);
    let visible_logs: Vec<Line> = app
        .debug_log
        .iter()
        .skip(start_idx)
        .map(|msg| {
            let style = if msg.contains("ERROR") {
                Style::default().fg(Color::Red)
            } else if msg.contains("OK:") {
                Style::default().fg(Color::Green)
            } else if msg.contains("GET") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(msg.as_str(), style))
        })
        .collect();

    let debug_widget = Paragraph::new(visible_logs).block(
        Block::default()
            .title(" Debug (` to close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(debug_widget, debug_area);
}

pub fn render_submolt_detail_modal(frame: &mut Frame, app: &App) {
    if !app.show_submolt_detail || app.submolts.is_empty() {
        return;
    }

    let submolt = &app.submolts[app.submolts_selected];

    // Modal dimensions: 50 wide x 12 tall
    let area = centered_fixed_rect(50, 12, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED))
        .title(format!(" m/{} ", submolt.name))
        .title_style(
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Content: description + subscriber count
    let desc = submolt.description.as_deref().unwrap_or("No description");
    let subs = humanize_number(submolt.subscriber_count);

    let text = vec![
        Line::from(Span::styled(desc, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} subscribers", subs),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Space or Esc to close",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, inner);
}

pub fn render_agent_preview_sidebar(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calculate content area (same as screen layouts)
    // Header: 11 lines, Footer: 3 lines
    let header_height = 11;
    let footer_height = 3;
    let content_y = header_height;
    let content_height = area.height.saturating_sub(header_height + footer_height);

    // Sidebar on the right side (40% width), within content area
    let sidebar_width = (area.width * 40) / 100;
    let sidebar_area = Rect::new(
        area.width.saturating_sub(sidebar_width),
        content_y,      // Start below header
        sidebar_width,
        content_height, // End above footer
    );

    frame.render_widget(Clear, sidebar_area);

    let title = app
        .preview_agent_name
        .as_ref()
        .map(|n| format!(" u/{} ", n))
        .unwrap_or_else(|| " Agent Preview ".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED))
        .title(title)
        .title_style(
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(sidebar_area);
    frame.render_widget(block, sidebar_area);

    // Content based on loading state and whether profile is loaded
    let content = if app.is_preview_loading {
        // Show animated spinner while loading, centered vertically and horizontally
        let spinner_char = SPINNER_FRAMES[app.spinner_frame];
        let vertical_padding = inner.height.saturating_sub(1) / 2;
        let mut lines: Vec<Line> = (0..vertical_padding).map(|_| Line::from("")).collect();
        lines.push(Line::from(Span::styled(
            format!("{} Loading...", spinner_char),
            Style::default().fg(MOLTBOOK_TEAL),
        )));
        lines
    } else if let Some(ref profile) = app.agent_profile {
        let desc = profile
            .description
            .as_deref()
            .unwrap_or("No description");
        let claimed = if profile.is_claimed {
            ""
        } else {
            " (unclaimed)"
        };
        let owner_info = profile
            .owner
            .as_ref()
            .and_then(|o| o.x_handle.as_ref())
            .map(|h| format!("@{}", h))
            .unwrap_or_default();

        let mut content = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" Karma: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_number_with_commas(profile.karma),
                    Style::default()
                        .fg(MOLTBOOK_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Followers: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_number_with_commas(profile.follower_count),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Following: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_number_with_commas(profile.following_count),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
        ];

        // Add description paragraphs with spacing between them
        for (i, paragraph) in desc.split("\n\n").enumerate() {
            if i > 0 {
                content.push(Line::from("")); // Space between paragraphs
            }
            let trimmed = paragraph.trim();
            if !trimmed.is_empty() {
                let mut para_spans = vec![Span::styled(" ", Style::default())];
                para_spans.extend(parse_simple_markdown(trimmed));
                content.push(Line::from(para_spans));
            }
        }

        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled(" Owner: ", Style::default().fg(Color::DarkGray)),
            Span::styled(owner_info, Style::default().fg(MOLTBOOK_TEAL)),
            Span::styled(claimed, Style::default().fg(Color::DarkGray)),
        ]));
        content.push(Line::from(vec![
            Span::styled(" Joined: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                humanize_date(&profile.created_at),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        content.push(Line::from(""));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            " Tab: Close   Enter: Open Profile",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));

        content
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                " Loading...",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let paragraph = if app.is_preview_loading {
        Paragraph::new(content)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new(content).wrap(Wrap { trim: true })
    };

    frame.render_widget(paragraph, inner);
}

pub fn render_about(frame: &mut Frame) {
    let area = frame.area();
    // Center in content area (accounting for header and footer)
    let header_height = 8;
    let footer_height = 3;
    let content_area = Rect::new(
        area.x,
        area.y + header_height,
        area.width,
        area.height.saturating_sub(header_height + footer_height),
    );
    let popup_area = centered_fixed_rect(70, 26, content_area);

    frame.render_widget(Clear, popup_area);

    let version = env!("CARGO_PKG_VERSION");

    let mut about_text: Vec<Line> = LOGO_ART
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(MOLTBOOK_RED)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    about_text.extend(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Moltbook TUI",
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "by Terminal Trove",
            Style::default().fg(Color::Rgb(184, 255, 167)),
        )),
        Line::from(Span::styled(
            "https://terminaltrove.com/moltbook-tui/",
            Style::default().fg(Color::Rgb(184, 255, 167)),
        )),
        Line::from(Span::styled(
            format!("v{}", version),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "A social network for AI agents.",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "They share, discuss, and upvote.",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Humans welcome to observe (in the terminal). 🦞",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "(moltbook TUI is not affiliated with Moltbook)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Built with:",
            Style::default().fg(MOLTBOOK_TEAL),
        )),
        Line::from(Span::styled(
            "  Ratatui",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "https://github.com/terminaltrove/moltbook-tui/",
            Style::default().fg(MOLTBOOK_TEAL),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press '8' or Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let about_widget = Paragraph::new(about_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" About ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_RED)),
        );

    frame.render_widget(about_widget, popup_area);
}
