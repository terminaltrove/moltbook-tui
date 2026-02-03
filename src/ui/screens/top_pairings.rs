use crate::app::{App, Screen};

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;
use crate::ui::utils::format_follower_count;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_top_pairings(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::TopPairings, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Top pairings list
    let items: Vec<ListItem> = app
        .top_pairings
        .iter()
        .enumerate()
        .map(|(i, human)| {
            let is_selected = i == app.top_pairings_selected;

            let rank_color = match human.rank {
                1 => Color::Rgb(255, 215, 0),   // Gold
                2 => Color::Rgb(192, 192, 192), // Silver
                3 => Color::Rgb(205, 127, 50),  // Bronze
                _ => Color::DarkGray,
            };

            let rank_style = Style::default()
                .fg(rank_color)
                .add_modifier(if human.rank <= 3 {
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

            let verified_badge = if human.x_verified { " ★" } else { "" };

            // Blank line for visual spacing
            let blank_line = Line::from(vec![Span::raw("")]);

            // Line 1: Rank + @handle + verified badge
            let handle_line = Line::from(vec![
                Span::styled(format!(" {:>2}  ", human.rank), rank_style),
                Span::styled(format!("@{}", human.x_handle), name_style),
                Span::styled(verified_badge, Style::default().fg(MOLTBOOK_YELLOW)),
            ]);

            // Line 2: Display name
            let display_name_line = Line::from(vec![
                Span::raw("     "),
                Span::styled(&human.x_name, Style::default().fg(Color::DarkGray)),
            ]);

            // Line 3: Agent info + follower count
            let agent_line = Line::from(vec![
                Span::raw("     "),
                Span::styled("Agent: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&human.bot_name, Style::default().fg(MOLTBOOK_TEAL)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} followers",
                        format_follower_count(human.x_follower_count)
                    ),
                    Style::default().fg(Color::White),
                ),
            ]);

            ListItem::new(vec![handle_line, display_name_line, agent_line, blank_line])
        })
        .collect();

    let list_block = Block::default()
        .title(" Top Pairings (Human + Agent) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));

    let mut list_state = ListState::default();
    list_state.select(Some(app.top_pairings_selected));
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
