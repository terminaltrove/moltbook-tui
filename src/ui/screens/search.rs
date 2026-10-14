use crate::api::{SearchResult, SearchType};
use crate::app::{App, Screen};
use crate::config::RowDisplay;

use crate::ui::colors::{MOLTBOOK_BLUE, MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use crate::ui::header::render_shared_header;
use crate::ui::overlays::render_error;
use crate::ui::utils::{
    format_number_with_commas, highlight_spans, humanize_date, strip_highlights,
};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const INDENT: &str = "          ";

pub fn render_search(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Logo + tagline + stats + nav tabs
            Constraint::Length(4),  // Query + result type
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new(render_shared_header(Screen::Search, app)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MOLTBOOK_RED)),
    );
    frame.render_widget(header, chunks[0]);

    render_search_box(frame, app, chunks[1]);
    render_results(frame, app, chunks[2]);

    let hints = if app.search_editing {
        "Enter: Search • Tab: Result type • Esc: Done typing"
    } else {
        "j/k: Nav • Enter: Open • Tab: Result type • /: Edit query • N: More • o: Browser • Esc: Back"
    };
    let footer = Paragraph::new(hints)
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

fn render_search_box(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut query = vec![Span::styled(" / ", Style::default().fg(Color::DarkGray))];
    if app.search_query.is_empty() && !app.search_editing {
        query.push(Span::styled(
            "press / to search posts, comments, agents and submolts",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        query.push(Span::styled(
            app.search_query.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if app.search_editing {
        query.push(Span::styled("▌", Style::default().fg(MOLTBOOK_TEAL)));
    }

    let mut types = vec![Span::raw(" ")];
    for kind in SearchType::ALL {
        let label = format!(" {} ", kind.label());
        types.push(if kind == app.search_type {
            Span::styled(
                label,
                Style::default()
                    .fg(Color::Rgb(0, 0, 0))
                    .bg(MOLTBOOK_TEAL)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(label, Style::default().fg(Color::DarkGray))
        });
        types.push(Span::raw(" "));
    }
    types.push(Span::styled(
        "  Tab: change",
        Style::default().fg(Color::DarkGray),
    ));

    let border = if app.search_editing {
        MOLTBOOK_TEAL
    } else {
        MOLTBOOK_RED
    };
    let search_box = Paragraph::new(vec![Line::from(query), Line::from(types)]).block(
        Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border)),
    );
    frame.render_widget(search_box, area);
}

fn render_results(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = match &app.search_submitted {
        None => " Results ".to_string(),
        Some(_) if app.search_results.is_empty() && app.is_loading => " Searching… ".to_string(),
        Some(q) => format!(
            " {} result{} for \"{}\"{} ",
            app.search_results.len(),
            if app.search_results.len() == 1 {
                ""
            } else {
                "s"
            },
            q,
            if app.search_has_more {
                " · N for more"
            } else {
                ""
            }
        ),
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED));

    if app.search_results.is_empty() {
        let message = match &app.search_submitted {
            None => vec![
                Line::from("Type a query and press Enter."),
                Line::from(""),
                Line::from(Span::styled(
                    "\"exact phrase\"   a or b   -excluded",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
            Some(_) if app.is_loading => vec![Line::from("Searching…")],
            Some(_) => vec![Line::from("No results.")],
        };
        let empty = Paragraph::new(message)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let width = area.width.saturating_sub(2 + INDENT.len() as u16 + 2) as usize;
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, hit)| result_item(hit, i == app.search_selected, width, app.row_display))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));
    let mut state = ListState::default();
    state.select(Some(app.search_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn result_item(
    hit: &SearchResult,
    selected: bool,
    width: usize,
    rows: RowDisplay,
) -> ListItem<'static> {
    let (badge, colour) = match hit.kind.as_str() {
        "post" => ("post", MOLTBOOK_RED),
        "comment" => ("comment", MOLTBOOK_BLUE),
        "agent" => ("agent", MOLTBOOK_TEAL),
        "submolt" => ("submolt", MOLTBOOK_YELLOW),
        other => (other, Color::DarkGray),
    };
    let title_style = if selected {
        Style::default()
            .fg(MOLTBOOK_TEAL)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };
    let hl = Style::default()
        .fg(MOLTBOOK_TEAL)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(Color::DarkGray);

    let title = match hit.kind.as_str() {
        "comment" => format!(
            "on: {}",
            hit.post
                .as_ref()
                .and_then(|p| p.title.as_deref())
                .unwrap_or("a post")
        ),
        "agent" => format!("u/{}", hit.title.as_deref().unwrap_or_default()),
        "submolt" => match (&hit.submolt, &hit.title) {
            (Some(s), Some(t)) if !t.eq_ignore_ascii_case(&s.name) => {
                format!("m/{} · {}", s.name, t)
            }
            (Some(s), _) => format!("m/{}", s.name),
            (None, t) => t.clone().unwrap_or_default(),
        },
        _ => hit.title.clone().unwrap_or_default(),
    };
    let mut first = vec![
        Span::raw(" "),
        Span::styled(format!("{:<9}", badge), Style::default().fg(colour)),
    ];
    first.extend(highlight_spans(&title, width, title_style, title_style));

    let mut snippet = vec![Span::raw(INDENT)];
    snippet.extend(highlight_spans(
        hit.content.as_deref().unwrap_or_default(),
        width,
        Style::default().fg(Color::Gray),
        hl,
    ));

    let mut meta: Vec<String> = Vec::new();
    match hit.kind.as_str() {
        "agent" => {}
        "submolt" => meta.push(format!(
            "{} members",
            format_number_with_commas(hit.upvotes)
        )),
        _ => {
            if let Some(s) = &hit.submolt {
                meta.push(format!("m/{}", s.name));
            }
            if let Some(a) = &hit.author {
                meta.push(format!("u/{}", a.name));
            }
            meta.push(format!("▲ {}", hit.upvotes - hit.downvotes));
        }
    }
    if let Some(when) = &hit.created_at {
        meta.push(humanize_date(when));
    }
    let meta_line = Line::from(vec![
        Span::raw(INDENT),
        Span::styled(strip_highlights(&meta.join(" • ")), dim),
    ]);

    let lines = match rows {
        RowDisplay::Compact => vec![Line::from(first), meta_line],
        _ => vec![
            Line::from(first),
            Line::from(snippet),
            meta_line,
            Line::from(""),
        ],
    };
    ListItem::new(lines)
}
