use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submolt {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoltCreator {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoltFull {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub subscriber_count: i64,
    pub created_at: String,
    pub last_activity_at: Option<String>,
    pub featured_at: Option<String>,
    pub created_by: Option<SubmoltCreator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    // `/stats` names these `totalAgents` etc.; `/homepage`'s `stats` uses the bare names.
    #[serde(alias = "totalAgents")]
    pub agents: u64,
    #[serde(alias = "totalSubmolts")]
    pub submolts: u64,
    #[serde(alias = "totalPosts")]
    pub posts: u64,
    #[serde(alias = "totalComments")]
    pub comments: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOwner {
    pub x_handle: Option<String>,
    pub x_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardAgent {
    pub id: String,
    pub name: String,
    pub karma: i64,
    pub is_claimed: bool,
    pub avatar_url: Option<String>,
    // The karma-sorted agent list carries no rank; `get_leaderboard` numbers it.
    #[serde(default)]
    pub rank: u32,
    pub owner: Option<AgentOwner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOwnerFull {
    pub x_handle: Option<String>,
    pub x_name: Option<String>,
    pub x_follower_count: Option<i64>,
    pub x_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentAgent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub karma: i64,
    pub follower_count: i64,
    pub created_at: String,
    pub is_claimed: bool,
    pub owner: Option<AgentOwnerFull>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub karma: i64,
    pub follower_count: i64,
    pub following_count: i64,
    #[serde(alias = "posts_count")]
    pub post_count: Option<i64>,
    pub created_at: String,
    pub is_claimed: bool,
    pub owner: Option<AgentOwnerFull>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfileResponse {
    pub agent: AgentProfile,
    #[serde(default, rename = "isFollowing")]
    pub is_following: bool,
    #[serde(default, rename = "recentPosts")]
    pub recent_posts: Vec<Post>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    // A profile's `recentPosts` carry only a `content_preview`.
    #[serde(alias = "content_preview")]
    pub content: Option<String>,
    pub url: Option<String>,
    pub upvotes: i64,
    pub downvotes: i64,
    pub comment_count: i64,
    pub created_at: String,
    pub author: Option<Agent>,
    pub submolt: Option<Submolt>,
}

impl Post {
    pub fn score(&self) -> i64 {
        self.upvotes - self.downvotes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub content: String,
    pub upvotes: i64,
    pub downvotes: i64,
    #[serde(default)]
    pub depth: i32,
    pub created_at: String,
    pub author: Option<Agent>,
    #[serde(default)]
    pub replies: Vec<Comment>,
}

impl Comment {
    pub fn score(&self) -> i64 {
        self.upvotes - self.downvotes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsResponse {
    pub posts: Vec<Post>,
    #[serde(default)]
    pub has_more: bool,
    // Paging is by cursor; `offset` is accepted and ignored. Random sort returns none.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub post: Post,
}

// `/posts/{id}` no longer embeds comments; they come from `/posts/{id}/comments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    #[serde(alias = "agents")]
    pub leaderboard: Vec<LeaderboardAgent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentAgentsResponse {
    pub agents: Vec<RecentAgent>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubmoltsResponse {
    pub submolts: Vec<SubmoltFull>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopHuman {
    pub id: String,
    pub x_id: String,
    pub x_handle: String,
    pub x_name: String,
    pub x_avatar: Option<String>,
    pub x_follower_count: i64,
    pub x_verified: bool,
    pub bot_count: i32,
    pub bot_name: String,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopHumansResponse {
    // `/agents/top-humans` says `humans`; `/homepage` embeds the same list as `topHumans`.
    #[serde(alias = "topHumans")]
    pub humans: Vec<TopHuman>,
}

/// What `/search` looks through. The site's own selector offers exactly these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchType {
    #[default]
    All,
    Posts,
    Comments,
    Agents,
    Submolts,
}

impl SearchType {
    pub const ALL: [SearchType; 5] = [
        SearchType::All,
        SearchType::Posts,
        SearchType::Comments,
        SearchType::Agents,
        SearchType::Submolts,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SearchType::All => "all",
            SearchType::Posts => "posts",
            SearchType::Comments => "comments",
            SearchType::Agents => "agents",
            SearchType::Submolts => "submolts",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SearchType::All => "All",
            SearchType::Posts => "Posts",
            SearchType::Comments => "Comments",
            SearchType::Agents => "Agents",
            SearchType::Submolts => "Submolts",
        }
    }

    pub fn cycle_next(&self) -> Self {
        let i = Self::ALL.iter().position(|t| t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAuthor {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPostRef {
    pub id: String,
    pub title: Option<String>,
}

/// One hit from `/search`. Every kind shares this shape and says which it is in
/// `kind`: a post or comment has `post_id`, an agent its name in `title`, a submolt
/// its name in `submolt` and its subscriber count in `upvotes`. `content` is a
/// snippet with matches wrapped in `⟦HL⟧…⟦/HL⟧`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: Option<String>,
    pub content: Option<String>,
    #[serde(default)]
    pub upvotes: i64,
    #[serde(default)]
    pub downvotes: i64,
    pub created_at: Option<String>,
    pub author: Option<SearchAuthor>,
    pub submolt: Option<Submolt>,
    pub post: Option<SearchPostRef>,
    #[serde(default)]
    pub post_id: String,
    /// The hit's page on the site, e.g. `/post/{id}` or `/u/{name}`.
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    New,
    Top,
    Discussed, // API uses "comments"
    Random,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::New => "new",
            SortOrder::Top => "top",
            SortOrder::Discussed => "comments",
            SortOrder::Random => "random",
        }
    }
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::New => write!(f, "New"),
            SortOrder::Top => write!(f, "Top"),
            SortOrder::Discussed => write!(f, "Discussed"),
            SortOrder::Random => write!(f, "Random"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFilter {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

impl TimeFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFilter::Hour => "hour",
            TimeFilter::Day => "day",
            TimeFilter::Week => "week",
            TimeFilter::Month => "month",
            TimeFilter::Year => "year",
            TimeFilter::All => "all",
        }
    }
}

impl std::fmt::Display for TimeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeFilter::Hour => write!(f, "Hour"),
            TimeFilter::Day => write!(f, "Day"),
            TimeFilter::Week => write!(f, "Week"),
            TimeFilter::Month => write!(f, "Month"),
            TimeFilter::Year => write!(f, "Year"),
            TimeFilter::All => write!(f, "All"),
        }
    }
}

// Real responses from moltbook.com, trimmed to one item per list. When the API
// changes shape again, these say which endpoint and which field.
#[cfg(test)]
mod tests {
    use super::*;

    fn parse<T: serde::de::DeserializeOwned>(json: &str) -> T {
        serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn posts_page_carries_a_cursor() {
        let r: PostsResponse = parse(include_str!("fixtures/posts.json"));
        assert!(!r.posts.is_empty());
        assert!(r.has_more);
        assert!(r.next_cursor.is_some());
    }

    #[test]
    fn comments_arrive_threaded() {
        let r: CommentsResponse = parse(include_str!("fixtures/comments.json"));
        assert!(!r.comments[0].replies.is_empty());
        assert_eq!(r.comments[0].replies[0].depth, 1);
    }

    #[test]
    fn stats_read_the_total_names() {
        let s: Stats = parse(include_str!("fixtures/stats.json"));
        assert!(s.agents > 0 && s.posts > 0);
    }

    #[test]
    fn search_results_of_every_kind() {
        let r: SearchResponse = parse(include_str!("fixtures/search.json"));
        let kinds: Vec<&str> = r.results.iter().map(|x| x.kind.as_str()).collect();
        assert_eq!(kinds, ["post", "comment", "agent", "submolt"]);
        assert!(r.results.iter().all(|x| x.content.is_some()));
    }

    #[test]
    fn a_single_post() {
        let r: PostResponse = parse(include_str!("fixtures/post.json"));
        assert!(r.post.content.is_some());
    }

    #[test]
    fn top_humans() {
        let r: TopHumansResponse = parse(include_str!("fixtures/top_humans.json"));
        assert!(r.humans[0].rank > 0);
    }

    #[test]
    fn leaderboard_reads_the_karma_sorted_agent_list() {
        let r: LeaderboardResponse = parse(include_str!("fixtures/leaderboard.json"));
        assert!(!r.leaderboard.is_empty());
    }

    #[test]
    fn recent_agents() {
        let r: RecentAgentsResponse = parse(include_str!("fixtures/recent_agents.json"));
        assert!(!r.agents.is_empty());
    }

    #[test]
    fn agent_profile_with_post_previews() {
        let r: AgentProfileResponse = parse(include_str!("fixtures/agent_profile.json"));
        assert!(r.agent.post_count.is_some());
        assert!(r.recent_posts.iter().all(|p| p.content.is_some()));
    }

    #[test]
    fn submolts() {
        let r: SubmoltsResponse = parse(include_str!("fixtures/submolts.json"));
        assert!(!r.submolts.is_empty());
    }
}
