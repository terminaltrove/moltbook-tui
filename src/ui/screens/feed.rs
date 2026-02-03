use crate::api::SortOrder;
use crate::app::{App, Screen};
use crate::config::RowDisplay;

use crate::ui::colors::{MOLTBOOK_BLUE, MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::fonts::SPINNER_FRAMES;
use crate::ui::header::{render_nav_tabs_line, render_sort_tabs_line, LOGO_ART};
use crate::ui::overlays::render_error;
use crate::ui::utils::{format_count, format_number_with_commas, humanize_date};

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render_feed(frame: &mut Frame, app: &App) {
    // Time filter is shown inline with sort tabs (except for New sort)
    let header_height = 13;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with simple styled logo
    let new_count = app.new_post_ids.len();
    let new_indicator = if new_count > 0 {
        format!(" ({} new)", new_count)
    } else {
        String::new()
    };

    // Build stats line
    let stats_line = if let Some(ref stats) = app.stats {
        Line::from(vec![
            Span::styled(
                " Sort: ",
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.sort_display(),
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                new_indicator.clone(),
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} agents", format_count(stats.agents)),
                Style::default().fg(MOLTBOOK_RED),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} submolts", format_count(stats.submolts)),
                Style::default().fg(MOLTBOOK_TEAL),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} posts", format_count(stats.posts)),
                Style::default().fg(MOLTBOOK_BLUE),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} comments", format_count(stats.comments)),
                Style::default().fg(MOLTBOOK_YELLOW),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                " Sort: ",
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.sort_display(),
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                new_indicator.clone(),
                Style::default()
                    .fg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    };

    // Build sort tabs line (with time filter inline if not New)
    let time_filter_opt = if app.sort_order != SortOrder::New {
        Some(app.time_filter)
    } else {
        None
    };
    let sort_tabs_line = render_sort_tabs_line(app.sort_order, time_filter_opt);

    // ANSI block art logo
    let mut logo_lines: Vec<Line> = LOGO_ART
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

    logo_lines.push(Line::from(Span::styled(
        " the front page of the agent internet",
        Style::default().fg(Color::DarkGray),
    )));
    logo_lines.push(Line::from(""));
    logo_lines.push(stats_line);
    logo_lines.push(Line::from(""));
    logo_lines.push(render_nav_tabs_line(Screen::Feed));
    logo_lines.push(Line::from(""));
    logo_lines.push(sort_tabs_line);

    let header = Paragraph::new(logo_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Posts list
    let posts: Vec<ListItem> = app
        .posts
        .iter()
        .enumerate()
        .map(|(i, post)| {
            let is_selected = i == app.selected_index;

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

            // Use lighter color for metadata when selected (DarkGray bg makes DarkGray text invisible)
            let meta_style = if is_selected {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(&post.title, title_style),
            ]);

            // Build meta spans dynamically, only including author if present
            let mut meta_spans = vec![
                Span::raw("    "),
                Span::styled(submolt, Style::default().fg(MOLTBOOK_TEAL)),
            ];
            if let Some(ref author) = post.author {
                meta_spans.push(Span::styled(" • ", meta_style));
                meta_spans.push(Span::styled(format!("u/{}", author.name), meta_style));
            }
            meta_spans.push(Span::styled(" • ", meta_style));
            meta_spans.push(Span::styled(humanize_date(&post.created_at), meta_style));
            meta_spans.push(Span::styled(" • ", meta_style));
            meta_spans.push(Span::styled(
                format!("{} pts", format_number_with_commas(post.score())),
                meta_style,
            ));
            meta_spans.push(Span::styled(" • ", meta_style));
            meta_spans.push(Span::styled(
                format!("{} comments", format_number_with_commas(post.comment_count)),
                meta_style,
            ));
            let meta = Line::from(meta_spans);

            // Build lines based on row_display setting
            let lines = match app.row_display {
                RowDisplay::Compact => vec![line, meta], // Title + meta (no extra spacing)
                RowDisplay::Normal => vec![line, meta], // Title + meta
                RowDisplay::Comfortable => vec![line, meta, Line::from("")], // Title + meta + blank
            };
            ListItem::new(lines)
        })
        .collect();

    // Build title - show submolt name when filtering
    let posts_title = if let Some(ref submolt) = app.current_submolt {
        format!("m/{} ({})", submolt.name, app.sort_display())
    } else {
        format!("Posts ({})", app.sort_display())
    };

    let posts_block = Block::default()
        .title(posts_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let posts_list = List::new(posts)
        .block(posts_block)
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));
    frame.render_stateful_widget(posts_list, chunks[1], &mut list_state);

    // Render scrollbar if there are more posts than visible
    let posts_area = chunks[1];
    let visible_height = posts_area.height.saturating_sub(2); // subtract borders
    let item_height = match app.row_display {
        RowDisplay::Compact => 2u16,
        RowDisplay::Normal => 2u16,
        RowDisplay::Comfortable => 3u16,
    };
    let visible_items = (visible_height / item_height) as usize;
    let total_items = app.posts.len();

    if total_items > visible_items {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state = ScrollbarState::new(total_items).position(app.selected_index);

        frame.render_stateful_widget(
            scrollbar,
            posts_area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }

    // Footer with keybindings, page info, and refresh countdown
    let page_indicator = if app.has_more_posts {
        format!("Page {} (more)", app.current_page + 1)
    } else {
        format!("Page {}", app.current_page + 1)
    };
    let countdown = app.seconds_until_refresh();
    let is_refreshing = app.is_loading && app.is_background_loading;

    let (spinner_text, refresh_text) = if is_refreshing {
        (
            format!("{} ", SPINNER_FRAMES[app.spinner_frame]),
            "Refreshing...".to_string(),
        )
    } else if app.refresh_interval_secs == 0 {
        ("  ".to_string(), "Refresh Off".to_string())
    } else if countdown == 0 {
        (
            format!("{} ", SPINNER_FRAMES[app.spinner_frame]),
            "Refreshing...".to_string(),
        )
    } else {
        ("  ".to_string(), format!("Refresh {:>2}s", countdown))
    };

    let refresh_color = if app.refresh_interval_secs > 0 {
        MOLTBOOK_TEAL
    } else {
        Color::DarkGray
    };

    // Build footer with optional submolt hint
    let nav_hint = if app.current_submolt.is_some() {
        format!(" j/k: Nav • Esc: All posts • ?: Help • {} • ", page_indicator)
    } else {
        format!(" j/k: Nav • N/P: Page • ?: Help • {} • ", page_indicator)
    };

    let footer_line = Line::from(vec![
        Span::styled(spinner_text, Style::default().fg(MOLTBOOK_TEAL)),
        Span::styled(nav_hint, Style::default().fg(Color::DarkGray)),
        Span::styled(refresh_text, Style::default().fg(refresh_color)),
        Span::styled(" • +/-: interval", Style::default().fg(Color::DarkGray)),
    ]);
    let footer = Paragraph::new(footer_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(footer, chunks[2]);

    // Error message overlay
    if app.error_message.is_some() {
        render_error(frame, app);
    }
}
