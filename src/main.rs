mod api;
mod app;
mod config;
mod mouse;
mod ui;

use anyhow::Result;
use api::{SearchType, TimeFilter};
use app::{App, Screen};
use clap::{builder::Styles, Parser};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Create custom color styles for help output
fn styles() -> Styles {
    Styles::styled()
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Rgb(anstyle::RgbColor(224, 27, 36)))),
        )
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Rgb(anstyle::RgbColor(224, 27, 36)))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Rgb(anstyle::RgbColor(224, 27, 36)))),
        )
        .placeholder(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite))),
        )
}

/// A TUI client for moltbook, the social network for AI Agents.
#[derive(Parser)]
#[command(name = "moltbook")]
#[command(version, about, long_about = None)]
#[command(styles = styles())]
#[command(after_help = "EXAMPLES:\n  \
    # Launch TUI\n  \
    moltbook\n\n  \
    # Launch with auto-refresh disabled\n  \
    moltbook --no-refresh\n\n\
    For more information, visit: https://github.com/terminaltrove/moltbook-tui")]
struct Cli {
    /// Disable auto-refresh on startup
    #[arg(long)]
    no_refresh: bool,
}

const REFRESH_INTERVAL_SECS: u64 = 30;

fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let browser = std::env::var("BROWSER").unwrap_or_else(|_| "xdg-open".to_string());
        let _ = std::process::Command::new(browser).arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
}
const POSTS_LIMIT: i64 = 25;
/// Top-level comments to read per post; replies come nested and are not counted.
const MAX_COMMENTS: usize = 500;

#[derive(Debug)]
enum AppEvent {
    Input(KeyCode),
    MouseClick(u16, u16),                              // (x, y) coordinates
    PostsLoaded(Vec<api::Post>, bool, Option<String>), // (posts, has_more, next_cursor)
    CommentsLoaded(Vec<api::Comment>),
    StatsLoaded(api::Stats),
    LeaderboardLoaded(Vec<api::LeaderboardAgent>),
    TopPairingsLoaded(Vec<api::TopHuman>),
    RecentAgentsLoaded(Vec<api::RecentAgent>),
    SubmoltsLoaded(Vec<api::SubmoltFull>),
    AgentProfileLoaded(api::AgentProfileResponse),
    AgentPreviewLoaded(api::AgentProfileResponse), // Preview update (keeps sidebar open)
    PostLoaded(api::Post),
    SearchLoaded {
        query: String,
        kind: SearchType,
        results: Vec<api::SearchResult>,
        has_more: bool,
        next_cursor: Option<String>,
        append: bool,
    },
    ConfigSaved(Result<config::Config, String>),
    Error(String),
    Debug(String),
    Tick,
    SpinnerTick,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config - always succeeds, api_key may be None
    let config = config::Config::load().unwrap_or_else(|_| config::Config {
        api_key: None,
        api_url: "https://moltbook.com/api/v1".to_string(),
        row_display: config::RowDisplay::default(),
        refresh_interval_secs: 10,
    });

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = crossterm::cursor::Show;
        // Then call the original panic handler
        original_hook(panic_info);
    }));

    // Create app - go directly to feed (no setup needed for read-only)
    let mut app = App::new();
    app.refresh_interval_secs = if cli.no_refresh {
        0
    } else {
        config.refresh_interval_secs
    };
    app.row_display = config.row_display;

    // Create API client (auth is optional for read-only endpoints)
    let api_client = Arc::new(api::ApiClient::new(
        config.api_url.clone(),
        config.api_key.clone(),
    ));

    // Create event channel
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);

    // Create shutdown channel for graceful task termination
    let (shutdown_tx, _shutdown_rx) = watch::channel(false);

    // Spawn input handler with graceful shutdown support
    let input_tx = tx.clone();
    let mut shutdown_rx_input = shutdown_tx.subscribe();
    let input_handle = tokio::spawn(async move {
        loop {
            // Use spawn_blocking for sync event polling to avoid blocking the async runtime
            let event_result = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(16)).unwrap_or(false) {
                    event::read().ok()
                } else {
                    None
                }
            });

            tokio::select! {
                result = event_result => {
                    match result {
                        Ok(Some(Event::Key(key))) => {
                            if key.kind == KeyEventKind::Press {
                                let _ = input_tx.send(AppEvent::Input(key.code)).await;
                            }
                        }
                        Ok(Some(Event::Mouse(mouse_event))) => {
                            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse_event.kind {
                                let _ = input_tx.send(AppEvent::MouseClick(mouse_event.column, mouse_event.row)).await;
                            }
                        }
                        Ok(Some(Event::Resize(_, _))) => {
                            // Consume resize events (ignore)
                        }
                        Ok(Some(Event::FocusGained | Event::FocusLost)) => {
                            // Consume focus events (ignore)
                        }
                        Ok(Some(Event::Paste(_))) => {
                            // Consume paste events (ignore)
                        }
                        _ => {}
                    }
                }
                _ = shutdown_rx_input.changed() => {
                    break;
                }
            }
        }
    });

    // Spawn tick handler for auto-refresh
    let tick_tx = tx.clone();
    let tick_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let _ = tick_tx.send(AppEvent::Tick).await;
        }
    });

    // Spawn spinner animation tick handler
    let spinner_tx = tx.clone();
    let spinner_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(80));
        loop {
            interval.tick().await;
            let _ = spinner_tx.send(AppEvent::SpinnerTick).await;
        }
    });

    // Initial load (works without auth for read-only endpoints)
    app.is_loading = true;
    load_posts(
        api_client.clone(),
        app.sort_order,
        app.time_filter_for_api(),
        None,
        None,
        tx.clone(),
    );
    load_stats(api_client.clone(), tx.clone());

    // Mutable API client for setup flow
    let mut api_client = api_client;

    // Main loop
    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if app.should_quit {
            break;
        }

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Input(key) => {
                    handle_input(&mut app, key, api_client.clone(), tx.clone());
                }
                AppEvent::MouseClick(x, y) => {
                    mouse::handle_mouse_click(&mut app, x, y, api_client.clone(), tx.clone());
                }
                AppEvent::PostsLoaded(posts, has_more, next_cursor) => {
                    app.is_loading = false;
                    app.is_background_loading = false;
                    app.error_message = None;
                    app.has_more_posts = has_more && next_cursor.is_some();
                    app.record_next_cursor(next_cursor);
                    app.update_posts(posts);
                }
                AppEvent::CommentsLoaded(comments) => {
                    app.is_loading = false;
                    app.error_message = None;
                    app.comments = comments;
                }
                AppEvent::StatsLoaded(stats) => {
                    app.stats = Some(stats);
                }
                AppEvent::LeaderboardLoaded(leaderboard) => {
                    app.is_loading = false;
                    app.error_message = None;
                    app.leaderboard = leaderboard;
                }
                AppEvent::TopPairingsLoaded(top_pairings) => {
                    app.is_loading = false;
                    app.error_message = None;
                    app.top_pairings = top_pairings;
                }
                AppEvent::RecentAgentsLoaded(agents) => {
                    app.is_loading = false;
                    app.error_message = None;
                    app.recent_agents = agents;
                }
                AppEvent::SubmoltsLoaded(mut submolts) => {
                    app.is_loading = false;
                    app.error_message = None;
                    // Sort submolts: featured first, then by subscriber count
                    submolts.sort_by(|a, b| {
                        match (&a.featured_at, &b.featured_at) {
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            _ => b.subscriber_count.cmp(&a.subscriber_count),
                        }
                    });
                    app.submolts = submolts;
                }
                AppEvent::AgentProfileLoaded(response) => {
                    app.is_loading = false;
                    app.error_message = None;
                    app.agent_profile = Some(response.agent);
                    app.agent_posts = response.recent_posts;
                    app.agent_posts_selected = 0;
                    // If we were previewing, now show the full profile screen
                    if app.show_agent_preview {
                        app.show_agent_preview = false;
                        app.preview_agent_name = None;
                    }
                }
                AppEvent::AgentPreviewLoaded(response) => {
                    // Preview update - keep sidebar open, just update the profile data
                    app.is_loading = false;
                    app.is_preview_loading = false;
                    app.error_message = None;
                    app.agent_profile = Some(response.agent);
                    app.agent_posts = response.recent_posts;
                    app.agent_posts_selected = 0;
                }
                AppEvent::ConfigSaved(result) => {
                    app.is_loading = false;
                    match result {
                        Ok(cfg) => {
                            // Update API client with new config
                            api_client = Arc::new(api::ApiClient::new(cfg.api_url, cfg.api_key));
                            // Switch to feed and load data
                            app.screen = Screen::Feed;
                            app.setup_error = None;
                            app.is_loading = true;
                            load_posts(
                                api_client.clone(),
                                app.sort_order,
                                app.time_filter_for_api(),
                                None,
                                app.current_submolt.as_ref().map(|s| s.name.clone()),
                                tx.clone(),
                            );
                            load_stats(api_client.clone(), tx.clone());
                        }
                        Err(e) => {
                            app.setup_error = Some(e);
                        }
                    }
                }
                AppEvent::PostLoaded(post) => {
                    // Only if the reader is still on the post this was fetched for.
                    if app.current_post.as_ref().map(|p| &p.id) == Some(&post.id) {
                        app.current_post = Some(post);
                    }
                }
                AppEvent::SearchLoaded {
                    query,
                    kind,
                    results,
                    has_more,
                    next_cursor,
                    append,
                } => {
                    app.is_loading = false;
                    app.is_background_loading = false;
                    if app.apply_search_results(
                        &query,
                        kind,
                        results,
                        has_more,
                        next_cursor,
                        append,
                    ) {
                        app.error_message = None;
                    }
                }
                AppEvent::Error(msg) => {
                    app.is_loading = false;
                    app.error_message = Some(msg.clone());
                    app.add_debug(format!("ERROR: {}", msg));
                }
                AppEvent::Debug(msg) => {
                    app.add_debug(msg);
                }
                AppEvent::Tick => {
                    // Only refresh if enabled and enough time has passed
                    let should_refresh =
                        app.refresh_interval_secs > 0 && app.seconds_until_refresh() == 0;
                    if app.screen == Screen::Feed && !app.is_loading && should_refresh {
                        app.is_loading = true;
                        app.is_background_loading = true;
                        let cursor = app.page_cursor();
                        load_posts(
                            api_client.clone(),
                            app.sort_order,
                            app.time_filter_for_api(),
                            cursor,
                            app.current_submolt.as_ref().map(|s| s.name.clone()),
                            tx.clone(),
                        );
                    }
                }
                AppEvent::SpinnerTick => {
                    if app.is_loading || app.is_preview_loading {
                        app.advance_spinner();
                    }
                }
            }
        }
    }

    // Signal shutdown to input handler for graceful termination
    let _ = shutdown_tx.send(true);

    // Give the input handler time to clean up, then abort all tasks
    tokio::time::sleep(Duration::from_millis(100)).await;
    input_handle.abort();
    tick_handle.abort();
    spinner_handle.abort();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_input(
    app: &mut App,
    key: KeyCode,
    _api_client: Arc<api::ApiClient>,
    tx: mpsc::Sender<AppEvent>,
) {
    app.add_debug(format!("Key: {:?}", key));

    // Setup screen handles input differently
    if app.screen == Screen::Setup {
        match key {
            KeyCode::Char(c) => {
                app.api_key_input.push(c);
                app.setup_error = None;
            }
            KeyCode::Backspace => {
                app.api_key_input.pop();
                app.setup_error = None;
            }
            KeyCode::Enter => {
                if app.api_key_input.trim().is_empty() {
                    app.setup_error = Some("API key cannot be empty".to_string());
                } else {
                    app.is_loading = true;
                    app.setup_error = None;
                    let api_key = app.api_key_input.trim().to_string();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let result = config::Config::save(&api_key);
                        let event = match result {
                            Ok(cfg) => AppEvent::ConfigSaved(Ok(cfg)),
                            Err(e) => AppEvent::ConfigSaved(Err(e.to_string())),
                        };
                        let _ = tx_clone.send(event).await;
                    });
                }
            }
            KeyCode::Esc => {
                app.should_quit = true;
            }
            _ => {}
        }
        return;
    }

    // Help menu takes priority - close it on any key
    if app.show_help {
        if key == KeyCode::Char('?') || key == KeyCode::Esc {
            app.toggle_help();
        }
        return;
    }

    // About modal takes priority
    if app.show_about {
        if key == KeyCode::Char('8') || key == KeyCode::Esc {
            app.toggle_about();
        }
        return;
    }

    // Use api_client for data loading
    let api_client = _api_client;

    // Submolt detail modal - handle its keys
    if app.show_submolt_detail {
        match key {
            KeyCode::Char(' ') | KeyCode::Esc => {
                app.show_submolt_detail = false;
            }
            _ => {}
        }
        return;
    }

    // Agent preview sidebar - handle its keys but allow navigation
    if app.show_agent_preview {
        match key {
            KeyCode::Tab | KeyCode::Esc => {
                app.show_agent_preview = false;
                app.preview_agent_name = None;
                return;
            }
            KeyCode::Enter => {
                // Open full agent profile
                app.show_agent_preview = false;
                app.preview_agent_name = None;
                app.previous_screen = Some(app.screen.clone());
                app.screen = Screen::AgentProfile;
                // Profile data is already loaded in app.agent_profile
                return;
            }
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('k') | KeyCode::Up => {
                // Allow navigation - fall through to normal handling
            }
            _ => {
                return;
            }
        }
    }

    // Error modal takes priority - handle its keys
    if app.error_message.is_some() {
        match key {
            KeyCode::Char('e') => {
                app.show_technical_error = !app.show_technical_error;
            }
            KeyCode::Char('r') => {
                if app.screen == Screen::Feed {
                    app.error_message = None;
                    app.show_technical_error = false;
                    app.is_loading = true;
                    let cursor = app.page_cursor();
                    load_posts(
                        api_client.clone(),
                        app.sort_order,
                        app.time_filter_for_api(),
                        cursor,
                        app.current_submolt.as_ref().map(|s| s.name.clone()),
                        tx.clone(),
                    );
                }
            }
            KeyCode::Esc => {
                app.error_message = None;
                app.show_technical_error = false;
            }
            _ => {}
        }
        return;
    }

    // Typing a search query: every key goes into the query until Enter or Esc, so
    // letters that are shortcuts elsewhere (q, n, j...) can be searched for.
    if app.screen == Screen::Search && app.search_editing {
        match key {
            KeyCode::Char(c) => app.search_query.push(c),
            KeyCode::Backspace => {
                app.search_query.pop();
            }
            KeyCode::Enter => run_search(app, api_client, tx),
            KeyCode::Tab => {
                app.search_type = app.search_type.cycle_next();
                if app.search_submitted.is_some() {
                    run_search(app, api_client, tx);
                    app.search_editing = true;
                }
            }
            KeyCode::Down => {
                if !app.search_results.is_empty() {
                    app.search_editing = false;
                }
            }
            KeyCode::Esc => {
                app.search_editing = false;
                if app.search_submitted.is_none() && app.search_query.is_empty() {
                    app.go_back();
                }
            }
            _ => {}
        }
        return;
    }

    match key {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('/') => {
            app.open_search();
        }
        KeyCode::Char('?') => {
            app.toggle_help();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let was_at_last = app.selected_index == app.posts.len().saturating_sub(1);
            app.select_next();
            // If we were at last post and there's more, load next page
            if app.screen == Screen::Feed
                && was_at_last
                && app.has_more_posts
                && !app.is_loading
                && !app.posts.is_empty()
            {
                app.next_page();
                app.is_loading = true;
                let cursor = app.page_cursor();
                load_posts(
                    api_client.clone(),
                    app.sort_order,
                    app.time_filter_for_api(),
                    cursor,
                    app.current_submolt.as_ref().map(|s| s.name.clone()),
                    tx.clone(),
                );
            }
            // Reaching the last search result fetches the next page beneath it.
            if app.screen == Screen::Search && app.search_selected + 1 >= app.search_results.len() {
                load_more_search(app, api_client.clone(), tx.clone());
            }
            // Update agent preview if sidebar is open
            if app.show_agent_preview {
                update_agent_preview_for_current_selection(app, api_client.clone(), tx.clone());
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let was_at_first = app.selected_index == 0;
            app.select_previous();
            // If we were at first post and there's a previous page, load it
            if app.screen == Screen::Feed
                && was_at_first
                && app.current_page > 0
                && !app.is_loading
                && !app.posts.is_empty()
            {
                app.prev_page();
                app.is_loading = true;
                let cursor = app.page_cursor();
                load_posts(
                    api_client.clone(),
                    app.sort_order,
                    app.time_filter_for_api(),
                    cursor,
                    app.current_submolt.as_ref().map(|s| s.name.clone()),
                    tx.clone(),
                );
            }
            // Update agent preview if sidebar is open
            if app.show_agent_preview {
                update_agent_preview_for_current_selection(app, api_client.clone(), tx.clone());
            }
        }
        KeyCode::Char('h') => {
            if app.screen == Screen::Submolts {
                app.select_left();
            }
        }
        KeyCode::Char('l') => {
            if app.screen == Screen::Submolts {
                app.select_right();
            }
        }
        KeyCode::Char(' ') => {
            if app.screen == Screen::Submolts && !app.submolts.is_empty() {
                app.show_submolt_detail = true;
            }
        }
        KeyCode::Tab => {
            // Toggle agent preview modal on agent-related screens
            match app.screen {
                Screen::Leaderboard => {
                    if !app.leaderboard.is_empty() {
                        let agent = &app.leaderboard[app.leaderboard_selected];
                        app.preview_agent_name = Some(agent.name.clone());
                        app.agent_profile = None;
                        app.show_agent_preview = true;
                        // Load agent profile for preview
                        app.is_loading = true;
                        app.is_preview_loading = true;
                        load_agent_preview(api_client.clone(), agent.name.clone(), tx.clone());
                    }
                }
                Screen::TopPairings => {
                    if !app.top_pairings.is_empty() {
                        let human = &app.top_pairings[app.top_pairings_selected];
                        app.preview_agent_name = Some(human.bot_name.clone());
                        app.agent_profile = None;
                        app.show_agent_preview = true;
                        // Load agent profile for preview
                        app.is_loading = true;
                        app.is_preview_loading = true;
                        load_agent_preview(api_client.clone(), human.bot_name.clone(), tx.clone());
                    }
                }
                Screen::Search => {
                    app.search_type = app.search_type.cycle_next();
                    if app.search_submitted.is_some() {
                        run_search(app, api_client.clone(), tx.clone());
                    }
                }
                Screen::RecentAgents => {
                    if !app.recent_agents.is_empty() {
                        let agent = &app.recent_agents[app.recent_selected];
                        app.preview_agent_name = Some(agent.name.clone());
                        app.agent_profile = None;
                        app.show_agent_preview = true;
                        // Load agent profile for preview
                        app.is_loading = true;
                        app.is_preview_loading = true;
                        load_agent_preview(api_client.clone(), agent.name.clone(), tx.clone());
                    }
                }
                _ => {}
            }
        }
        KeyCode::Enter => {
            if app.screen == Screen::Feed {
                if let Some(post) = app.selected_post() {
                    let post_id = post.id.clone();
                    app.open_selected_post();
                    app.is_loading = true;
                    load_post_with_comments(api_client, post_id, tx);
                }
            } else if app.screen == Screen::PostDetail {
                if let Some(comment_id) = app.get_selected_comment_id() {
                    app.toggle_comment_collapse(&comment_id);
                }
            } else if app.screen == Screen::Leaderboard {
                // Open full agent profile
                if !app.leaderboard.is_empty() {
                    let agent = &app.leaderboard[app.leaderboard_selected];
                    app.previous_screen = Some(Screen::Leaderboard);
                    app.screen = Screen::AgentProfile;
                    app.is_loading = true;
                    load_agent_profile(api_client, agent.name.clone(), tx);
                }
            } else if app.screen == Screen::TopPairings {
                // Open full agent profile
                if !app.top_pairings.is_empty() {
                    let human = &app.top_pairings[app.top_pairings_selected];
                    app.previous_screen = Some(Screen::TopPairings);
                    app.screen = Screen::AgentProfile;
                    app.is_loading = true;
                    load_agent_profile(api_client, human.bot_name.clone(), tx);
                }
            } else if app.screen == Screen::RecentAgents {
                // Open full agent profile
                if !app.recent_agents.is_empty() {
                    let agent = &app.recent_agents[app.recent_selected];
                    app.previous_screen = Some(Screen::RecentAgents);
                    app.screen = Screen::AgentProfile;
                    app.is_loading = true;
                    load_agent_profile(api_client, agent.name.clone(), tx);
                }
            } else if app.screen == Screen::AgentProfile {
                // Open selected post from agent's posts
                if !app.agent_posts.is_empty() {
                    let post = app.agent_posts[app.agent_posts_selected].clone();
                    let post_id = post.id.clone();
                    app.previous_screen = Some(Screen::AgentProfile);
                    app.current_post = Some(post);
                    app.comments.clear();
                    app.comment_scroll = 0;
                    app.selected_comment_index = 0;
                    app.screen = Screen::PostDetail;
                    app.is_loading = true;
                    load_post_with_comments(api_client, post_id, tx);
                }
            } else if app.screen == Screen::Search {
                open_search_hit(app, api_client, tx);
            } else if app.screen == Screen::Submolts {
                // Load posts from selected submolt
                if !app.submolts.is_empty() {
                    let submolt = app.submolts[app.submolts_selected].clone();
                    app.current_submolt = Some(submolt.clone());
                    app.screen = Screen::Feed;
                    app.current_page = 0;
                    app.selected_index = 0;
                    app.is_loading = true;
                    load_posts(
                        api_client,
                        app.sort_order,
                        app.time_filter_for_api(),
                        None,
                        Some(submolt.name),
                        tx,
                    );
                }
            }
        }
        KeyCode::Esc => {
            app.go_back();
        }
        KeyCode::Char('r') => {
            if !app.is_loading {
                match app.screen {
                    Screen::Feed => {
                        app.is_loading = true;
                        let cursor = app.page_cursor();
                        load_posts(
                            api_client,
                            app.sort_order,
                            app.time_filter_for_api(),
                            cursor,
                            app.current_submolt.as_ref().map(|s| s.name.clone()),
                            tx,
                        );
                    }
                    Screen::Leaderboard => {
                        app.is_loading = true;
                        load_leaderboard(api_client, tx);
                    }
                    Screen::TopPairings => {
                        app.is_loading = true;
                        load_top_pairings(api_client, tx);
                    }
                    Screen::RecentAgents => {
                        app.is_loading = true;
                        load_recent_agents(api_client, tx);
                    }
                    Screen::Submolts => {
                        app.is_loading = true;
                        load_submolts(api_client, tx);
                    }
                    Screen::Stats => {
                        app.is_loading = true;
                        load_stats(api_client, tx);
                    }
                    Screen::AgentProfile => {
                        if let Some(ref profile) = app.agent_profile {
                            app.is_loading = true;
                            load_agent_profile(api_client, profile.name.clone(), tx);
                        }
                    }
                    Screen::Search => {
                        if let Some(query) = app.search_submitted.clone() {
                            app.search_query = query;
                            run_search(app, api_client, tx);
                        }
                    }
                    _ => {}
                }
            }
        }
        // Sort order keys
        KeyCode::Char('n') => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(api::SortOrder::New);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        KeyCode::Char('t') => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(api::SortOrder::Top);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        KeyCode::Char('d') => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(api::SortOrder::Discussed);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        KeyCode::Char('R') => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(api::SortOrder::Random);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        KeyCode::Char('f') | KeyCode::Right => {
            if app.screen == Screen::Submolts {
                app.select_right();
            } else if app.screen == Screen::Settings {
                handle_settings_change(app, true);
            } else if app.screen == Screen::Feed
                && !app.is_loading
                && app.sort_order != api::SortOrder::New
            {
                app.cycle_time_filter();
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        KeyCode::Left => {
            if app.screen == Screen::Submolts {
                app.select_left();
            } else if app.screen == Screen::Settings {
                handle_settings_change(app, false);
            } else if app.screen == Screen::Feed
                && !app.is_loading
                && app.sort_order != api::SortOrder::New
            {
                app.cycle_time_filter_reverse();
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        // 's' for shuffle (switch to random sort with fresh seed)
        KeyCode::Char('s') => {
            if app.screen == Screen::Feed && !app.is_loading {
                app.set_sort_order(api::SortOrder::Random);
                app.current_page = 0;
                app.is_loading = true;
                load_posts(api_client, app.sort_order, app.time_filter_for_api(), None, app.current_submolt.as_ref().map(|s| s.name.clone()), tx);
            }
        }
        // Page navigation (capital N/P)
        KeyCode::Char('N') => {
            if app.screen == Screen::Search {
                load_more_search(app, api_client, tx);
            } else if app.screen == Screen::Feed && !app.is_loading && app.has_more_posts {
                app.next_page();
                app.is_loading = true;
                let cursor = app.page_cursor();
                load_posts(
                    api_client,
                    app.sort_order,
                    app.time_filter_for_api(),
                    cursor,
                    app.current_submolt.as_ref().map(|s| s.name.clone()),
                    tx,
                );
            }
        }
        KeyCode::Char('P') => {
            if app.screen == Screen::Feed && !app.is_loading && app.current_page > 0 {
                app.prev_page();
                app.is_loading = true;
                let cursor = app.page_cursor();
                load_posts(
                    api_client,
                    app.sort_order,
                    app.time_filter_for_api(),
                    cursor,
                    app.current_submolt.as_ref().map(|s| s.name.clone()),
                    tx,
                );
            }
        }
        KeyCode::Char('1') => {
            app.add_debug("-> Feed".to_string());
            app.screen = Screen::Feed;
        }
        KeyCode::Char('2') => {
            app.add_debug("-> Leaderboard".to_string());
            app.screen = Screen::Leaderboard;
            if app.leaderboard.is_empty() {
                app.is_loading = true;
                load_leaderboard(api_client, tx);
            }
        }
        KeyCode::Char('3') => {
            app.add_debug("-> TopPairings".to_string());
            app.screen = Screen::TopPairings;
            if app.top_pairings.is_empty() {
                app.is_loading = true;
                load_top_pairings(api_client, tx);
            }
        }
        KeyCode::Char('4') => {
            app.add_debug("-> RecentAgents".to_string());
            app.screen = Screen::RecentAgents;
            if app.recent_agents.is_empty() {
                app.is_loading = true;
                load_recent_agents(api_client, tx);
            }
        }
        KeyCode::Char('5') => {
            app.add_debug("-> Submolts".to_string());
            app.screen = Screen::Submolts;
            if app.submolts.is_empty() {
                app.is_loading = true;
                load_submolts(api_client, tx);
            }
        }
        KeyCode::Char('6') => {
            app.add_debug("-> Stats".to_string());
            app.screen = Screen::Stats;
            if app.stats.is_none() {
                app.is_loading = true;
                load_stats(api_client, tx);
            }
        }
        KeyCode::Char('7') => {
            app.add_debug("-> Settings".to_string());
            app.screen = Screen::Settings;
        }
        KeyCode::Char('8') => {
            app.toggle_about();
        }
        KeyCode::Char('o') => {
            if let Some(path) = app.web_path() {
                open_url(&format!("https://www.moltbook.com{}", path));
            }
        }
        // Refresh interval adjustment
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.refresh_interval_secs = (app.refresh_interval_secs + 5).min(60);
            app.add_debug(format!("Refresh interval: {}s", app.refresh_interval_secs));
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            app.refresh_interval_secs = app.refresh_interval_secs.saturating_sub(5);
            if app.refresh_interval_secs == 0 {
                app.add_debug("Refresh interval: Off".to_string());
            } else {
                app.add_debug(format!("Refresh interval: {}s", app.refresh_interval_secs));
            }
        }
        KeyCode::Char('a') => {
            if app.refresh_interval_secs == 0 {
                app.refresh_interval_secs = REFRESH_INTERVAL_SECS;
                app.add_debug(format!("Auto-refresh: On ({}s)", app.refresh_interval_secs));
            } else {
                app.refresh_interval_secs = 0;
                app.add_debug("Auto-refresh: Off".to_string());
            }
        }
        // Debug mode toggle (backtick key)
        KeyCode::Char('`') => {
            app.toggle_debug();
        }
        _ => {}
    }
}

fn load_posts(
    api_client: Arc<api::ApiClient>,
    sort: api::SortOrder,
    time_filter: Option<TimeFilter>,
    cursor: Option<String>,
    submolt: Option<String>,
    tx: mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        let submolt_str = submolt.as_deref();
        let _ = tx
            .send(AppEvent::Debug(format!(
                "GET /posts?sort={:?}&time={:?}&cursor={:?}&submolt={:?}",
                sort, time_filter, cursor, submolt_str
            )))
            .await;
        match api_client
            .get_posts(
                sort,
                time_filter,
                POSTS_LIMIT,
                cursor.as_deref(),
                submolt_str,
            )
            .await
        {
            Ok(response) => {
                // Debug first post's author
                if let Some(first) = response.posts.first() {
                    let author_info = first
                        .author
                        .as_ref()
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| "NONE".to_string());
                    let _ = tx
                        .send(AppEvent::Debug(format!(
                            "First post author: {}",
                            author_info
                        )))
                        .await;
                }
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} posts loaded",
                        response.posts.len()
                    )))
                    .await;
                let _ = tx
                    .send(AppEvent::PostsLoaded(
                        response.posts,
                        response.has_more,
                        response.next_cursor,
                    ))
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

/// Start a fresh search for whatever is in the query box.
fn run_search(app: &mut App, api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    if let Some(query) = app.begin_search() {
        app.is_loading = true;
        load_search(api_client, query, app.search_type, None, tx);
    }
}

/// Append the next page of the current search, if there is one and none is on its way.
fn load_more_search(app: &mut App, api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    if app.is_loading || !app.search_has_more {
        return;
    }
    if let (Some(query), Some(cursor)) = (app.search_submitted.clone(), app.search_cursor.clone()) {
        app.is_loading = true;
        app.is_background_loading = true; // no modal spinner over the list being read
        load_search(api_client, query, app.search_type, Some(cursor), tx);
    }
}

fn load_search(
    api_client: Arc<api::ApiClient>,
    query: String,
    kind: SearchType,
    cursor: Option<String>,
    tx: mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        let append = cursor.is_some();
        let _ = tx
            .send(AppEvent::Debug(format!(
                "GET /search?q={}&type={}{}",
                query,
                kind.as_str(),
                if append { "&cursor=…" } else { "" }
            )))
            .await;
        match api_client.search(&query, kind, cursor.as_deref()).await {
            Ok(response) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} search results",
                        response.results.len()
                    )))
                    .await;
                let _ = tx
                    .send(AppEvent::SearchLoaded {
                        query,
                        kind,
                        results: response.results,
                        has_more: response.has_more,
                        next_cursor: response.next_cursor,
                        append,
                    })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!("Search failed: {}", e)))
                    .await;
            }
        }
    });
}

/// Open whatever the selected search result is: a post (or the post a comment is
/// on) with its comments, an agent's profile, or a submolt's feed.
fn open_search_hit(app: &mut App, api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    let Some(hit) = app.selected_search_result().cloned() else {
        return;
    };
    match hit.kind.as_str() {
        "post" | "comment" => {
            app.open_search_post(&hit);
            let post_id = app
                .current_post
                .as_ref()
                .map(|p| p.id.clone())
                .unwrap_or_default();
            app.is_loading = true;
            load_post_with_comments(api_client.clone(), post_id.clone(), tx.clone());
            load_post(api_client, post_id, tx);
        }
        "agent" => {
            let name = hit
                .title
                .clone()
                .or_else(|| hit.author.as_ref().map(|a| a.name.clone()));
            if let Some(name) = name {
                app.previous_screen = Some(Screen::Search);
                app.screen = Screen::AgentProfile;
                app.is_loading = true;
                load_agent_profile(api_client, name, tx);
            }
        }
        "submolt" => {
            if let Some(submolt) = hit.submolt.clone() {
                app.current_submolt = Some(api::SubmoltFull {
                    id: submolt.id.unwrap_or(hit.id),
                    display_name: submolt.display_name.unwrap_or_else(|| submolt.name.clone()),
                    name: submolt.name.clone(),
                    description: hit.content.as_deref().map(ui::utils::strip_highlights),
                    subscriber_count: hit.upvotes,
                    created_at: hit.created_at.unwrap_or_default(),
                    last_activity_at: None,
                    featured_at: None,
                    created_by: None,
                });
                app.screen = Screen::Feed;
                app.current_page = 0;
                app.selected_index = 0;
                app.is_loading = true;
                load_posts(
                    api_client,
                    app.sort_order,
                    app.time_filter_for_api(),
                    None,
                    Some(submolt.name),
                    tx,
                );
            }
        }
        _ => {}
    }
}

/// Fetch one post in full. A search hit carries only a title and a snippet, so the
/// post detail opens with that and fills in when this arrives.
fn load_post(api_client: Arc<api::ApiClient>, post_id: String, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match api_client.get_post(&post_id).await {
            Ok(post) => {
                let _ = tx.send(AppEvent::PostLoaded(post)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Debug(format!("GET /posts/{}: {}", post_id, e)))
                    .await;
            }
        }
    });
}

fn load_post_with_comments(
    api_client: Arc<api::ApiClient>,
    post_id: String,
    tx: mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        let _ = tx
            .send(AppEvent::Debug(format!("GET /posts/{}/comments", post_id)))
            .await;
        match api_client.get_comments(&post_id, MAX_COMMENTS).await {
            Ok(comments) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} top-level comments loaded",
                        comments.len()
                    )))
                    .await;
                let _ = tx.send(AppEvent::CommentsLoaded(comments)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!("Failed to load comments: {}", e)))
                    .await;
            }
        }
    });
}

fn load_stats(api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx.send(AppEvent::Debug("GET /stats".to_string())).await;
        match api_client.get_stats().await {
            Ok(stats) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: stats loaded (agents={}, posts={})",
                        stats.agents, stats.posts
                    )))
                    .await;
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

fn load_leaderboard(api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx
            .send(AppEvent::Debug("GET /agents/leaderboard".to_string()))
            .await;
        match api_client.get_leaderboard().await {
            Ok(leaderboard) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} agents in leaderboard",
                        leaderboard.len()
                    )))
                    .await;
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

fn load_top_pairings(api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx
            .send(AppEvent::Debug("GET /api/homepage".to_string()))
            .await;
        match api_client.get_top_humans().await {
            Ok(top_humans) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} top humans loaded",
                        top_humans.len()
                    )))
                    .await;
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

fn load_recent_agents(api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx
            .send(AppEvent::Debug("GET /agents/recent".to_string()))
            .await;
        match api_client.get_recent_agents().await {
            Ok(agents) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} recent agents loaded",
                        agents.len()
                    )))
                    .await;
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

fn load_submolts(api_client: Arc<api::ApiClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx.send(AppEvent::Debug("GET /submolts".to_string())).await;
        match api_client.get_submolts().await {
            Ok(submolts) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: {} submolts loaded",
                        submolts.len()
                    )))
                    .await;
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

fn load_agent_profile(api_client: Arc<api::ApiClient>, name: String, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx
            .send(AppEvent::Debug(format!(
                "GET /agents/profile?name={}",
                name
            )))
            .await;
        match api_client.get_agent_profile(&name).await {
            Ok(response) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: agent profile loaded with {} posts",
                        response.recent_posts.len()
                    )))
                    .await;
                let _ = tx.send(AppEvent::AgentProfileLoaded(response)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!(
                        "Failed to load agent profile: {}",
                        e
                    )))
                    .await;
            }
        }
    });
}

fn load_agent_preview(api_client: Arc<api::ApiClient>, name: String, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = tx
            .send(AppEvent::Debug(format!(
                "GET /agents/profile?name={} (preview)",
                name
            )))
            .await;
        match api_client.get_agent_profile(&name).await {
            Ok(response) => {
                let _ = tx
                    .send(AppEvent::Debug(format!(
                        "OK: agent preview loaded with {} posts",
                        response.recent_posts.len()
                    )))
                    .await;
                let _ = tx.send(AppEvent::AgentPreviewLoaded(response)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppEvent::Error(format!(
                        "Failed to load agent profile: {}",
                        e
                    )))
                    .await;
            }
        }
    });
}

fn update_agent_preview_for_current_selection(
    app: &mut App,
    api_client: Arc<api::ApiClient>,
    tx: mpsc::Sender<AppEvent>,
) {
    match app.screen {
        Screen::Leaderboard => {
            if !app.leaderboard.is_empty() {
                let agent = &app.leaderboard[app.leaderboard_selected];
                app.preview_agent_name = Some(agent.name.clone());
                app.agent_profile = None; // Clear old profile
                app.is_preview_loading = true;
                load_agent_preview(api_client, agent.name.clone(), tx);
            }
        }
        Screen::TopPairings => {
            if !app.top_pairings.is_empty() {
                let human = &app.top_pairings[app.top_pairings_selected];
                app.preview_agent_name = Some(human.bot_name.clone());
                app.agent_profile = None;
                app.is_preview_loading = true;
                load_agent_preview(api_client, human.bot_name.clone(), tx);
            }
        }
        Screen::RecentAgents => {
            if !app.recent_agents.is_empty() {
                let agent = &app.recent_agents[app.recent_selected];
                app.preview_agent_name = Some(agent.name.clone());
                app.agent_profile = None;
                app.is_preview_loading = true;
                load_agent_preview(api_client, agent.name.clone(), tx);
            }
        }
        _ => {}
    }
}

fn handle_settings_change(app: &mut App, forward: bool) {
    match app.settings_selected {
        0 => {
            // Row Display setting
            app.row_display = if forward {
                app.row_display.cycle_next()
            } else {
                app.row_display.cycle_prev()
            };
            app.add_debug(format!("Row display: {}", app.row_display.as_str()));
            // Save settings
            if let Err(e) =
                config::Config::save_settings(app.row_display, app.refresh_interval_secs)
            {
                app.add_debug(format!("Failed to save settings: {}", e));
            }
        }
        1 => {
            // Refresh Interval setting
            let intervals = [0u64, 10, 30, 60, 120];
            let current_idx = intervals
                .iter()
                .position(|&i| i == app.refresh_interval_secs)
                .unwrap_or(1); // Default to 10s if not found

            let new_idx = if forward {
                (current_idx + 1) % intervals.len()
            } else {
                (current_idx + intervals.len() - 1) % intervals.len()
            };

            app.refresh_interval_secs = intervals[new_idx];
            if app.refresh_interval_secs == 0 {
                app.add_debug("Refresh interval: Off".to_string());
            } else {
                app.add_debug(format!("Refresh interval: {}s", app.refresh_interval_secs));
            }
            // Save settings
            if let Err(e) =
                config::Config::save_settings(app.row_display, app.refresh_interval_secs)
            {
                app.add_debug(format!("Failed to save settings: {}", e));
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A client pointed at a port nothing listens on: requests the keys trigger fail
    /// quietly in the background, and the app's own state is what these check.
    fn harness() -> (
        App,
        Arc<api::ApiClient>,
        mpsc::Sender<AppEvent>,
        mpsc::Receiver<AppEvent>,
    ) {
        let (tx, rx) = mpsc::channel(100);
        let client = Arc::new(api::ApiClient::new("http://127.0.0.1:9".into(), None));
        (App::new(), client, tx, rx)
    }

    fn press(
        app: &mut App,
        client: &Arc<api::ApiClient>,
        tx: &mpsc::Sender<AppEvent>,
        keys: &[KeyCode],
    ) {
        for key in keys {
            handle_input(app, *key, client.clone(), tx.clone());
        }
    }

    fn typed(text: &str) -> Vec<KeyCode> {
        text.chars().map(KeyCode::Char).collect()
    }

    #[test]
    fn slash_opens_search_ready_for_typing_from_any_screen() {
        let (mut app, c, tx, _rx) = harness();
        app.screen = Screen::Stats;
        press(&mut app, &c, &tx, &[KeyCode::Char('/')]);
        assert_eq!(app.screen, Screen::Search);
        assert!(app.search_editing);
    }

    #[test]
    fn typing_a_query_never_fires_the_shortcuts_its_letters_are_bound_to() {
        let (mut app, c, tx, _rx) = harness();
        let sort = app.sort_order;
        press(&mut app, &c, &tx, &[KeyCode::Char('/')]);
        press(&mut app, &c, &tx, &typed("quit 1 ? jk nt `8"));
        assert_eq!(app.search_query, "quit 1 ? jk nt `8");
        assert_eq!(app.screen, Screen::Search, "1 did not switch screens");
        assert!(!app.should_quit, "q did not quit");
        assert!(!app.show_help && !app.show_about && !app.debug_mode);
        assert_eq!(app.sort_order, sort, "n and t did not re-sort the feed");

        press(&mut app, &c, &tx, &[KeyCode::Backspace, KeyCode::Backspace]);
        assert_eq!(app.search_query, "quit 1 ? jk nt ");
    }

    #[test]
    fn tab_while_typing_changes_the_result_type() {
        let (mut app, c, tx, _rx) = harness();
        press(
            &mut app,
            &c,
            &tx,
            &[KeyCode::Char('/'), KeyCode::Tab, KeyCode::Tab],
        );
        assert_eq!(app.search_type, SearchType::Comments);
        assert!(app.search_editing, "still typing");
    }

    #[test]
    fn esc_stops_typing_and_a_second_esc_leaves_search() {
        let (mut app, c, tx, _rx) = harness();
        app.screen = Screen::Leaderboard;
        press(&mut app, &c, &tx, &[KeyCode::Char('/')]);
        press(&mut app, &c, &tx, &typed("abc"));
        press(&mut app, &c, &tx, &[KeyCode::Esc]);
        assert_eq!(app.screen, Screen::Search);
        assert!(!app.search_editing);
        press(&mut app, &c, &tx, &[KeyCode::Esc]);
        assert_eq!(app.screen, Screen::Leaderboard);
    }

    #[test]
    fn esc_on_an_untouched_search_box_goes_straight_back() {
        let (mut app, c, tx, _rx) = harness();
        press(&mut app, &c, &tx, &[KeyCode::Char('/'), KeyCode::Esc]);
        assert_eq!(app.screen, Screen::Feed);
    }

    #[test]
    fn enter_on_an_empty_query_sends_nothing() {
        let (mut app, c, tx, mut rx) = harness();
        press(
            &mut app,
            &c,
            &tx,
            &[KeyCode::Char('/'), KeyCode::Char(' '), KeyCode::Enter],
        );
        assert!(app.search_editing && !app.is_loading);
        assert!(rx.try_recv().is_err(), "no request was started");
    }

    #[tokio::test]
    async fn enter_sends_the_query_with_its_type() {
        let (mut app, c, tx, mut rx) = harness();
        press(&mut app, &c, &tx, &[KeyCode::Char('/'), KeyCode::Tab]);
        press(&mut app, &c, &tx, &typed("agent memory"));
        press(&mut app, &c, &tx, &[KeyCode::Enter]);
        assert!(app.is_loading && !app.search_editing);
        assert_eq!(app.search_submitted.as_deref(), Some("agent memory"));
        match rx.recv().await {
            Some(AppEvent::Debug(line)) => {
                assert_eq!(line, "GET /search?q=agent memory&type=posts")
            }
            other => panic!("expected the request to be logged, got {other:?}"),
        }
    }

    #[test]
    fn with_results_showing_letters_are_shortcuts_again() {
        let (mut app, c, tx, _rx) = harness();
        app.screen = Screen::Search;
        app.search_results =
            serde_json::from_str::<serde_json::Value>(include_str!("api/fixtures/search.json"))
                .map(|v| serde_json::from_value(v["results"].clone()).unwrap())
                .unwrap();
        press(&mut app, &c, &tx, &[KeyCode::Char('j'), KeyCode::Char('j')]);
        assert_eq!(app.search_selected, 2);
        press(&mut app, &c, &tx, &[KeyCode::Char('/')]);
        assert!(app.search_editing, "/ goes back to editing the query");
    }
}
