use crate::api::{
    AgentProfile, Comment, LeaderboardAgent, Post, RecentAgent, SortOrder, Stats, SubmoltFull,
    TimeFilter, TopHuman,
};
use crate::config::RowDisplay;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Setup,
    Feed,
    PostDetail,
    Stats,
    Leaderboard,
    TopPairings,
    RecentAgents,
    Submolts,
    Settings,
    AgentProfile,
}

pub struct App {
    pub screen: Screen,
    pub posts: Vec<Post>,
    pub selected_index: usize,
    pub sort_order: SortOrder,
    pub time_filter: TimeFilter,
    pub current_post: Option<Post>,
    pub comments: Vec<Comment>,
    pub comment_scroll: usize,
    pub selected_comment_index: usize,
    pub collapsed_comments: HashSet<String>,
    pub seen_post_ids: HashSet<String>,
    pub new_post_ids: HashSet<String>,
    pub last_refresh: Option<std::time::Instant>,
    pub is_loading: bool,
    pub is_background_loading: bool,
    pub error_message: Option<String>,
    pub show_technical_error: bool,
    pub should_quit: bool,
    pub show_help: bool,
    pub current_page: usize,
    pub has_more_posts: bool,
    pub spinner_frame: usize,
    pub refresh_interval_secs: u64,
    // New API data
    pub stats: Option<Stats>,
    pub leaderboard: Vec<LeaderboardAgent>,
    pub top_pairings: Vec<TopHuman>,
    pub recent_agents: Vec<RecentAgent>,
    pub submolts: Vec<SubmoltFull>,
    pub leaderboard_selected: usize,
    pub top_pairings_selected: usize,
    pub recent_selected: usize,
    pub submolts_selected: usize,
    pub submolts_scroll_row: usize,
    // Setup screen
    pub api_key_input: String,
    pub setup_error: Option<String>,
    // Debug mode
    pub debug_mode: bool,
    pub debug_log: Vec<String>,
    // Navigation flag
    pub select_bottom_on_load: bool,
    // Settings
    pub settings_selected: usize,
    pub row_display: RowDisplay,
    // Submolt detail modal
    pub show_submolt_detail: bool,
    // Currently viewing submolt (None = all posts)
    pub current_submolt: Option<SubmoltFull>,
    // Agent profile
    pub agent_profile: Option<AgentProfile>,
    pub agent_posts: Vec<Post>,
    pub agent_posts_selected: usize,
    pub show_agent_preview: bool,
    pub preview_agent_name: Option<String>,
    pub previous_screen: Option<Screen>,
    pub is_preview_loading: bool,
    pub show_about: bool,
    pub last_frame_area: Option<(u16, u16)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Feed,
            posts: Vec::new(),
            selected_index: 0,
            sort_order: SortOrder::New,
            time_filter: TimeFilter::Day,
            current_post: None,
            comments: Vec::new(),
            comment_scroll: 0,
            selected_comment_index: 0,
            collapsed_comments: HashSet::new(),
            seen_post_ids: HashSet::new(),
            new_post_ids: HashSet::new(),
            last_refresh: None,
            is_loading: false,
            is_background_loading: false,
            error_message: None,
            show_technical_error: false,
            should_quit: false,
            show_help: false,
            current_page: 0,
            has_more_posts: false,
            spinner_frame: 0,
            refresh_interval_secs: 0,
            stats: None,
            leaderboard: Vec::new(),
            top_pairings: Vec::new(),
            recent_agents: Vec::new(),
            submolts: Vec::new(),
            leaderboard_selected: 0,
            top_pairings_selected: 0,
            recent_selected: 0,
            submolts_selected: 0,
            submolts_scroll_row: 0,
            api_key_input: String::new(),
            setup_error: None,
            debug_mode: false,
            debug_log: Vec::new(),
            select_bottom_on_load: false,
            settings_selected: 0,
            row_display: RowDisplay::default(),
            show_submolt_detail: false,
            current_submolt: None,
            agent_profile: None,
            agent_posts: Vec::new(),
            agent_posts_selected: 0,
            show_agent_preview: false,
            preview_agent_name: None,
            previous_screen: None,
            is_preview_loading: false,
            show_about: false,
            last_frame_area: None,
        }
    }

    pub fn add_debug(&mut self, msg: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        self.debug_log.push(format!("[{}] {}", timestamp, msg));
        // Keep only last 100 entries
        if self.debug_log.len() > 100 {
            self.debug_log.remove(0);
        }
    }

    pub fn toggle_debug(&mut self) {
        self.debug_mode = !self.debug_mode;
    }

    pub fn seconds_until_refresh(&self) -> u64 {
        match self.last_refresh {
            Some(instant) => {
                let elapsed = instant.elapsed().as_secs();
                self.refresh_interval_secs.saturating_sub(elapsed)
            }
            None => self.refresh_interval_secs,
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_about(&mut self) {
        self.show_about = !self.show_about;
    }

    pub fn next_page(&mut self) {
        if self.has_more_posts {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.select_bottom_on_load = true;
        }
    }

    pub fn advance_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 10;
    }

    pub fn select_next(&mut self) {
        match self.screen {
            Screen::Feed => {
                if !self.posts.is_empty() && self.selected_index < self.posts.len() - 1 {
                    self.selected_index += 1;
                }
            }
            Screen::PostDetail => {
                let visible_count = self.get_visible_comment_ids().len();
                if visible_count > 0 && self.selected_comment_index < visible_count - 1 {
                    self.selected_comment_index += 1;
                }
            }
            Screen::Setup | Screen::Stats => {}
            Screen::Leaderboard => {
                if !self.leaderboard.is_empty()
                    && self.leaderboard_selected < self.leaderboard.len() - 1
                {
                    self.leaderboard_selected += 1;
                }
            }
            Screen::TopPairings => {
                if !self.top_pairings.is_empty()
                    && self.top_pairings_selected < self.top_pairings.len() - 1
                {
                    self.top_pairings_selected += 1;
                }
            }
            Screen::RecentAgents => {
                if !self.recent_agents.is_empty()
                    && self.recent_selected < self.recent_agents.len() - 1
                {
                    self.recent_selected += 1;
                }
            }
            Screen::Submolts => {
                // Move down one row (4 items) in grid
                if !self.submolts.is_empty() && self.submolts_selected + 4 < self.submolts.len() {
                    self.submolts_selected += 4;
                }
            }
            Screen::Settings => {
                if self.settings_selected < 1 {
                    self.settings_selected += 1;
                }
            }
            Screen::AgentProfile => {
                if !self.agent_posts.is_empty()
                    && self.agent_posts_selected < self.agent_posts.len() - 1
                {
                    self.agent_posts_selected += 1;
                }
            }
        }
    }

    pub fn select_previous(&mut self) {
        match self.screen {
            Screen::Feed => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Screen::PostDetail => {
                if self.selected_comment_index > 0 {
                    self.selected_comment_index -= 1;
                }
            }
            Screen::Setup | Screen::Stats => {}
            Screen::Leaderboard => {
                if self.leaderboard_selected > 0 {
                    self.leaderboard_selected -= 1;
                }
            }
            Screen::TopPairings => {
                if self.top_pairings_selected > 0 {
                    self.top_pairings_selected -= 1;
                }
            }
            Screen::RecentAgents => {
                if self.recent_selected > 0 {
                    self.recent_selected -= 1;
                }
            }
            Screen::Submolts => {
                // Move up one row (4 items) in grid
                if self.submolts_selected >= 4 {
                    self.submolts_selected -= 4;
                }
            }
            Screen::Settings => {
                if self.settings_selected > 0 {
                    self.settings_selected -= 1;
                }
            }
            Screen::AgentProfile => {
                if self.agent_posts_selected > 0 {
                    self.agent_posts_selected -= 1;
                }
            }
        }
    }

    pub fn select_left(&mut self) {
        if self.screen == Screen::Submolts {
            // Don't move left if at left edge of grid (column 0)
            if self.submolts_selected % 4 > 0 {
                self.submolts_selected -= 1;
            }
        }
    }

    pub fn select_right(&mut self) {
        if self.screen == Screen::Submolts {
            // Don't move right if at right edge of grid (column 3) or past end
            if self.submolts_selected % 4 < 3 && self.submolts_selected + 1 < self.submolts.len() {
                self.submolts_selected += 1;
            }
        }
    }

    pub fn sort_display(&self) -> String {
        match self.sort_order {
            SortOrder::New => "New".to_string(),
            _ => format!("{} - {}", self.sort_order, self.time_filter),
        }
    }

    pub fn time_filter_for_api(&self) -> Option<TimeFilter> {
        if self.sort_order == SortOrder::New {
            None
        } else {
            Some(self.time_filter)
        }
    }

    pub fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_order = order;
        // Reset time filter to Day when switching to Top/Discussed if currently on Hour
        if order != SortOrder::New && self.time_filter == TimeFilter::Hour {
            self.time_filter = TimeFilter::Day;
        }
    }

    pub fn cycle_time_filter(&mut self) {
        self.time_filter = match self.time_filter {
            TimeFilter::Hour => TimeFilter::Day,
            TimeFilter::Day => TimeFilter::Week,
            TimeFilter::Week => TimeFilter::Month,
            TimeFilter::Month => TimeFilter::Year,
            TimeFilter::Year => TimeFilter::All,
            TimeFilter::All => TimeFilter::Hour,
        };
    }

    pub fn cycle_time_filter_reverse(&mut self) {
        self.time_filter = match self.time_filter {
            TimeFilter::Hour => TimeFilter::All,
            TimeFilter::Day => TimeFilter::Hour,
            TimeFilter::Week => TimeFilter::Day,
            TimeFilter::Month => TimeFilter::Week,
            TimeFilter::Year => TimeFilter::Month,
            TimeFilter::All => TimeFilter::Year,
        };
    }

    pub fn open_selected_post(&mut self) {
        if let Some(post) = self.posts.get(self.selected_index) {
            // Mark as seen when opening
            self.new_post_ids.remove(&post.id);
            self.seen_post_ids.insert(post.id.clone());
            self.current_post = Some(post.clone());
            self.comments.clear();
            self.comment_scroll = 0;
            self.screen = Screen::PostDetail;
        }
    }

    pub fn go_back(&mut self) {
        match self.screen {
            Screen::PostDetail => {
                // Check if we came from AgentProfile
                if self.previous_screen == Some(Screen::AgentProfile) {
                    self.screen = Screen::AgentProfile;
                    self.current_post = None;
                    self.comments.clear();
                    self.comment_scroll = 0;
                    self.selected_comment_index = 0;
                    self.collapsed_comments.clear();
                    self.previous_screen = None;
                } else {
                    self.screen = Screen::Feed;
                    self.current_post = None;
                    self.comments.clear();
                    self.comment_scroll = 0;
                    self.selected_comment_index = 0;
                    self.collapsed_comments.clear();
                }
            }
            Screen::Setup => {
                self.should_quit = true;
            }
            Screen::Feed => {
                // If viewing a submolt, go back to all posts first
                if self.current_submolt.is_some() {
                    self.current_submolt = None;
                } else {
                    self.should_quit = true;
                }
            }
            Screen::AgentProfile => {
                // Go back to the screen we came from
                if let Some(prev) = self.previous_screen.take() {
                    self.screen = prev;
                } else {
                    self.screen = Screen::Leaderboard;
                }
                self.agent_profile = None;
                self.agent_posts.clear();
                self.agent_posts_selected = 0;
            }
            Screen::Stats
            | Screen::Leaderboard
            | Screen::TopPairings
            | Screen::RecentAgents
            | Screen::Submolts
            | Screen::Settings => {
                self.screen = Screen::Feed;
            }
        }
    }

    pub fn toggle_comment_collapse(&mut self, comment_id: &str) {
        if self.collapsed_comments.contains(comment_id) {
            self.collapsed_comments.remove(comment_id);
        } else {
            self.collapsed_comments.insert(comment_id.to_string());
        }
    }

    pub fn is_comment_collapsed(&self, comment_id: &str) -> bool {
        self.collapsed_comments.contains(comment_id)
    }

    pub fn get_visible_comment_ids(&self) -> Vec<String> {
        fn collect_visible(
            comments: &[Comment],
            collapsed: &HashSet<String>,
            result: &mut Vec<String>,
        ) {
            for comment in comments {
                result.push(comment.id.clone());
                if !collapsed.contains(&comment.id) {
                    collect_visible(&comment.replies, collapsed, result);
                }
            }
        }
        let mut ids = Vec::new();
        collect_visible(&self.comments, &self.collapsed_comments, &mut ids);
        ids
    }

    pub fn get_selected_comment_id(&self) -> Option<String> {
        let visible_ids = self.get_visible_comment_ids();
        visible_ids.get(self.selected_comment_index).cloned()
    }

    pub fn update_posts(&mut self, posts: Vec<Post>) {
        // Find new posts
        let current_ids: HashSet<&String> = self.posts.iter().map(|p| &p.id).collect();

        for post in &posts {
            if !current_ids.contains(&post.id) && !self.seen_post_ids.contains(&post.id) {
                self.new_post_ids.insert(post.id.clone());
            }
        }

        self.posts = posts;
        self.last_refresh = Some(std::time::Instant::now());

        // Select bottom if navigating to previous page
        if self.select_bottom_on_load && !self.posts.is_empty() {
            self.selected_index = self.posts.len() - 1;
            self.select_bottom_on_load = false;
        } else if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
            // Adjust selection if needed
            self.selected_index = self.posts.len() - 1;
        }
    }

    pub fn selected_post(&self) -> Option<&Post> {
        self.posts.get(self.selected_index)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
