use crate::api::Comment;
use crate::app::App;

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL};
use crate::ui::overlays::render_error;
use crate::ui::utils::format_number_with_commas;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_post_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(15),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header - show post title
    let header_text = app
        .current_post
        .as_ref()
        .map(|p| p.title.clone())
        .unwrap_or_else(|| "Post Detail".to_string());
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_RED)),
        );
    frame.render_widget(header, chunks[0]);

    // Post content
    if let Some(ref post) = app.current_post {
        let submolt = post
            .submolt
            .as_ref()
            .map(|s| format!("m/{}", s.name))
            .unwrap_or_else(|| "m/unknown".to_string());

        let content = post.content.clone().unwrap_or_default();

        // Build author/submolt line dynamically
        let mut info_spans: Vec<Span> = Vec::new();
        if let Some(ref author) = post.author {
            info_spans.push(Span::styled("by ", Style::default().fg(Color::DarkGray)));
            info_spans.push(Span::styled(
                format!("u/{}", author.name),
                Style::default().fg(Color::White),
            ));
            info_spans.push(Span::styled(" in ", Style::default().fg(Color::DarkGray)));
        } else {
            info_spans.push(Span::styled("in ", Style::default().fg(Color::DarkGray)));
        }
        info_spans.push(Span::styled(&submolt, Style::default().fg(MOLTBOOK_TEAL)));
        info_spans.push(Span::styled(" • ", Style::default().fg(Color::DarkGray)));
        info_spans.push(Span::styled(
            format!("{} upvotes", format_number_with_commas(post.score())),
            Style::default().fg(Color::White),
        ));
        info_spans.push(Span::styled(" • ", Style::default().fg(Color::DarkGray)));
        info_spans.push(Span::styled(
            format!("{} comments", format_number_with_commas(post.comment_count)),
            Style::default().fg(Color::White),
        ));

        // Start with metadata line
        let mut post_lines = vec![
            Line::from(info_spans),
            Line::from(""), // Blank line after metadata
        ];

        // Split content into multiple lines preserving paragraphs
        for line in content.lines() {
            post_lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            )));
        }

        // Add URL if present
        if let Some(ref url) = post.url {
            post_lines.push(Line::from("")); // Blank line before URL
            post_lines.push(Line::from(Span::styled(
                url.clone(),
                Style::default().fg(Color::Cyan),
            )));
        }

        let post_widget = Paragraph::new(post_lines)
            .block(
                Block::default()
                    .title("Post")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(MOLTBOOK_RED)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(post_widget, chunks[1]);
    }

    // Comments
    render_comments(frame, app, chunks[2]);

    // Footer with refresh countdown
    let countdown = app.seconds_until_refresh();
    let footer_text = format!(
        "j/k: Nav • Enter: Collapse • Esc: Back • ?: Help • Refresh {}s",
        countdown
    );
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_RED)),
        );
    frame.render_widget(footer, chunks[3]);

    // Error message overlay
    if app.error_message.is_some() {
        render_error(frame, app);
    }
}

/// Wraps text to fit within max_width, returning individual lines
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            result.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        for word in line.split_whitespace() {
            if current_line.is_empty() {
                // First word - take it even if too long
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                // Word fits with space
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                // Word doesn't fit, start new line
                result.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            result.push(current_line);
        }
    }

    if result.is_empty() {
        result.push(String::new());
    }
    result
}

fn render_comments(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let selected_id = app.get_selected_comment_id();
    let mut visible_comment_count = 0;

    // Build the prefix strings for each depth level
    // prefix_stack tracks whether we're at the last sibling at each depth
    fn flatten_comments(
        comments: &[Comment],
        lines: &mut Vec<Line<'static>>,
        depth: usize,
        prefix_stack: &mut Vec<bool>, // true if this depth level has more siblings
        app: &App,
        selected_id: &Option<String>,
        visible_count: &mut usize,
        available_width: usize,
    ) {
        let len = comments.len();
        for (i, comment) in comments.iter().enumerate() {
            let is_last = i == len - 1;
            let is_selected = selected_id.as_ref() == Some(&comment.id);
            let is_collapsed = app.is_comment_collapsed(&comment.id);
            let reply_count = count_total_comments(&comment.replies);

            // Build the prefix for this comment
            let mut prefix = String::new();
            for (d, has_more) in prefix_stack.iter().enumerate() {
                if d < depth {
                    if *has_more {
                        prefix.push_str("│ ");
                    } else {
                        prefix.push_str("  ");
                    }
                }
            }

            // Add the branch character for this comment
            let branch = if depth == 0 {
                ""
            } else if is_last {
                "└─"
            } else {
                "├─"
            };

            // Selection indicator
            let selection_marker = if is_selected { "▶ " } else { "  " };

            // When collapsed with replies, show only a compact summary line
            if is_collapsed && reply_count > 0 {
                let total_hidden = count_total_comments(&comment.replies) + 1; // +1 for this comment
                let summary = format!("[+{} comments hidden]", total_hidden);

                let summary_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                let line = Line::from(vec![
                    Span::styled(selection_marker, Style::default().fg(MOLTBOOK_TEAL)),
                    Span::styled(prefix.clone(), Style::default().fg(Color::DarkGray)),
                    Span::styled(branch, Style::default().fg(Color::DarkGray)),
                    Span::styled(summary, summary_style),
                ]);
                lines.push(line);
                *visible_count += 1;
                lines.push(Line::from("")); // Spacing
            } else {
                // Expanded: show full comment
                let collapse_indicator = if !comment.replies.is_empty() {
                    "[-] ".to_string()
                } else {
                    "".to_string()
                };

                let header_style = if is_selected {
                    Style::default()
                        .fg(MOLTBOOK_TEAL)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                        .fg(MOLTBOOK_TEAL)
                        .add_modifier(Modifier::BOLD)
                };

                // Build header text based on whether author is present
                let header_text = if let Some(ref author) = comment.author {
                    format!("{} • ↑ {}", author.name, comment.score())
                } else {
                    format!("↑ {}", comment.score())
                };

                let header = Line::from(vec![
                    Span::styled(selection_marker, Style::default().fg(MOLTBOOK_TEAL)),
                    Span::styled(prefix.clone(), Style::default().fg(Color::DarkGray)),
                    Span::styled(branch, Style::default().fg(Color::DarkGray)),
                    Span::styled(collapse_indicator, Style::default().fg(Color::Yellow)),
                    Span::styled(header_text, header_style),
                ]);
                lines.push(header);
                *visible_count += 1;

                // Build content prefix (continues the tree lines)
                let mut content_prefix = String::from("  "); // space for selection marker
                for (d, has_more) in prefix_stack.iter().enumerate() {
                    if d < depth {
                        if *has_more {
                            content_prefix.push_str("│ ");
                        } else {
                            content_prefix.push_str("  ");
                        }
                    }
                }
                // Add continuation for this level
                if depth > 0 {
                    if is_last {
                        content_prefix.push_str("  ");
                    } else {
                        content_prefix.push_str("│ ");
                    }
                }

                // Calculate text width: total - prefix - 2 for border
                let text_width = available_width
                    .saturating_sub(content_prefix.len())
                    .saturating_sub(2);

                // Render content lines with manual wrapping
                for content_line in comment.content.lines() {
                    for wrapped_line in wrap_text(content_line, text_width) {
                        lines.push(Line::from(vec![
                            Span::styled(
                                content_prefix.clone(),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(wrapped_line, Style::default().fg(Color::White)),
                        ]));
                    }
                }

                lines.push(Line::from("")); // Spacing
            }

            // Recurse into replies if not collapsed
            if !is_collapsed && !comment.replies.is_empty() {
                prefix_stack.push(!is_last);
                flatten_comments(
                    &comment.replies,
                    lines,
                    depth + 1,
                    prefix_stack,
                    app,
                    selected_id,
                    visible_count,
                    available_width,
                );
                prefix_stack.pop();
            }
        }
    }

    let mut prefix_stack = Vec::new();
    flatten_comments(
        &app.comments,
        &mut lines,
        0,
        &mut prefix_stack,
        app,
        &selected_id,
        &mut visible_comment_count,
        area.width as usize,
    );

    // Apply scroll based on selected comment to keep it visible
    let visible_height = area.height.saturating_sub(2) as usize; // account for borders
    let lines_per_comment = 3; // header + avg content + spacing
    let selected_line_approx = app.selected_comment_index * lines_per_comment;

    let scroll = if selected_line_approx >= app.comment_scroll + visible_height {
        selected_line_approx.saturating_sub(visible_height / 2)
    } else if selected_line_approx < app.comment_scroll {
        selected_line_approx.saturating_sub(visible_height / 4)
    } else {
        app.comment_scroll
    };

    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll).collect();

    let comments_widget = Paragraph::new(visible_lines).block(
        Block::default()
            .title(format!(
                "Comments ({}) - j/k: nav, Enter: collapse",
                count_total_comments(&app.comments)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );

    frame.render_widget(comments_widget, area);
}

pub fn count_total_comments(comments: &[Comment]) -> usize {
    comments
        .iter()
        .fold(0, |acc, c| acc + 1 + count_total_comments(&c.replies))
}
