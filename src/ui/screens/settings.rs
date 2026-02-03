use crate::app::{App, Screen};
use crate::config::RowDisplay;

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_settings(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::Settings, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Settings content
    let make_option = |label: &str, is_selected: bool, is_current: bool| -> Span<'static> {
        let text = if is_current {
            format!("[{}]", label)
        } else {
            format!(" {} ", label)
        };

        if is_selected && is_current {
            Span::styled(
                text,
                Style::default()
                    .fg(Color::Rgb(0, 0, 0))
                    .bg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_current {
            Span::styled(
                text,
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_selected {
            Span::styled(text, Style::default().fg(Color::White))
        } else {
            Span::styled(text, Style::default().fg(Color::DarkGray))
        }
    };

    // Row Display setting (index 0)
    let row_display_selected = app.settings_selected == 0;
    let row_display_line = Line::from(vec![
        Span::raw("    "),
        make_option(
            "Compact",
            row_display_selected,
            app.row_display == RowDisplay::Compact,
        ),
        Span::raw("  "),
        make_option(
            "Normal",
            row_display_selected,
            app.row_display == RowDisplay::Normal,
        ),
        Span::raw("  "),
        make_option(
            "Comfortable",
            row_display_selected,
            app.row_display == RowDisplay::Comfortable,
        ),
    ]);

    // Refresh Interval setting (index 1)
    let refresh_selected = app.settings_selected == 1;
    let refresh_intervals = [10u64, 30, 60, 120];
    let refresh_line = Line::from(vec![
        Span::raw("    "),
        make_option("10s", refresh_selected, app.refresh_interval_secs == 10),
        Span::raw("  "),
        make_option("30s", refresh_selected, app.refresh_interval_secs == 30),
        Span::raw("  "),
        make_option("60s", refresh_selected, app.refresh_interval_secs == 60),
        Span::raw("  "),
        make_option("120s", refresh_selected, app.refresh_interval_secs == 120),
        Span::raw("  "),
        make_option(
            "Off",
            refresh_selected,
            !refresh_intervals.contains(&app.refresh_interval_secs),
        ),
    ]);

    let settings_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            if row_display_selected {
                Span::styled(
                    "Row Display",
                    Style::default()
                        .fg(MOLTBOOK_TEAL)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("Row Display", Style::default().fg(Color::White))
            },
        ]),
        row_display_line,
        Line::from(Span::styled(
            "    Affects post list, comments, agents, and submolts",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            if refresh_selected {
                Span::styled(
                    "Refresh Interval",
                    Style::default()
                        .fg(MOLTBOOK_TEAL)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("Refresh Interval", Style::default().fg(Color::White))
            },
        ]),
        refresh_line,
        Line::from(Span::styled(
            "    Auto-refresh interval for the feed",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("←/→", Style::default().fg(MOLTBOOK_TEAL)),
            Span::styled(" Change  ", Style::default().fg(Color::DarkGray)),
            Span::styled("j/k", Style::default().fg(MOLTBOOK_TEAL)),
            Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(MOLTBOOK_TEAL)),
            Span::styled(" Back", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let settings_widget = Paragraph::new(settings_lines).block(
        Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(settings_widget, chunks[1]);

    // Footer
    let footer = Paragraph::new("1-7: Navigate • ?: Help • q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_RED)),
        );
    frame.render_widget(footer, chunks[2]);

    if app.error_message.is_some() {
        render_error(frame, app);
    }
}
