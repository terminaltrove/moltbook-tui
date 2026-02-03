use crate::app::{App, Screen};

use crate::ui::colors::{MOLTBOOK_BLUE, MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;
use crate::ui::utils::format_number_with_commas;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_stats(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::Stats, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Stats content
    let stats_lines = if let Some(ref stats) = app.stats {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Platform Statistics",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Agents:    "),
                Span::styled(
                    format!("{} agents", format_number_with_commas(stats.agents as i64)),
                    Style::default()
                        .fg(MOLTBOOK_RED)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Submolts:  "),
                Span::styled(
                    format!(
                        "{} submolts",
                        format_number_with_commas(stats.submolts as i64)
                    ),
                    Style::default()
                        .fg(MOLTBOOK_TEAL)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Posts:     "),
                Span::styled(
                    format!("{} posts", format_number_with_commas(stats.posts as i64)),
                    Style::default()
                        .fg(MOLTBOOK_BLUE)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Comments:  "),
                Span::styled(
                    format!(
                        "{} comments",
                        format_number_with_commas(stats.comments as i64)
                    ),
                    Style::default()
                        .fg(MOLTBOOK_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  Navigation",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  Press 1 - Feed"),
            Line::from("  Press 2 - Leaderboard (top agents)"),
            Line::from("  Press 3 - Top Pairings (top humans + agents)"),
            Line::from("  Press 4 - Agents"),
            Line::from("  Press 5 - Submolts (communities)"),
            Line::from("  Press 6 - Stats (this screen)"),
            Line::from("  Press 7 - Settings"),
        ]
    } else {
        vec![Line::from(""), Line::from("  Loading stats...")]
    };

    let stats_widget = Paragraph::new(stats_lines).block(
        Block::default()
            .title("Stats")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(stats_widget, chunks[1]);

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
