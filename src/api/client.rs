use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

use super::models::{
    AgentProfileResponse, Comment, CommentsResponse, LeaderboardAgent, LeaderboardResponse, Post,
    PostResponse, PostsResponse, RecentAgent, RecentAgentsResponse, SearchResponse, SearchType,
    SortOrder, Stats, SubmoltFull, SubmoltsResponse, TimeFilter, TopHuman, TopHumansResponse,
};

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url,
            api_key,
        }
    }

    /// Check if an error is retryable (connection/timeout errors or 5xx status)
    fn is_retryable(error: &anyhow::Error) -> bool {
        if let Some(reqwest_err) = error.downcast_ref::<reqwest::Error>() {
            // Retry on connection errors, timeouts, or 5xx server errors
            if reqwest_err.is_connect() || reqwest_err.is_timeout() {
                return true;
            }
            if let Some(status) = reqwest_err.status() {
                return status.is_server_error();
            }
        }
        false
    }

    /// Retry a request with exponential backoff
    async fn retry_request<T, F, Fut>(&self, mut request_fn: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        const MAX_RETRIES: u32 = 3;
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match request_fn().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Only retry on retryable errors and if we have attempts left
                    if Self::is_retryable(&e) && attempt < MAX_RETRIES - 1 {
                        let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Request failed after retries")))
    }

    /// Build a GET request, optionally adding auth header if api_key is set
    fn get_request(&self, url: &str) -> reqwest::RequestBuilder {
        let req = self.client.get(url);
        if let Some(ref key) = self.api_key {
            if !key.is_empty() {
                return req.header("Authorization", format!("Bearer {}", key));
            }
        }
        req
    }

    pub async fn get_posts(
        &self,
        sort: SortOrder,
        time_filter: Option<TimeFilter>,
        limit: i64,
        cursor: Option<&str>,
        submolt: Option<&str>,
    ) -> Result<PostsResponse> {
        let mut url = format!(
            "{}/posts?sort={}&limit={}",
            self.base_url,
            sort.as_str(),
            limit
        );

        if let Some(cursor) = cursor {
            url.push_str(&format!("&cursor={}", urlencoding::encode(cursor)));
        }

        // Add time filter for Top, Discussed, and Random
        if let Some(time) = time_filter {
            if sort != SortOrder::New {
                url.push_str(&format!("&time={}", time.as_str()));
            }
        }

        // Add shuffle parameter for Random sort
        if sort == SortOrder::Random {
            let shuffle = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            url.push_str(&format!("&shuffle={}", shuffle));
        }

        // Add submolt filter if provided
        if let Some(submolt_name) = submolt {
            url.push_str(&format!("&submolt={}", urlencoding::encode(submolt_name)));
        }

        self.retry_request(|| async {
            self.get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<PostsResponse>()
                .await
                .map_err(Into::into)
        })
        .await
    }

    pub async fn get_post(&self, post_id: &str) -> Result<Post> {
        let url = format!("{}/posts/{}", self.base_url, post_id);

        self.retry_request(|| async {
            let response: PostResponse = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(response.post)
        })
        .await
    }

    /// Search posts, comments, agents and submolts, 20 at a time. The query accepts
    /// the site's syntax: `"exact phrase"`, `a or b`, `-excluded`.
    pub async fn search(
        &self,
        query: &str,
        kind: SearchType,
        cursor: Option<&str>,
    ) -> Result<SearchResponse> {
        let mut url = format!(
            "{}/search?q={}&type={}&limit=20",
            self.base_url,
            urlencoding::encode(query),
            kind.as_str()
        );
        if let Some(cursor) = cursor {
            url.push_str(&format!("&cursor={}", urlencoding::encode(cursor)));
        }

        self.retry_request(|| async {
            self.get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<SearchResponse>()
                .await
                .map_err(Into::into)
        })
        .await
    }

    /// Fetch a post's comment tree, following the cursor until `max` top-level
    /// comments have been read. Replies arrive nested under their parents.
    pub async fn get_comments(&self, post_id: &str, max: usize) -> Result<Vec<Comment>> {
        const PAGE: usize = 100; // the server caps `limit` here
        let mut comments = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/posts/{}/comments?sort=best&limit={}",
                self.base_url, post_id, PAGE
            );
            if let Some(c) = &cursor {
                url.push_str(&format!("&cursor={}", urlencoding::encode(c)));
            }

            let page: CommentsResponse = self
                .retry_request(|| async {
                    self.get_request(&url)
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<CommentsResponse>()
                        .await
                        .map_err(Into::into)
                })
                .await?;

            comments.extend(page.comments);
            match page.next_cursor {
                Some(next) if page.has_more && comments.len() < max => cursor = Some(next),
                _ => break,
            }
        }

        Ok(comments)
    }

    pub async fn get_stats(&self) -> Result<Stats> {
        let url = format!("{}/stats", self.base_url);

        self.retry_request(|| async {
            self.get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<Stats>()
                .await
                .map_err(Into::into)
        })
        .await
    }

    /// The site's leaderboard is the agent directory sorted by karma. The older
    /// `/agents/leaderboard` still answers but the site has stopped calling it.
    pub async fn get_leaderboard(&self) -> Result<Vec<LeaderboardAgent>> {
        let url = format!("{}/agents/recent?limit=50&sort_by=karma", self.base_url);

        self.retry_request(|| async {
            let response = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<LeaderboardResponse>()
                .await?;
            Ok(response
                .leaderboard
                .into_iter()
                .zip(1..)
                .map(|(agent, rank)| LeaderboardAgent { rank, ..agent })
                .collect())
        })
        .await
    }

    pub async fn get_recent_agents(&self) -> Result<Vec<RecentAgent>> {
        let url = format!("{}/agents/recent?limit=50&sort=recent", self.base_url);

        self.retry_request(|| async {
            let response = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<RecentAgentsResponse>()
                .await?;
            Ok(response.agents)
        })
        .await
    }

    pub async fn get_submolts(&self) -> Result<Vec<SubmoltFull>> {
        // Unparameterised, this now returns only 20; ask for a full page as the site does.
        let url = format!(
            "{}/submolts?limit=100&page=1&sort=subscribers",
            self.base_url
        );

        self.retry_request(|| async {
            let response: SubmoltsResponse = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(response.submolts)
        })
        .await
    }

    pub async fn get_top_humans(&self) -> Result<Vec<TopHuman>> {
        let url = format!("{}/agents/top-humans?limit=100", self.base_url);

        self.retry_request(|| async {
            let response: TopHumansResponse = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(response.humans)
        })
        .await
    }

    pub async fn get_agent_profile(&self, name: &str) -> Result<AgentProfileResponse> {
        let url = format!(
            "{}/agents/profile?name={}",
            self.base_url,
            urlencoding::encode(name)
        );

        self.retry_request(|| async {
            self.get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<AgentProfileResponse>()
                .await
                .map_err(Into::into)
        })
        .await
    }
}

// Calls every endpoint the app uses against the real API. Skipped by default
// because it needs the network; run with `cargo test -- --ignored`.
#[cfg(test)]
mod live {
    use super::*;

    const API: &str = "https://moltbook.com/api/v1";

    fn client() -> ApiClient {
        ApiClient::new(API.to_string(), None)
    }

    #[tokio::test]
    #[ignore = "hits the live API"]
    async fn every_sort_pages_forward_without_repeating() {
        let c = client();
        for (sort, time) in [
            (SortOrder::New, None),
            (SortOrder::Top, Some(TimeFilter::Week)),
            (SortOrder::Discussed, Some(TimeFilter::Day)),
        ] {
            let first = c.get_posts(sort, time, 25, None, None).await.unwrap();
            assert!(!first.posts.is_empty(), "{sort:?}: empty first page");
            let cursor = first
                .next_cursor
                .as_deref()
                .unwrap_or_else(|| panic!("{sort:?}: no cursor"));
            let second = c
                .get_posts(sort, time, 25, Some(cursor), None)
                .await
                .unwrap();
            // The server's cursor mishandles ties: a post sharing the sort key of the
            // last one on a page can come back at the top of the next. Allow that, but
            // the page must still move forward.
            let repeats = second
                .posts
                .iter()
                .filter(|p| first.posts.iter().any(|q| q.id == p.id))
                .count();
            assert!(
                repeats <= 2 && repeats < second.posts.len(),
                "{sort:?}: second page repeats {repeats} of the first"
            );
        }
        let random = c
            .get_posts(SortOrder::Random, Some(TimeFilter::Week), 25, None, None)
            .await;
        assert!(!random.unwrap().posts.is_empty(), "Random: empty");
    }

    #[tokio::test]
    #[ignore = "hits the live API"]
    async fn the_submolt_filter_holds() {
        let r = client()
            .get_posts(SortOrder::New, None, 25, None, Some("introductions"))
            .await
            .unwrap();
        assert!(!r.posts.is_empty());
        assert!(r
            .posts
            .iter()
            .all(|p| p.submolt.as_ref().map(|s| s.name.as_str()) == Some("introductions")));
    }

    #[tokio::test]
    #[ignore = "hits the live API"]
    async fn a_busy_post_loads_its_comments() {
        let c = client();
        let post = c
            .get_posts(SortOrder::Discussed, Some(TimeFilter::Week), 1, None, None)
            .await
            .unwrap()
            .posts
            .remove(0);
        let comments = c.get_comments(&post.id, 500).await.unwrap();
        assert!(
            !comments.is_empty(),
            "{} comments claimed, none loaded",
            post.comment_count
        );
    }

    #[tokio::test]
    #[ignore = "hits the live API"]
    async fn search_finds_every_kind_and_pages_forward() {
        let c = client();
        for kind in SearchType::ALL {
            let first = c.search("agent memory", kind, None).await.unwrap();
            assert!(!first.results.is_empty(), "{kind:?}: no results");
            if kind != SearchType::All {
                let want = kind.as_str().trim_end_matches('s');
                assert!(
                    first.results.iter().all(|r| r.kind == want),
                    "{kind:?}: results of another kind"
                );
            }
            if let Some(cursor) = first.next_cursor.as_deref() {
                let second = c.search("agent memory", kind, Some(cursor)).await.unwrap();
                assert!(
                    second
                        .results
                        .iter()
                        .all(|r| first.results.iter().all(|f| f.id != r.id)),
                    "{kind:?}: second page repeats the first"
                );
            }
        }
        let post = c
            .search("agent memory", SearchType::Posts, None)
            .await
            .unwrap()
            .results
            .remove(0);
        assert_eq!(c.get_post(&post.post_id).await.unwrap().id, post.post_id);
    }

    #[tokio::test]
    #[ignore = "hits the live API"]
    async fn the_other_screens_load() {
        let c = client();
        let stats = c.get_stats().await.unwrap();
        assert!(
            stats.agents > 0 && stats.posts > 0,
            "stats are all zero: {stats:?}"
        );
        let leaders = c.get_leaderboard().await.unwrap();
        assert!(!leaders.is_empty(), "leaderboard");
        assert!(
            leaders.iter().zip(1..).all(|(a, r)| a.rank == r),
            "leaderboard ranks are not 1..n"
        );
        assert!(
            leaders.windows(2).all(|w| w[0].karma >= w[1].karma),
            "leaderboard is not sorted by karma"
        );
        assert!(
            !c.get_recent_agents().await.unwrap().is_empty(),
            "recent agents"
        );
        assert!(!c.get_submolts().await.unwrap().is_empty(), "submolts");
        assert!(
            !c.get_top_humans().await.unwrap().is_empty(),
            "top pairings"
        );
        let profile = c.get_agent_profile(&leaders[0].name).await.unwrap();
        assert_eq!(profile.agent.name, leaders[0].name);
    }
}

// Against a local stand-in for the API: what the client asks for, and what it does
// with the answer. Runs offline, unlike `live`.
#[cfg(test)]
mod offline {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    type Seen = Arc<Mutex<Vec<String>>>;

    /// Answer every request with `reply(path)` and record the paths asked for.
    async fn serve(reply: impl Fn(&str) -> String + Send + Sync + 'static) -> (ApiClient, Seen) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let seen: Seen = Arc::default();
        let log = seen.clone();
        let reply = Arc::new(reply);
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let (log, reply) = (log.clone(), reply.clone());
                tokio::spawn(async move {
                    let mut buf = vec![0; 16 * 1024];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
                    log.lock().unwrap().push(path.clone());
                    let body = reply(&path);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        });
        (ApiClient::new(base, None), seen)
    }

    fn comment(id: &str) -> String {
        format!(
            r#"{{"id":"{id}","content":"c","upvotes":0,"downvotes":0,"created_at":"2026-01-01T00:00:00Z"}}"#
        )
    }

    fn post(id: &str) -> String {
        format!(
            r#"{{"id":"{id}","title":"t","upvotes":0,"downvotes":0,"comment_count":0,"created_at":"2026-01-01T00:00:00Z"}}"#
        )
    }

    #[tokio::test]
    async fn the_feed_asks_for_a_cursor_and_never_an_offset() {
        let (c, seen) = serve(|_| {
            format!(
                r#"{{"posts":[{}],"has_more":true,"next_cursor":"n"}}"#,
                post("p")
            )
        })
        .await;
        let page = c
            .get_posts(
                SortOrder::Top,
                Some(TimeFilter::Week),
                25,
                Some("abc=="),
                Some("general"),
            )
            .await
            .unwrap();
        assert_eq!(page.next_cursor.as_deref(), Some("n"));
        let path = seen.lock().unwrap()[0].clone();
        assert!(path.starts_with("/posts?sort=top&limit=25"), "{path}");
        assert!(
            path.contains("&cursor=abc%3D%3D"),
            "cursor is sent, encoded: {path}"
        );
        assert!(
            path.contains("&time=week") && path.contains("&submolt=general"),
            "{path}"
        );
        assert!(!path.contains("offset"), "the API ignores offset: {path}");

        c.get_posts(SortOrder::New, None, 25, None, None)
            .await
            .unwrap();
        let path = seen.lock().unwrap()[1].clone();
        assert!(
            !path.contains("cursor") && !path.contains("time="),
            "first page of New: {path}"
        );
    }

    #[tokio::test]
    async fn comments_follow_the_cursor_until_the_last_page() {
        let (c, seen) = serve(|path| {
            if path.contains("cursor=second") {
                format!(
                    r#"{{"comments":[{}],"has_more":false,"next_cursor":null}}"#,
                    comment("b")
                )
            } else {
                format!(
                    r#"{{"comments":[{}],"has_more":true,"next_cursor":"second"}}"#,
                    comment("a")
                )
            }
        })
        .await;
        let comments = c.get_comments("p1", 500).await.unwrap();
        let ids: Vec<&str> = comments.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert!(
            seen[0].starts_with("/posts/p1/comments?sort=best&limit=100"),
            "{}",
            seen[0]
        );
        assert!(!seen[0].contains("cursor"));
        assert!(seen[1].ends_with("&cursor=second"), "{}", seen[1]);
    }

    #[tokio::test]
    async fn comments_stop_at_the_cap_even_if_the_server_has_more() {
        let (c, seen) = serve(|_| {
            format!(
                r#"{{"comments":[{}],"has_more":true,"next_cursor":"more"}}"#,
                comment("x")
            )
        })
        .await;
        let comments = c.get_comments("p1", 3).await.unwrap();
        assert_eq!(comments.len(), 3);
        assert_eq!(seen.lock().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn the_leaderboard_is_the_karma_sorted_directory_numbered_from_one() {
        let (c, seen) = serve(|_| {
            r#"{"agents":[
                {"id":"1","name":"a","karma":30,"is_claimed":true},
                {"id":"2","name":"b","karma":20,"is_claimed":true},
                {"id":"3","name":"c","karma":10,"is_claimed":false}]}"#
                .to_string()
        })
        .await;
        let ranks: Vec<u32> = c
            .get_leaderboard()
            .await
            .unwrap()
            .iter()
            .map(|a| a.rank)
            .collect();
        assert_eq!(ranks, [1, 2, 3]);
        assert_eq!(
            seen.lock().unwrap()[0],
            "/agents/recent?limit=50&sort_by=karma"
        );
    }

    #[tokio::test]
    async fn each_screen_asks_the_endpoint_the_site_uses() {
        let (c, seen) = serve(|path| {
            match path.split('?').next().unwrap() {
                "/agents/recent" => r#"{"agents":[]}"#,
                "/agents/top-humans" => r#"{"humans":[]}"#,
                "/submolts" => r#"{"submolts":[]}"#,
                "/stats" => {
                    r#"{"totalAgents":1,"totalSubmolts":2,"totalPosts":3,"totalComments":4}"#
                }
                _ => "{}",
            }
            .to_string()
        })
        .await;
        c.get_recent_agents().await.unwrap();
        c.get_top_humans().await.unwrap();
        c.get_submolts().await.unwrap();
        let stats = c.get_stats().await.unwrap();
        assert_eq!((stats.agents, stats.comments), (1, 4));
        assert_eq!(
            *seen.lock().unwrap(),
            [
                "/agents/recent?limit=50&sort=recent",
                "/agents/top-humans?limit=100",
                "/submolts?limit=100&page=1&sort=subscribers",
                "/stats",
            ]
        );
    }

    #[tokio::test]
    async fn search_encodes_the_query_and_sends_type_and_cursor() {
        let (c, seen) = serve(|_| r#"{"results":[],"has_more":false}"#.to_string()).await;
        c.search(r#""exact" a&b -c"#, SearchType::Agents, Some("cur/1"))
            .await
            .unwrap();
        assert_eq!(
            seen.lock().unwrap()[0],
            "/search?q=%22exact%22%20a%26b%20-c&type=agents&limit=20&cursor=cur%2F1"
        );
    }

    #[tokio::test]
    async fn a_single_post_is_read_out_of_its_envelope() {
        let (c, seen) = serve(|_| format!(r#"{{"success":true,"post":{}}}"#, post("p9"))).await;
        assert_eq!(c.get_post("p9").await.unwrap().id, "p9");
        assert_eq!(seen.lock().unwrap()[0], "/posts/p9");
    }
}
