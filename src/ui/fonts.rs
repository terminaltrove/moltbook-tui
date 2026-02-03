use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Deserialize)]
pub struct FontData {
    pub characters: HashMap<String, Vec<String>>,
}

pub static FONT_LARGE: LazyLock<FontData> = LazyLock::new(|| {
    let json = include_str!("../fonts/pixeloidbold.bit");
    serde_json::from_str(json).expect("Failed to parse pixeloidbold font")
});

pub const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn get_block_char(c: char) -> Vec<&'static str> {
    let key = c.to_uppercase().to_string();
    FONT_LARGE
        .characters
        .get(&key)
        .map(|lines| lines.iter().map(|s| s.as_str()).collect())
        .unwrap_or_else(|| vec![""; 7])
}

/// Render a name using block characters
pub fn render_figlet_name(name: &str, max_chars: usize, half_scale: bool) -> Vec<String> {
    let chars: Vec<char> = name.to_uppercase().chars().take(max_chars).collect();
    let char_bitmaps: Vec<Vec<&str>> = chars.iter().map(|&c| get_block_char(c)).collect();

    let max_height = char_bitmaps.iter().map(|b| b.len()).max().unwrap_or(7);

    let gap = "  "; // Always 2-space gap between characters

    let lines: Vec<String> = (0..max_height)
        .map(|row| {
            char_bitmaps
                .iter()
                .map(|bitmap| bitmap.get(row).copied().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(gap)
        })
        .collect();

    if half_scale {
        scale_figlet_half(lines)
    } else {
        lines
    }
}

/// Scale figlet output to 0.5x using half-block characters (both horizontal and vertical)
pub fn scale_figlet_half(lines: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    // Process pairs of rows (vertical scaling)
    for chunk in lines.chunks(2) {
        let top_row = chunk.first().map(|s| s.as_str()).unwrap_or("");
        let bot_row = chunk.get(1).map(|s| s.as_str()).unwrap_or("");

        let top_chars: Vec<char> = top_row.chars().collect();
        let bot_chars: Vec<char> = bot_row.chars().collect();
        let max_len = top_chars.len().max(bot_chars.len());

        let mut scaled_row = String::new();
        // Process pairs of columns (horizontal scaling)
        let mut col = 0;
        while col < max_len {
            let top_left = top_chars.get(col).copied().unwrap_or(' ');
            let top_right = top_chars.get(col + 1).copied().unwrap_or(' ');
            let bot_left = bot_chars.get(col).copied().unwrap_or(' ');
            let bot_right = bot_chars.get(col + 1).copied().unwrap_or(' ');

            // Combine 2x2 block into single half-block character
            let top_filled = top_left == '█' || top_right == '█';
            let bot_filled = bot_left == '█' || bot_right == '█';

            scaled_row.push(match (top_filled, bot_filled) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
            col += 2;
        }
        result.push(scaled_row);
    }
    result
}
