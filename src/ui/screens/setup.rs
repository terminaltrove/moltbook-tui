use crate::app::App;

use crate::ui::colors::{MOLTBOOK_RED, MOLTBOOK_TEAL};
use crate::ui::header::LOGO_ART;
use crate::ui::overlays::render_spinner;
use crate::ui::utils::centered_fixed_rect;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_setup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Center the setup form
    let popup_area = centered_fixed_rect(60, 18, area);

    frame.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Logo
            Constraint::Length(2), // Instructions
            Constraint::Length(3), // Input field
            Constraint::Length(2), // Error message
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(popup_area);

    // Logo
    let logo_lines: Vec<Line> = LOGO_ART
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
    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![Line::from(Span::styled(
        "Enter your API key to get started",
        Style::default().fg(Color::White),
    ))])
    .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[1]);

    // Input field with cursor
    let input_display = if app.api_key_input.is_empty() {
        Span::styled(
            "_",
            Style::default()
                .fg(MOLTBOOK_TEAL)
                .add_modifier(Modifier::SLOW_BLINK),
        )
    } else {
        // Mask the API key for security but show length
        let masked: String = "*".repeat(app.api_key_input.len());
        Span::styled(format!("{}_", masked), Style::default().fg(MOLTBOOK_TEAL))
    };

    let input_widget = Paragraph::new(Line::from(input_display))
        .block(
            Block::default()
                .title(" API Key ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MOLTBOOK_TEAL)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(input_widget, chunks[2]);

    // Error message
    if let Some(ref error) = app.setup_error {
        let error_widget = Paragraph::new(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(error_widget, chunks[3]);
    }

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(MOLTBOOK_TEAL)),
        Span::styled(" to save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(MOLTBOOK_TEAL)),
        Span::styled(" to quit", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[4]);

    // Outer border
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MOLTBOOK_RED))
        .title(" Setup ");
    frame.render_widget(border, popup_area);

    // Loading spinner overlay
    if app.is_loading {
        render_spinner(frame, app);
    }
}
