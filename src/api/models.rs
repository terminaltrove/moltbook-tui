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
    pub agents: u64,
    pub submolts: u64,
    pub posts: u64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostDetailResponse {
    pub post: Post,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
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
pub struct HomepageResponse {
    pub success: bool,
    #[serde(rename = "topHumans")]
    pub top_humans: Vec<TopHuman>,
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
