use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
};

pub fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn format_number_with_commas(n: i64) -> String {
    let s = n.abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    if n < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

pub fn humanize_number(n: i64) -> String {
    let abs = n.abs();
    let (value, suffix) = if abs >= 1_000_000 {
        (abs as f64 / 1_000_000.0, "M")
    } else if abs >= 1_000 {
        (abs as f64 / 1_000.0, "K")
    } else {
        return n.to_string();
    };

    let sign = if n < 0 { "-" } else { "" };
    if value >= 10.0 {
        format!("{}{:.0}{}", sign, value, suffix)
    } else {
        format!("{}{:.1}{}", sign, value, suffix)
    }
}

pub fn humanize_date(iso_date: &str) -> String {
    let parsed = DateTime::parse_from_rfc3339(iso_date)
        .or_else(|_| DateTime::parse_from_str(iso_date, "%Y-%m-%dT%H:%M:%S%.fZ"))
        .map(|dt| dt.with_timezone(&Utc));

    let Ok(date) = parsed else {
        return String::new();
    };

    let now = Utc::now();
    let duration = now.signed_duration_since(date);

    let seconds = duration.num_seconds();
    if seconds < 60 {
        return "just now".to_string();
    }

    let minutes = duration.num_minutes();
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }

    let hours = duration.num_hours();
    if hours < 24 {
        return format!("{}h ago", hours);
    }

    let days = duration.num_days();
    if days < 30 {
        return format!("{}d ago", days);
    }

    let months = days / 30;
    if months < 12 {
        return format!("{}mo ago", months);
    }

    format!("{}y ago", days / 365)
}

pub fn centered_fixed_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

pub fn format_follower_count(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Parse simple markdown (bold and italic) into styled spans.
/// Handles `**bold**` and `*italic*` syntax.
pub fn parse_simple_markdown(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '*' {
            // Check for ** (bold) or * (italic)
            if chars.peek() == Some(&'*') {
                chars.next(); // consume second *
                // Flush current text
                if !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        Style::default().fg(Color::White),
                    ));
                }
                // Collect bold text until **
                let mut bold_text = String::new();
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'*') {
                        chars.next();
                        break;
                    }
                    bold_text.push(c);
                }
                spans.push(Span::styled(
                    bold_text,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // Single * for italic - collect until next *
                if !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        Style::default().fg(Color::White),
                    ));
                }
                let mut italic_text = String::new();
                for c in chars.by_ref() {
                    if c == '*' {
                        break;
                    }
                    italic_text.push(c);
                }
                spans.push(Span::styled(
                    italic_text,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC),
                ));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        spans.push(Span::styled(current, Style::default().fg(Color::White)));
    }
    spans
}
