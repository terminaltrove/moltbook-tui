use crate::app::{App, Screen};
use crate::config::RowDisplay;

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::{render_error, render_submolt_detail_modal};
use crate::ui::utils::humanize_number;

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

pub fn render_submolts(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header with ANSI logo and nav tabs
    let header = Paragraph::new(render_shared_header(Screen::Submolts, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    // Count featured submolts (already sorted: featured first)
    let featured_count = app.submolts.iter().filter(|s| s.featured_at.is_some()).count();

    // Submolts grid with scrolling
    let title = if featured_count > 0 {
        format!("Submolts ({}) - {} Featured", app.submolts.len(), featured_count)
    } else {
        format!("Submolts ({})", app.submolts.len())
    };
    let grid_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    let grid_area = grid_block.inner(chunks[1]);
    frame.render_widget(grid_block, chunks[1]);

    // Calculate dynamic row counts based on row_display
    let row_height = match app.row_display {
        RowDisplay::Compact => 4u16,     // name + subs + minimal padding
        RowDisplay::Normal => 6u16,      // name + 2 desc lines + subs + spacing
        RowDisplay::Comfortable => 7u16, // name + 2 desc lines + subs + blank + spacing
    };
    let visible_rows = (grid_area.height / row_height).max(1) as usize;
    let total_items = app.submolts.len();
    let total_rows = total_items.div_ceil(4);

    // Auto-scroll to keep selection visible
    let selected_row = app.submolts_selected / 4;
    if selected_row < app.submolts_scroll_row {
        app.submolts_scroll_row = selected_row;
    } else if selected_row >= app.submolts_scroll_row + visible_rows {
        app.submolts_scroll_row = selected_row.saturating_sub(visible_rows - 1);
    }

    // Create row layout dynamically
    let row_constraints: Vec<Constraint> = (0..visible_rows)
        .map(|_| Constraint::Length(row_height))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_area);

    // Render visible rows with scroll offset
    for (display_row_idx, row_area) in rows.iter().enumerate() {
        let actual_row_idx = app.submolts_scroll_row + display_row_idx;

        if actual_row_idx >= total_rows {
            break;
        }

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 4); 4])
            .split(*row_area);

        for col_idx in 0..4 {
            let idx = actual_row_idx * 4 + col_idx;
            if idx < app.submolts.len() {
                let submolt = &app.submolts[idx];
                let is_selected = idx == app.submolts_selected;
                let is_featured = submolt.featured_at.is_some();

                let name_style = if is_selected {
                    Style::default()
                        .fg(MOLTBOOK_TEAL)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MOLTBOOK_TEAL)
                };

                let desc = submolt.description.as_deref().unwrap_or("No description");

                // Build name line with star for featured submolts
                let name_line = if is_featured {
                    Line::from(vec![
                        Span::styled("★ ", Style::default().fg(MOLTBOOK_YELLOW)),
                        Span::styled(format!("m/{}", submolt.name), name_style),
                    ])
                } else {
                    Line::from(Span::styled(format!("m/{}", submolt.name), name_style))
                };

                // Build cell content based on row_display setting
                let cell_content = match app.row_display {
                    RowDisplay::Compact => vec![
                        name_line,
                        Line::from(Span::styled(
                            format!("{} subs", humanize_number(submolt.subscriber_count)),
                            Style::default().fg(Color::White),
                        )),
                    ],
                    RowDisplay::Normal => vec![
                        name_line,
                        Line::from(Span::styled(desc, Style::default().fg(Color::DarkGray))),
                        Line::from(Span::styled(
                            format!("{} subs", humanize_number(submolt.subscriber_count)),
                            Style::default().fg(Color::White),
                        )),
                    ],
                    RowDisplay::Comfortable => vec![
                        name_line,
                        Line::from(Span::styled(desc, Style::default().fg(Color::DarkGray))),
                        Line::from(Span::styled(
                            format!("{} subs", humanize_number(submolt.subscriber_count)),
                            Style::default().fg(Color::White),
                        )),
                        Line::from(""),
                    ],
                };

                let cell_style = if is_selected {
                    Style::default().bg(Color::Rgb(30, 30, 30))
                } else {
                    Style::default()
                };

                let cell = Paragraph::new(cell_content)
                    .style(cell_style)
                    .block(Block::default().padding(Padding::new(2, 2, 1, 0)))
                    .wrap(Wrap { trim: true });

                frame.render_widget(cell, cols[col_idx]);
            }
        }
    }

    // Render scrollbar if needed
    if total_rows > visible_rows {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state = ScrollbarState::new(total_rows).position(app.submolts_scroll_row);

        frame.render_stateful_widget(
            scrollbar,
            chunks[1].inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }

    // Footer
    let footer = Paragraph::new("h/j/k/l: Nav • Enter: View Posts • Space: Details • ?: Help")
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

    // Render submolt detail modal on top
    render_submolt_detail_modal(frame, app);
}
