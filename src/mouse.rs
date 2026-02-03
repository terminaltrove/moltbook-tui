//! Mouse click handling for the TUI

use crate::api::{ApiClient, SortOrder, TimeFilter};
use crate::app::{App, Screen};
use crate::config::RowDisplay;
use crate::AppEvent;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Main entry point for mouse click handling
pub fn handle_mouse_click(
    app: &mut App,
    x: u16,
    y: u16,
    api_client: Arc<ApiClient>,
    tx: mpsc::Sender<AppEvent>,
) {
    app.add_debug(format!("Mouse click at ({}, {})", x, y));

    // Dismiss About modal on click
    if app.show_about {
        app.show_about = false;
        return;
    }

    // Skip if other modals are open
    if app.show_help || app.show_submolt_detail || app.show_agent_preview {
        return;
    }

    // Skip if error modal is shown
    if app.error_message.is_some() {
        return;
    }

    let (width, height) = app.last_frame_area.unwrap_or((80, 24));

    // Check nav tabs first (in header area)
    // Header is typically 11-13 lines depending on screen
    let header_height = get_header_height(app);

    if y > 0 && y < header_height {
        if let Some(action) = get_nav_tab_at_position(x, y, &app.screen) {
            handle_nav_action(app, action, api_client, tx);
            return;
        }
    }

    // Screen-specific handling
    match app.screen {
        Screen::Feed => handle_feed_click(app, x, y, width, height, api_client, tx),
        Screen::Leaderboard => handle_leaderboard_click(app, x, y, height),
        Screen::TopPairings => handle_top_pairings_click(app, x, y, height),
        Screen::RecentAgents => handle_recent_agents_click(app, x, y, height),
        Screen::Submolts => handle_submolts_click(app, x, y, width, height),
        Screen::PostDetail => handle_post_detail_click(app, x, y, height),
        Screen::Settings => handle_settings_click(app, y, height),
        _ => {}
    }
}

/// Get header height based on current screen
fn get_header_height(app: &App) -> u16 {
    match app.screen {
        Screen::Feed => 13, // Feed has sort tabs, making header taller
        Screen::PostDetail => 3, // PostDetail has minimal header
        _ => 11, // Most screens use shared header (11 lines)
    }
}

/// Navigation action from clicking tabs
#[derive(Debug, Clone, Copy)]
enum NavAction {
    Screen(u8), // 1-8 for screen numbers
    Sort(SortOrder),
    TimeFilter(TimeFilter),
    Shuffle,
}

/// Detect which nav tab was clicked based on x position
/// Returns the screen number (1-8) if a tab was clicked
fn get_nav_tab_at_position(x: u16, y: u16, current_screen: &Screen) -> Option<NavAction> {
    // Nav tabs are on line 9 (0-indexed) in the shared header
    // For Feed screen, sort tabs are on line 11-12

    // Shared header nav tabs line is at y=9 (inside the header box)
    // The actual y position depends on the border, so line 9 corresponds to y=9

    // Nav tabs format: " [1] Feed  [2] Leaderboard  [3] Top Pairings  [4] Agents  [5] Submolts  [6] Stats  [7] Settings  [8] About "
    // Approximate x positions for each tab (accounting for the format " [X] Label  "):
    // [1] Feed:        x = 1-11
    // [2] Leaderboard: x = 13-30
    // [3] Top Pairings: x = 32-51
    // [4] Agents:      x = 53-66
    // [5] Submolts:    x = 68-83
    // [6] Stats:       x = 85-97
    // [7] Settings:    x = 99-115
    // [8] About:       x = 117+

    // The nav tabs line is at y=9 within the header (0-indexed from top of terminal)
    if y == 9 {
        // Check each tab range
        if (1..=11).contains(&x) {
            return Some(NavAction::Screen(1));
        } else if (13..=30).contains(&x) {
            return Some(NavAction::Screen(2));
        } else if (32..=51).contains(&x) {
            return Some(NavAction::Screen(3));
        } else if (53..=66).contains(&x) {
            return Some(NavAction::Screen(4));
        } else if (68..=83).contains(&x) {
            return Some(NavAction::Screen(5));
        } else if (85..=97).contains(&x) {
            return Some(NavAction::Screen(6));
        } else if (99..=115).contains(&x) {
            return Some(NavAction::Screen(7));
        } else if x >= 117 {
            return Some(NavAction::Screen(8));
        }
    }

    // For Feed screen, check sort tabs on line 11
    if *current_screen == Screen::Feed && y == 11 {
        // Sort tabs format: " [N]ew    [T]op    [D]iscussed    [R]andom | [s]huffle  Hour  Day  Week  Month  Year  All  [f] cycle"
        // Approximate positions:
        // [N]ew:       x = 1-8
        // [T]op:       x = 13-19
        // [D]iscussed: x = 24-36
        // [R]andom:    x = 41-52
        // [s]huffle:   x = 56-68
        // Time filters start around x = 70+

        if (1..=8).contains(&x) {
            return Some(NavAction::Sort(SortOrder::New));
        } else if (13..=19).contains(&x) {
            return Some(NavAction::Sort(SortOrder::Top));
        } else if (24..=36).contains(&x) {
            return Some(NavAction::Sort(SortOrder::Discussed));
        } else if (41..=52).contains(&x) {
            return Some(NavAction::Sort(SortOrder::Random));
        } else if (56..=68).contains(&x) {
            return Some(NavAction::Shuffle);
        }
        // Time filter pills (only shown when not sorting by New)
        else if (72..=78).contains(&x) {
            return Some(NavAction::TimeFilter(TimeFilter::Hour));
        } else if (82..=87).contains(&x) {
            return Some(NavAction::TimeFilter(TimeFilter::Day));
        } else if (91..=97).contains(&x) {
            return Some(NavAction::TimeFilter(TimeFilter::Week));
        } else if (101..=108).contains(&x) {
            return Some(NavAction::TimeFilter(TimeFilter::Month));
        } else if (112..=118).contains(&x) {
            return Some(NavAction::TimeFilter(TimeFilter::Year));
        } else if (122..=127).contains(&x) {
            return Some(NavAction::TimeFilter(TimeFilter::All));
        }
    }

    None
}

/// Handle navigation action (screen switch, sort change, etc.)
fn handle_nav_action(
    app: &mut App,
    action: NavAction,
    api_client: Arc<ApiClient>,
    tx: mpsc::Sender<AppEvent>,
) {
    match action {
        NavAction::Screen(num) => {
            handle_screen_switch(app, num, api_client, tx);
        }
        NavAction::Sort(order) => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(order);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(app, api_client, tx);
            }
        }
        NavAction::TimeFilter(filter) => {
            if app.screen == Screen::Feed && !app.is_loading && app.sort_order != SortOrder::New {
                app.time_filter = filter;
                app.current_page = 0;
                app.is_loading = true;
                load_posts(app, api_client, tx);
            }
        }
        NavAction::Shuffle => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(SortOrder::Random);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(app, api_client, tx);
            }
        }
    }
}

/// Switch to screen based on number key (1-8)
fn handle_screen_switch(
    app: &mut App,
    screen_num: u8,
    api_client: Arc<ApiClient>,
    tx: mpsc::Sender<AppEvent>,
) {
    match screen_num {
        1 => {
            app.add_debug("-> Feed (click)".to_string());
            app.screen = Screen::Feed;
        }
        2 => {
            app.add_debug("-> Leaderboard (click)".to_string());
            app.screen = Screen::Leaderboard;
            if app.leaderboard.is_empty() {
                app.is_loading = true;
                load_leaderboard(api_client, tx);
            }
        }
        3 => {
            app.add_debug("-> TopPairings (click)".to_string());
            app.screen = Screen::TopPairings;
            if app.top_pairings.is_empty() {
                app.is_loading = true;
                load_top_pairings(api_client, tx);
            }
        }
        4 => {
            app.add_debug("-> RecentAgents (click)".to_string());
            app.screen = Screen::RecentAgents;
            if app.recent_agents.is_empty() {
                app.is_loading = true;
                load_recent_agents(api_client, tx);
            }
        }
        5 => {
            app.add_debug("-> Submolts (click)".to_string());
            app.screen = Screen::Submolts;
            if app.submolts.is_empty() {
                app.is_loading = true;
                load_submolts(api_client, tx);
            }
        }
        6 => {
            app.add_debug("-> Stats (click)".to_string());
            app.screen = Screen::Stats;
            if app.stats.is_none() {
                app.is_loading = true;
                load_stats(api_client, tx);
            }
        }
        7 => {
            app.add_debug("-> Settings (click)".to_string());
            app.screen = Screen::Settings;
        }
        8 => {
            app.toggle_about();
        }
        _ => {}
    }
}

/// Handle clicks in the Feed screen
fn handle_feed_click(
    app: &mut App,
    _x: u16,
    y: u16,
    _width: u16,
    height: u16,
    _api_client: Arc<ApiClient>,
    _tx: mpsc::Sender<AppEvent>,
) {
    // Feed layout:
    // - Header: 13 lines (including sort tabs)
    // - Posts list: from y=13 to y=height-3
    // - Footer: 3 lines

    let header_height = 13u16;
    let footer_height = 3u16;

    // Check if click is in posts area
    if y > header_height && y < height.saturating_sub(footer_height) {
        // Calculate which post was clicked
        let relative_y = y - header_height - 1; // -1 for border

        // Item height depends on row_display
        let item_height = match app.row_display {
            RowDisplay::Compact => 2u16,
            RowDisplay::Normal => 2u16,
            RowDisplay::Comfortable => 3u16,
        };

        let clicked_index = (relative_y / item_height) as usize;

        if clicked_index < app.posts.len() {
            app.selected_index = clicked_index;
            app.add_debug(format!("Selected post {}", clicked_index));
        }
    }
}

/// Handle clicks in the Leaderboard screen
fn handle_leaderboard_click(app: &mut App, _x: u16, y: u16, height: u16) {
    // Leaderboard layout:
    // - Header: 11 lines
    // - List: from y=11 to y=height-3
    // - Footer: 3 lines

    let header_height = 11u16;
    let footer_height = 3u16;

    if y > header_height && y < height.saturating_sub(footer_height) {
        let relative_y = y - header_height - 1;

        // Leaderboard items have variable height:
        // - Top 3: ~7 lines each (figlet art)
        // - Others: 4 lines each

        // This is an approximation - we'll use scroll position tracking
        // For simplicity, estimate based on position
        let mut accumulated_height = 0u16;
        for (i, agent) in app.leaderboard.iter().enumerate() {
            let item_height = if agent.rank <= 3 { 7u16 } else { 4u16 };

            if relative_y >= accumulated_height && relative_y < accumulated_height + item_height {
                app.leaderboard_selected = i;
                app.add_debug(format!("Selected leaderboard agent {}", i));
                return;
            }
            accumulated_height += item_height;
        }
    }
}

/// Handle clicks in the Top Pairings screen
fn handle_top_pairings_click(app: &mut App, _x: u16, y: u16, height: u16) {
    // Top Pairings layout:
    // - Header: 11 lines
    // - List: from y=11 to y=height-3
    // - Footer: 3 lines

    let header_height = 11u16;
    let footer_height = 3u16;

    if y > header_height && y < height.saturating_sub(footer_height) {
        let relative_y = y - header_height - 1;

        // Each pairing is 4 lines (handle, name, agent info, blank)
        let item_height = 4u16;
        let clicked_index = (relative_y / item_height) as usize;

        if clicked_index < app.top_pairings.len() {
            app.top_pairings_selected = clicked_index;
            app.add_debug(format!("Selected pairing {}", clicked_index));
        }
    }
}

/// Handle clicks in the Recent Agents screen
fn handle_recent_agents_click(app: &mut App, _x: u16, y: u16, height: u16) {
    // Recent Agents layout:
    // - Header: 11 lines
    // - List: from y=11 to y=height-3
    // - Footer: 3 lines

    let header_height = 11u16;
    let footer_height = 3u16;

    if y > header_height && y < height.saturating_sub(footer_height) {
        let relative_y = y - header_height - 1;

        // Item height depends on row_display
        let item_height = match app.row_display {
            RowDisplay::Compact => 3u16,
            RowDisplay::Normal => 4u16,
            RowDisplay::Comfortable => 4u16,
        };

        let clicked_index = (relative_y / item_height) as usize;

        if clicked_index < app.recent_agents.len() {
            app.recent_selected = clicked_index;
            app.add_debug(format!("Selected recent agent {}", clicked_index));
        }
    }
}

/// Handle clicks in the Submolts screen (4-column grid)
fn handle_submolts_click(app: &mut App, x: u16, y: u16, width: u16, height: u16) {
    // Submolts layout:
    // - Header: 11 lines
    // - Grid: from y=11 to y=height-3 (4 columns)
    // - Footer: 3 lines

    let header_height = 11u16;
    let footer_height = 3u16;

    if y > header_height && y < height.saturating_sub(footer_height) {
        let relative_y = y - header_height - 1;

        // Row height depends on row_display
        let row_height = match app.row_display {
            RowDisplay::Compact => 4u16,
            RowDisplay::Normal => 6u16,
            RowDisplay::Comfortable => 7u16,
        };

        // Grid is 4 columns
        let col_width = (width.saturating_sub(2)) / 4; // -2 for borders
        let relative_x = x.saturating_sub(1); // -1 for left border

        let clicked_row = (relative_y / row_height) as usize;
        let clicked_col = (relative_x / col_width).min(3) as usize;

        // Account for scroll offset
        let actual_row = app.submolts_scroll_row + clicked_row;
        let clicked_index = actual_row * 4 + clicked_col;

        if clicked_index < app.submolts.len() {
            app.submolts_selected = clicked_index;
            app.add_debug(format!(
                "Selected submolt {} (row {}, col {})",
                clicked_index, actual_row, clicked_col
            ));
        }
    }
}

/// Handle clicks in the Post Detail screen
fn handle_post_detail_click(app: &mut App, _x: u16, y: u16, height: u16) {
    // Post Detail layout:
    // - Header: 3 lines
    // - Post content: 15 lines
    // - Comments: from y=18 to y=height-3
    // - Footer: 3 lines

    let comments_start = 18u16;
    let footer_height = 3u16;

    if y > comments_start && y < height.saturating_sub(footer_height) {
        let relative_y = y - comments_start - 1;

        // Comments have variable height, but we estimate ~3 lines per comment
        let lines_per_comment = 3u16;
        let clicked_index = (relative_y / lines_per_comment) as usize;

        let visible_count = app.get_visible_comment_ids().len();
        if clicked_index < visible_count {
            app.selected_comment_index = clicked_index;
            app.add_debug(format!("Selected comment {}", clicked_index));
        }
    }
}

/// Handle clicks in the Settings screen
fn handle_settings_click(app: &mut App, y: u16, _height: u16) {
    // Settings layout:
    // - Header: 11 lines
    // - Settings list starts at y=12

    let header_height = 11u16;

    if y > header_height {
        let relative_y = y - header_height - 1;

        // Each setting is ~2 lines
        let item_height = 2u16;
        let clicked_index = (relative_y / item_height) as usize;

        // Only 2 settings currently
        if clicked_index <= 1 {
            app.settings_selected = clicked_index;
            app.add_debug(format!("Selected setting {}", clicked_index));
        }
    }
}

// Helper functions to load data (mirrors main.rs functions)

const POSTS_LIMIT: i64 = 25;

fn load_posts(app: &App, api_client: Arc<ApiClient>, tx: mpsc::Sender<AppEvent>) {
    let sort = app.sort_order;
    let time_filter = app.time_filter_for_api();
    let offset = app.current_page as i64 * POSTS_LIMIT;
    let submolt = app.current_submolt.as_ref().map(|s| s.name.clone());

    tokio::spawn(async move {
        let submolt_str = submolt.as_deref();
        match api_client
            .get_posts(sort, time_filter, POSTS_LIMIT, offset, submolt_str)
            .await
        {
            Ok(response) => {
                let has_more = response.posts.len() as i64 == POSTS_LIMIT;
                let _ = tx
                    .send(AppEvent::PostsLoaded(response.posts, has_more))
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!("Failed to load posts: {}", e)))
                    .await;
            }
        }
    });
}

fn load_leaderboard(api_client: Arc<ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match api_client.get_leaderboard().await {
            Ok(leaderboard) => {
                let _ = tx.send(AppEvent::LeaderboardLoaded(leaderboard)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!(
                        "Failed to load leaderboard: {}",
                        e
                    )))
                    .await;
            }
        }
    });
}

fn load_top_pairings(api_client: Arc<ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match api_client.get_top_humans().await {
            Ok(top_humans) => {
                let _ = tx.send(AppEvent::TopPairingsLoaded(top_humans)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!(
                        "Failed to load top pairings: {}",
                        e
                    )))
                    .await;
            }
        }
    });
}

fn load_recent_agents(api_client: Arc<ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match api_client.get_recent_agents().await {
            Ok(agents) => {
                let _ = tx.send(AppEvent::RecentAgentsLoaded(agents)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!(
                        "Failed to load recent agents: {}",
                        e
                    )))
                    .await;
            }
        }
    });
}

fn load_submolts(api_client: Arc<ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match api_client.get_submolts().await {
            Ok(submolts) => {
                let _ = tx.send(AppEvent::SubmoltsLoaded(submolts)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!("Failed to load submolts: {}", e)))
                    .await;
            }
        }
    });
}

fn load_stats(api_client: Arc<ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match api_client.get_stats().await {
            Ok(stats) => {
                let _ = tx.send(AppEvent::StatsLoaded(stats)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!("Failed to load stats: {}", e)))
                    .await;
            }
        }
    });
}
