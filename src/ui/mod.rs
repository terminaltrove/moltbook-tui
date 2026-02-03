pub mod colors;
pub mod fonts;
pub mod header;
pub mod overlays;
pub mod screens;
pub mod utils;

use crate::app::{App, Screen};

use overlays::{render_about, render_agent_preview_sidebar, render_debug, render_help, render_spinner};
use screens::{
    render_agent_profile, render_feed, render_leaderboard, render_post_detail,
    render_recent_agents, render_settings, render_setup, render_stats, render_submolts,
    render_top_pairings,
};

use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &mut App) {
    // Store frame dimensions for mouse click handling
    app.last_frame_area = Some((frame.area().width, frame.area().height));

    match app.screen {
        Screen::Setup => render_setup(frame, app),
        Screen::Feed => render_feed(frame, app),
        Screen::PostDetail => render_post_detail(frame, app),
        Screen::Stats => render_stats(frame, app),
        Screen::Leaderboard => render_leaderboard(frame, app),
        Screen::TopPairings => render_top_pairings(frame, app),
        Screen::RecentAgents => render_recent_agents(frame, app),
        Screen::Submolts => render_submolts(frame, app),
        Screen::Settings => render_settings(frame, app),
        Screen::AgentProfile => render_agent_profile(frame, app),
    }

    // Render overlays on top (modal spinner only for navigation, not background refresh or preview loading)
    if app.is_loading && !app.is_background_loading && !app.is_preview_loading {
        render_spinner(frame, app);
    }

    if app.show_help {
        render_help(frame);
    }

    if app.show_about {
        render_about(frame);
    }

    // Agent preview sidebar
    if app.show_agent_preview {
        render_agent_preview_sidebar(frame, app);
    }

    // Debug overlay (toggle with backtick `)
    if app.debug_mode {
        render_debug(frame, app);
    }
}
