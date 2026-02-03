use crate::api::{SortOrder, TimeFilter};
use crate::app::{App, Screen};

use super::colors::{MOLTBOOK_BLUE, MOLTBOOK_RED, MOLTBOOK_TEAL, MOLTBOOK_YELLOW};
use super::utils::format_count;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub const LOGO_ART: &[&str] = &[
    "                 ██  ██  ██                   ██   ",
    " ██▀██▀█▄ ▄█▀▀█▄ ██ ▀██▀ ██▀▀█▄ ▄█▀▀█▄ ▄█▀▀█▄ ██▄█▀",
    " ██ ██ ██ ██  ██ ██  ██  ██  ██ ██  ██ ██  ██ ███▄ ",
    " ▀▀ ▀▀ ▀▀  ▀▀▀▀  ▀▀  ▀▀▀ ▀▀▀▀▀   ▀▀▀▀   ▀▀▀▀  ▀▀ ▀▀",
];

pub fn render_sort_tabs_line(current: SortOrder, time_filter: Option<TimeFilter>) -> Line<'static> {
    let make_sort_tab = |order: SortOrder, key: char| -> Span<'static> {
        let is_active = current == order;
        let label = format!(" [{}]{} ", key.to_uppercase(), &format!("{}", order)[1..]);

        if is_active {
            Span::styled(
                label,
                Style::default()
                    .fg(Color::Rgb(0, 0, 0))
                    .bg(MOLTBOOK_RED)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(label, Style::default().fg(Color::DarkGray))
        }
    };

    let mut spans = vec![
        Span::raw(" "),
        make_sort_tab(SortOrder::New, 'n'),
        Span::raw("    "),
        make_sort_tab(SortOrder::Top, 't'),
        Span::raw("    "),
        make_sort_tab(SortOrder::Discussed, 'd'),
        Span::raw("    "),
        make_sort_tab(SortOrder::Random, 'R'),
    ];

    // Always show shuffle pill
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(
        " [s]huffle ",
        Style::default().fg(Color::Rgb(0, 0, 0)).bg(MOLTBOOK_TEAL),
    ));

    // Add time filter tabs if provided (not shown for "New" sort)
    if let Some(current_time) = time_filter {
        let make_time_tab = |filter: TimeFilter| -> Span<'static> {
            let is_active = current_time == filter;
            let label = format!(" {} ", filter);

            if is_active {
                Span::styled(
                    label,
                    Style::default()
                        .fg(Color::Rgb(0, 0, 0))
                        .bg(MOLTBOOK_RED)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(label, Style::default().fg(Color::DarkGray))
            }
        };

        spans.push(Span::raw("  "));
        spans.push(make_time_tab(TimeFilter::Hour));
        spans.push(Span::raw("  "));
        spans.push(make_time_tab(TimeFilter::Day));
        spans.push(Span::raw("  "));
        spans.push(make_time_tab(TimeFilter::Week));
        spans.push(Span::raw("  "));
        spans.push(make_time_tab(TimeFilter::Month));
        spans.push(Span::raw("  "));
        spans.push(make_time_tab(TimeFilter::Year));
        spans.push(Span::raw("  "));
        spans.push(make_time_tab(TimeFilter::All));
        spans.push(Span::styled(
            "  [f] cycle",
            Style::default().fg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

pub fn render_nav_tabs_line(current_screen: Screen) -> Line<'static> {
    let make_tab = |key: char, label: &str, screen: Screen| -> Span<'static> {
        let is_active = current_screen == screen;
        let text = format!(" [{}] {} ", key, label);

        if is_active {
            Span::styled(
                text,
                Style::default()
                    .fg(Color::Rgb(0, 0, 0))
                    .bg(MOLTBOOK_RED)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(text, Style::default().fg(Color::DarkGray))
        }
    };

    Line::from(vec![
        Span::raw(" "),
        make_tab('1', "Feed", Screen::Feed),
        Span::raw("  "),
        make_tab('2', "Leaderboard", Screen::Leaderboard),
        Span::raw("  "),
        make_tab('3', "Top Pairings", Screen::TopPairings),
        Span::raw("  "),
        make_tab('4', "Agents", Screen::RecentAgents),
        Span::raw("  "),
        make_tab('5', "Submolts", Screen::Submolts),
        Span::raw("  "),
        make_tab('6', "Stats", Screen::Stats),
        Span::raw("  "),
        make_tab('7', "Settings", Screen::Settings),
        Span::raw("  "),
        Span::styled(" [8] About ", Style::default().fg(Color::DarkGray)),
    ])
}

pub fn build_stats_line(app: &App) -> Line<'static> {
    if let Some(ref stats) = app.stats {
        Line::from(vec![
            Span::styled(" ", Style::default()),
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
        Line::from(Span::styled(
            " Loading stats...",
            Style::default().fg(Color::DarkGray),
        ))
    }
}

pub fn render_shared_header(current_screen: Screen, app: &App) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = LOGO_ART
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

    lines.push(Line::from(Span::styled(
        " the front page of the agent internet",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(build_stats_line(app));
    lines.push(Line::from(""));
    lines.push(render_nav_tabs_line(current_screen));

    lines
}
