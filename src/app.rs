use crate::api::{
    Agent, AgentProfile, Comment, LeaderboardAgent, Post, RecentAgent, SearchResult, SearchType,
    SortOrder, Stats, SubmoltFull, TimeFilter, TopHuman,
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
    Search,
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
    // The cursor that fetches each page, by page number. The API pages by cursor,
    // so a page can only be reached by having loaded the one before it.
    pub page_cursors: Vec<Option<String>>,
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
    // Search. `search_editing` routes every key into the query until Enter or Esc;
    // `search_submitted` is the query the results belong to, which can differ from
    // `search_query` while it is being edited, and is what stale responses are checked against.
    pub search_query: String,
    pub search_editing: bool,
    pub search_type: SearchType,
    pub search_submitted: Option<String>,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    pub search_cursor: Option<String>,
    pub search_has_more: bool,
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
            page_cursors: vec![None],
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
            search_query: String::new(),
            search_editing: false,
            search_type: SearchType::All,
            search_submitted: None,
            search_results: Vec::new(),
            search_selected: 0,
            search_cursor: None,
            search_has_more: false,
        }
    }

    /// Show the search screen with the query open for typing. Earlier results stay.
    pub fn open_search(&mut self) {
        if self.screen != Screen::Search {
            self.previous_screen = Some(self.screen.clone());
        }
        self.screen = Screen::Search;
        self.search_editing = true;
    }

    /// Start a fresh search for the current query and type. Returns the query to send,
    /// or `None` when there is nothing to search for.
    pub fn begin_search(&mut self) -> Option<String> {
        let query = self.search_query.trim().to_string();
        if query.is_empty() {
            return None;
        }
        self.search_submitted = Some(query.clone());
        self.search_results.clear();
        self.search_selected = 0;
        self.search_cursor = None;
        self.search_has_more = false;
        self.search_editing = false;
        Some(query)
    }

    /// Take in a page of search results. A reply to a search that has since been
    /// replaced (different query or type) is ignored, and returns false.
    pub fn apply_search_results(
        &mut self,
        query: &str,
        kind: SearchType,
        results: Vec<SearchResult>,
        has_more: bool,
        next_cursor: Option<String>,
        append: bool,
    ) -> bool {
        if self.search_submitted.as_deref() != Some(query) || self.search_type != kind {
            return false;
        }
        if append {
            // The cursor can hand back a result tied with the last one.
            let fresh: Vec<_> = results
                .into_iter()
                .filter(|r| self.search_results.iter().all(|s| s.id != r.id))
                .collect();
            self.search_results.extend(fresh);
        } else {
            self.search_results = results;
            self.search_selected = 0;
        }
        self.search_has_more = has_more && next_cursor.is_some();
        self.search_cursor = next_cursor;
        true
    }

    /// The moltbook.com page for whatever is selected, as a path: `/post/{id}`,
    /// `/u/{name}` or `/m/{name}`. The site used to use `/posts/`, `/agent/` and `/s/`.
    pub fn web_path(&self) -> Option<String> {
        match self.screen {
            Screen::Feed => self
                .posts
                .get(self.selected_index)
                .map(|p| format!("/post/{}", p.id)),
            Screen::PostDetail => self
                .current_post
                .as_ref()
                .map(|p| format!("/post/{}", p.id)),
            Screen::AgentProfile => self
                .agent_profile
                .as_ref()
                .map(|a| format!("/u/{}", a.name)),
            Screen::Submolts => self
                .submolts
                .get(self.submolts_selected)
                .map(|s| format!("/m/{}", s.name)),
            Screen::Search => self.selected_search_result().and_then(|h| h.url.clone()),
            Screen::Leaderboard => self
                .leaderboard
                .get(self.leaderboard_selected)
                .map(|a| format!("/u/{}", a.name)),
            Screen::RecentAgents => self
                .recent_agents
                .get(self.recent_selected)
                .map(|a| format!("/u/{}", a.name)),
            _ => None,
        }
    }

    pub fn selected_search_result(&self) -> Option<&SearchResult> {
        self.search_results.get(self.search_selected)
    }

    /// Open a post from a search hit straight away with what the hit carries; the
    /// full post replaces it when `PostLoaded` arrives.
    pub fn open_search_post(&mut self, hit: &SearchResult) {
        let post_id = if hit.post_id.is_empty() {
            hit.id.clone()
        } else {
            hit.post_id.clone()
        };
        let title = match hit.kind.as_str() {
            "comment" => hit.post.as_ref().and_then(|p| p.title.clone()),
            _ => hit.title.clone(),
        };
        let is_post = hit.kind == "post";
        self.current_post = Some(Post {
            id: post_id,
            title: title.unwrap_or_default(),
            content: None,
            url: None,
            upvotes: if is_post { hit.upvotes } else { 0 },
            downvotes: if is_post { hit.downvotes } else { 0 },
            comment_count: 0,
            created_at: if is_post {
                hit.created_at.clone().unwrap_or_default()
            } else {
                String::new()
            },
            author: if is_post {
                hit.author.as_ref().map(|a| Agent {
                    id: String::new(),
                    name: a.name.clone(),
                })
            } else {
                None
            },
            submolt: if is_post { hit.submolt.clone() } else { None },
        });
        self.comments.clear();
        self.comment_scroll = 0;
        self.selected_comment_index = 0;
        self.collapsed_comments.clear();
        self.previous_screen = Some(Screen::Search);
        self.screen = Screen::PostDetail;
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

    /// The cursor for the page currently being viewed (`None` for the first page).
    pub fn page_cursor(&self) -> Option<String> {
        self.page_cursors.get(self.current_page).cloned().flatten()
    }

    /// Remember where the page after the current one starts.
    pub fn record_next_cursor(&mut self, cursor: Option<String>) {
        self.page_cursors.truncate(self.current_page + 1);
        self.page_cursors.resize(self.current_page + 1, None);
        self.page_cursors.push(cursor);
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
            Screen::Search => {
                if self.search_selected + 1 < self.search_results.len() {
                    self.search_selected += 1;
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
            Screen::Search => {
                if self.search_selected > 0 {
                    self.search_selected -= 1;
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
                // Check if we came from AgentProfile or Search
                if self.previous_screen == Some(Screen::Search) {
                    self.screen = Screen::Search;
                    self.current_post = None;
                    self.comments.clear();
                    self.comment_scroll = 0;
                    self.selected_comment_index = 0;
                    self.collapsed_comments.clear();
                    self.previous_screen = None;
                } else if self.previous_screen == Some(Screen::AgentProfile) {
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
            Screen::Search => {
                self.search_editing = false;
                self.screen = match self.previous_screen.take() {
                    Some(Screen::PostDetail) | Some(Screen::AgentProfile) | None => Screen::Feed,
                    Some(prev) => prev,
                };
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn fixture(name: &str) -> Value {
        let text = match name {
            "posts" => include_str!("api/fixtures/posts.json"),
            "search" => include_str!("api/fixtures/search.json"),
            "submolts" => include_str!("api/fixtures/submolts.json"),
            "leaderboard" => include_str!("api/fixtures/leaderboard.json"),
            "recent_agents" => include_str!("api/fixtures/recent_agents.json"),
            "agent_profile" => include_str!("api/fixtures/agent_profile.json"),
            _ => unreachable!(),
        };
        serde_json::from_str(text).unwrap()
    }

    fn list<T: serde::de::DeserializeOwned>(name: &str, key: &str) -> Vec<T> {
        serde_json::from_value(fixture(name)[key].clone()).unwrap()
    }

    fn hits() -> Vec<SearchResult> {
        list("search", "results")
    }

    fn hit(kind: &str) -> SearchResult {
        hits().into_iter().find(|h| h.kind == kind).unwrap()
    }

    fn searching(query: &str) -> App {
        let mut app = App::new();
        app.search_query = query.to_string();
        app.begin_search();
        app
    }

    // --- Feed paging by cursor ---

    #[test]
    fn each_page_is_fetched_with_the_cursor_the_page_before_it_returned() {
        let mut app = App::new();
        assert_eq!(app.page_cursor(), None, "the first page needs no cursor");

        app.record_next_cursor(Some("c1".into()));
        app.has_more_posts = true;
        app.next_page();
        assert_eq!(app.page_cursor().as_deref(), Some("c1"));

        app.record_next_cursor(Some("c2".into()));
        app.next_page();
        assert_eq!(app.page_cursor().as_deref(), Some("c2"));

        app.prev_page();
        assert_eq!(app.page_cursor().as_deref(), Some("c1"));
        app.prev_page();
        assert_eq!(app.page_cursor(), None);
    }

    #[test]
    fn reloading_a_page_forgets_the_cursors_after_it() {
        let mut app = App::new();
        app.record_next_cursor(Some("c1".into()));
        app.has_more_posts = true;
        app.next_page();
        app.record_next_cursor(Some("c2".into()));
        app.prev_page();

        // Page one reloaded (a refresh, say) and the feed moved on underneath it.
        app.record_next_cursor(Some("c1-new".into()));
        assert_eq!(app.page_cursors, vec![None, Some("c1-new".into())]);
    }

    // --- Links to the site ---

    #[test]
    fn browser_links_use_the_sites_current_paths() {
        let mut app = App::new();
        app.posts = list("posts", "posts");
        let post = app.posts[0].id.clone();
        assert_eq!(app.web_path(), Some(format!("/post/{post}")));

        app.screen = Screen::PostDetail;
        app.current_post = Some(app.posts[0].clone());
        assert_eq!(app.web_path(), Some(format!("/post/{post}")));

        app.screen = Screen::Leaderboard;
        app.leaderboard = list("leaderboard", "agents");
        assert_eq!(
            app.web_path(),
            Some(format!("/u/{}", app.leaderboard[0].name))
        );

        app.screen = Screen::RecentAgents;
        app.recent_agents = list("recent_agents", "agents");
        assert_eq!(
            app.web_path(),
            Some(format!("/u/{}", app.recent_agents[0].name))
        );

        app.screen = Screen::AgentProfile;
        app.agent_profile =
            Some(serde_json::from_value(fixture("agent_profile")["agent"].clone()).unwrap());
        let name = app.agent_profile.as_ref().unwrap().name.clone();
        assert_eq!(app.web_path(), Some(format!("/u/{name}")));

        app.screen = Screen::Submolts;
        app.submolts = list("submolts", "submolts");
        assert_eq!(app.web_path(), Some(format!("/m/{}", app.submolts[0].name)));

        app.screen = Screen::Search;
        app.search_results = hits();
        assert_eq!(app.web_path(), app.search_results[0].url.clone());
    }

    // --- Search ---

    #[test]
    fn an_empty_or_blank_query_is_not_searched() {
        let mut app = App::new();
        app.search_query = "   ".into();
        assert_eq!(app.begin_search(), None);
        assert_eq!(app.search_submitted, None);
    }

    #[test]
    fn a_search_trims_its_query_and_stops_typing() {
        let mut app = App::new();
        app.open_search();
        app.search_query = "  agent memory ".into();
        assert_eq!(app.begin_search().as_deref(), Some("agent memory"));
        assert!(!app.search_editing);
    }

    #[test]
    fn results_for_an_older_query_or_type_are_ignored() {
        let mut app = searching("new query");
        assert!(!app.apply_search_results(
            "old query",
            SearchType::All,
            hits(),
            true,
            Some("c".into()),
            false
        ));
        assert!(app.search_results.is_empty());

        app.search_type = SearchType::Agents;
        assert!(!app.apply_search_results(
            "new query",
            SearchType::All,
            hits(),
            true,
            Some("c".into()),
            false
        ));
        assert!(app.search_results.is_empty());
    }

    #[test]
    fn a_further_page_appends_without_repeating_a_result() {
        let mut app = searching("q");
        let first = hits();
        assert!(app.apply_search_results(
            "q",
            SearchType::All,
            first.clone(),
            true,
            Some("c".into()),
            false
        ));
        app.search_selected = 2;

        // The server's cursor can return the last result again at the top of the next page.
        let next = vec![first.last().unwrap().clone(), hit("post")];
        let mut fresh = hit("post");
        fresh.id = "brand-new".into();
        let next = [next, vec![fresh]].concat();
        app.apply_search_results("q", SearchType::All, next, false, None, true);

        let ids: Vec<&str> = app.search_results.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids.len(), first.len() + 1, "{ids:?}");
        assert_eq!(*ids.last().unwrap(), "brand-new");
        assert_eq!(app.search_selected, 2, "appending keeps the reader's place");
    }

    #[test]
    fn there_is_no_next_page_without_a_cursor() {
        let mut app = searching("q");
        app.apply_search_results("q", SearchType::All, hits(), true, None, false);
        assert!(!app.search_has_more);
    }

    #[test]
    fn a_comment_hit_opens_the_post_it_is_on() {
        let mut app = App::new();
        app.open_search();
        let comment = hit("comment");
        app.open_search_post(&comment);
        let post = app.current_post.as_ref().unwrap();
        assert_eq!(app.screen, Screen::PostDetail);
        assert_eq!(post.id, comment.post_id);
        assert_eq!(
            Some(&post.title),
            comment.post.as_ref().unwrap().title.as_ref()
        );
        assert!(
            post.author.is_none(),
            "the comment's author is not the post's"
        );
    }

    #[test]
    fn going_back_from_a_post_opened_by_search_returns_to_the_results() {
        let mut app = App::new();
        app.open_search();
        app.search_results = hits();
        app.open_search_post(&hit("post"));
        app.go_back();
        assert_eq!(app.screen, Screen::Search);
        assert_eq!(
            app.search_results.len(),
            hits().len(),
            "results survive the trip"
        );
    }

    #[test]
    fn leaving_search_returns_to_where_it_was_opened() {
        let mut app = App::new();
        app.screen = Screen::Leaderboard;
        app.open_search();
        app.go_back();
        assert_eq!(app.screen, Screen::Leaderboard);

        // A post or profile behind it may be gone by then, so those fall back to the feed.
        app.screen = Screen::PostDetail;
        app.open_search();
        app.go_back();
        assert_eq!(app.screen, Screen::Feed);
    }

    #[test]
    fn moving_through_results_stops_at_either_end() {
        let mut app = App::new();
        app.screen = Screen::Search;
        app.search_results = hits();
        for _ in 0..10 {
            app.select_next();
        }
        assert_eq!(app.search_selected, hits().len() - 1);
        for _ in 0..10 {
            app.select_previous();
        }
        assert_eq!(app.search_selected, 0);
    }

    // The site's pages, not the API: catches the next time it moves them.
    mod live {
        use super::*;
        use crate::api::{ApiClient, SortOrder, TimeFilter};

        async fn status(path: &str) -> u16 {
            reqwest::get(format!("https://www.moltbook.com{path}"))
                .await
                .unwrap()
                .status()
                .as_u16()
        }

        #[tokio::test]
        #[ignore = "hits the live site"]
        async fn every_browser_link_the_app_opens_exists() {
            let c = ApiClient::new("https://moltbook.com/api/v1".into(), None);
            let mut app = App::new();
            app.posts = c
                .get_posts(SortOrder::Top, Some(TimeFilter::Week), 1, None, None)
                .await
                .unwrap()
                .posts;
            app.leaderboard = c.get_leaderboard().await.unwrap();
            app.submolts = c.get_submolts().await.unwrap();

            for screen in [Screen::Feed, Screen::Leaderboard, Screen::Submolts] {
                app.screen = screen.clone();
                let path = app.web_path().unwrap();
                assert_eq!(status(&path).await, 200, "{screen:?} opens {path}");
            }
        }
    }
}
