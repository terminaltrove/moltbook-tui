use crate::app::{App, Screen};

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::fonts::render_figlet_name;
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;
use crate::ui::utils::format_number_with_commas;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_leaderboard(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::Leaderboard, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Leaderboard list
    let items: Vec<ListItem> = app
        .leaderboard
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let is_selected = i == app.leaderboard_selected;

            let rank_color = match agent.rank {
                1 => Color::Rgb(255, 215, 0),   // Gold
                2 => Color::Rgb(192, 192, 192), // Silver
                3 => Color::Rgb(205, 127, 50),  // Bronze
                _ => Color::DarkGray,
            };

            let rank_style = Style::default()
                .fg(rank_color)
                .add_modifier(if agent.rank <= 3 {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });

            let name_style = if is_selected {
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let verified = agent
                .owner
                .as_ref()
                .and_then(|o| o.x_verified)
                .unwrap_or(false);
            let verified_badge = if verified { " ★" } else { "" };

            let x_handle = agent
                .owner
                .as_ref()
                .and_then(|o| o.x_handle.as_ref())
                .map(|h| format!("@{}", h))
                .unwrap_or_default();

            let claimed = if agent.is_claimed { "" } else { " (unclaimed)" };

            // Blank line for visual spacing
            let blank_line = Line::from(vec![Span::raw("")]);

            if agent.rank <= 3 {
                // pixeloidbold at 0.5x scale (matches bit tool output)
                let figlet_lines = render_figlet_name(&agent.name, 12, true);

                let art_style = Style::default().fg(rank_color).add_modifier(Modifier::BOLD);

                let mut lines = Vec::new();

                // Add top padding before each top-3 entry
                lines.push(Line::from(vec![Span::raw("")]));

                // First line: rank + first figlet line
                if let Some(first) = figlet_lines.first() {
                    lines.push(Line::from(vec![
                        Span::styled(format!(" {:>2}  ", agent.rank), rank_style),
                        Span::styled(first.clone(), art_style),
                    ]));
                }

                // Remaining figlet lines (typically just one more for 2-line font)
                for figlet_line in figlet_lines.iter().skip(1) {
                    lines.push(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(figlet_line.clone(), art_style),
                    ]));
                }

                // Handle line (on its own for vertical centering)
                lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(verified_badge, Style::default().fg(MOLTBOOK_TEAL)),
                    Span::styled(x_handle.clone(), Style::default().fg(MOLTBOOK_TEAL)),
                    Span::styled(claimed, Style::default().fg(MOLTBOOK_TEAL)),
                ]));

                // Karma line
                lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(
                        format!("↑ {} karma", format_number_with_commas(agent.karma)),
                        Style::default().fg(MOLTBOOK_YELLOW),
                    ),
                ]));

                // Blank line for top 3 spacing
                lines.push(blank_line);

                ListItem::new(lines)
            } else {
                // Regular text rendering for ranks 4-10
                let rank_star = "";

                // Line 1: Name only
                let name_line = Line::from(vec![
                    Span::styled(format!(" {:>2}  ", agent.rank), rank_style),
                    Span::styled(rank_star, rank_style),
                    Span::styled(&agent.name, name_style),
                ]);

                // Line 2: Handle in teal
                let handle_line = Line::from(vec![
                    Span::raw("     "),
                    Span::styled(verified_badge, Style::default().fg(MOLTBOOK_TEAL)),
                    Span::styled(x_handle.clone(), Style::default().fg(MOLTBOOK_TEAL)),
                    Span::styled(claimed, Style::default().fg(MOLTBOOK_TEAL)),
                ]);

                // Line 3: Karma in gold
                let karma_line = Line::from(vec![
                    Span::raw("     "),
                    Span::styled(
                        format!("↑ {} karma", format_number_with_commas(agent.karma)),
                        Style::default().fg(MOLTBOOK_YELLOW),
                    ),
                ]);

                ListItem::new(vec![name_line, handle_line, karma_line, blank_line])
            }
        })
        .collect();

    let list_block = Block::default()
        .title(" ★ Top 10 Agents ★ ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));

    let mut list_state = ListState::default();
    list_state.select(Some(app.leaderboard_selected));
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer
    let footer = Paragraph::new("j/k: Nav • 1-7: Screens • ?: Help")
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
