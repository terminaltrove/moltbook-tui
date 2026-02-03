use crate::app::{App, Screen};
use crate::config::RowDisplay;

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;
use crate::ui::utils::{format_number_with_commas, humanize_date};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_recent_agents(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::RecentAgents, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Recent agents list
    let items: Vec<ListItem> = app
        .recent_agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let is_selected = i == app.recent_selected;
            let rank = i + 1;

            let rank_style = Style::default().fg(Color::DarkGray);

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
            let verified_badge = if verified { "★" } else { "" };

            let x_handle = agent
                .owner
                .as_ref()
                .and_then(|o| o.x_handle.as_ref())
                .map(|h| format!("@{}", h))
                .unwrap_or_default();

            let claimed = if agent.is_claimed { "" } else { " (unclaimed)" };

            // Line 1: Rank + Name
            let name_line = Line::from(vec![
                Span::styled(format!(" {:>2}  ", rank), rank_style),
                Span::styled(&agent.name, name_style),
            ]);

            // Line 2: Verified badge + Handle + Claimed status
            let handle_line = Line::from(vec![
                Span::raw("     "),
                Span::styled(verified_badge, Style::default().fg(MOLTBOOK_TEAL)),
                Span::styled(x_handle, Style::default().fg(MOLTBOOK_TEAL)),
                Span::styled(claimed, Style::default().fg(MOLTBOOK_TEAL)),
            ]);

            // Line 3: Karma + Creation time
            let karma_line = Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    format!("↑ {} karma", format_number_with_commas(agent.karma)),
                    Style::default().fg(MOLTBOOK_YELLOW),
                ),
                Span::styled(
                    format!(" • {}", humanize_date(&agent.created_at)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            // Build lines based on row_display setting
            let lines = match app.row_display {
                RowDisplay::Compact => vec![name_line, karma_line, Line::from("")],
                RowDisplay::Normal => vec![name_line, handle_line, karma_line, Line::from("")],
                RowDisplay::Comfortable => vec![name_line, handle_line, karma_line, Line::from("")],
            };
            ListItem::new(lines)
        })
        .collect();

    let list_block = Block::default()
        .title(format!("Recent Agents ({})", app.recent_agents.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));

    let mut list_state = ListState::default();
    list_state.select(Some(app.recent_selected));
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
