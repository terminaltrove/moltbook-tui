use crate::api::AgentProfile;
use crate::app::{App, Screen};

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;
use crate::ui::utils::{format_number_with_commas, humanize_date, parse_simple_markdown};

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

pub fn render_agent_profile(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Header
            Constraint::Length(10), // Agent info
            Constraint::Min(0),     // Posts list
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::AgentProfile, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Agent info section
    if let Some(ref profile) = app.agent_profile {
        render_agent_info(frame, profile, chunks[1]);
    } else {
        let loading = Paragraph::new("Loading agent profile...")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .title(" Agent ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(MOLTBOOK_RED)),
            );
        frame.render_widget(loading, chunks[1]);
    }

    // Posts list
    render_agent_posts(frame, app, chunks[2]);

    // Footer
    let footer = Paragraph::new("j/k: Nav • Enter: Open Post • r: Refresh • Esc: Back • ?: Help")
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_RED)),
        );
    frame.render_widget(footer, chunks[3]);

    if app.error_message.is_some() {
        render_error(frame, app);
    }
}

fn render_agent_info(frame: &mut Frame, profile: &AgentProfile, area: Rect) {
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

    let verified = profile
        .owner
        .as_ref()
        .and_then(|o| o.x_verified)
        .unwrap_or(false);
    let verified_badge = if verified { " ★" } else { "" };

    let post_count = profile
        .post_count
        .map(format_number_with_commas)
        .unwrap_or_else(|| "?".to_string());

    // Create outer block for the entire info section
    let outer_block = Block::default()
        .title(format!(" u/{} ", profile.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split the inner area into: username (1 line), description (flexible), spacer, stats (1 line), owner (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Username line + empty line
            Constraint::Min(1),    // Description (flexible, wraps)
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Stats + Owner lines
        ])
        .split(inner_area);

    // Username line
    let username_line = Line::from(vec![
        Span::styled(
            format!("  u/{}", profile.name),
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(claimed, Style::default().fg(Color::DarkGray)),
    ]);
    let username_widget = Paragraph::new(username_line);
    frame.render_widget(username_widget, chunks[0]);

    // Description with markdown parsing and text wrapping
    let mut desc_spans = vec![Span::styled("  ", Style::default())];
    desc_spans.extend(parse_simple_markdown(desc));
    let desc_line = Line::from(desc_spans);
    let desc_widget = Paragraph::new(desc_line).wrap(Wrap { trim: false });
    frame.render_widget(desc_widget, chunks[1]);

    // Stats and owner lines
    let footer_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  ↑ {} karma", format_number_with_commas(profile.karma)),
                Style::default()
                    .fg(MOLTBOOK_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} followers", format_number_with_commas(profile.follower_count)),
                Style::default().fg(Color::White),
            ),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} following", format_number_with_commas(profile.following_count)),
                Style::default().fg(Color::White),
            ),
            Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} posts", post_count),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),  // Blank line for spacing
        Line::from(vec![
            Span::styled("  Owner: ", Style::default().fg(Color::DarkGray)),
            Span::styled(verified_badge, Style::default().fg(MOLTBOOK_TEAL)),
            Span::styled(owner_info, Style::default().fg(MOLTBOOK_TEAL)),
            Span::styled("  │  Joined: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                humanize_date(&profile.created_at),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    let footer_widget = Paragraph::new(footer_lines);
    frame.render_widget(footer_widget, chunks[3]);
}

fn render_agent_posts(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .agent_posts
        .iter()
        .enumerate()
        .map(|(i, post)| {
            let is_selected = i == app.agent_posts_selected;

            let submolt = post
                .submolt
                .as_ref()
                .map(|s| format!("m/{}", s.name))
                .unwrap_or_else(|| "m/unknown".to_string());

            let title_style = if is_selected {
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let meta_style = if is_selected {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(&post.title, title_style),
            ]);

            let meta = Line::from(vec![
                Span::raw("    "),
                Span::styled(submolt, Style::default().fg(MOLTBOOK_TEAL)),
                Span::styled(" • ", meta_style),
                Span::styled(humanize_date(&post.created_at), meta_style),
                Span::styled(" • ", meta_style),
                Span::styled(
                    format!("{} pts", format_number_with_commas(post.score())),
                    meta_style,
                ),
                Span::styled(" • ", meta_style),
                Span::styled(
                    format!("{} comments", format_number_with_commas(post.comment_count)),
                    meta_style,
                ),
            ]);

            ListItem::new(vec![line, meta, Line::from("")])
        })
        .collect();

    let posts_block = Block::default()
        .title(format!(" Posts ({}) ", app.agent_posts.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let posts_list = List::new(items)
        .block(posts_block)
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));

    let mut list_state = ListState::default();
    list_state.select(Some(app.agent_posts_selected));
    frame.render_stateful_widget(posts_list, area, &mut list_state);

    // Render scrollbar if needed
    let visible_height = area.height.saturating_sub(2);
    let item_height = 3u16;
    let visible_items = (visible_height / item_height) as usize;
    let total_items = app.agent_posts.len();

    if total_items > visible_items {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state =
            ScrollbarState::new(total_items).position(app.agent_posts_selected);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }
}
