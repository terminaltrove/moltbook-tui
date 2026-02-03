use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

use super::models::{
    AgentProfileResponse, HomepageResponse, LeaderboardAgent, LeaderboardResponse, PostDetailResponse,
    PostsResponse, RecentAgent, RecentAgentsResponse, SortOrder, Stats, SubmoltFull, SubmoltsResponse,
    TimeFilter, TopHuman,
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
        offset: i64,
        submolt: Option<&str>,
    ) -> Result<PostsResponse> {
        let mut url = format!(
            "{}/posts?sort={}&limit={}&offset={}",
            self.base_url,
            sort.as_str(),
            limit,
            offset
        );

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

    pub async fn get_post(&self, post_id: &str) -> Result<PostDetailResponse> {
        let url = format!("{}/posts/{}", self.base_url, post_id);

        self.retry_request(|| async {
            self.get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<PostDetailResponse>()
                .await
                .map_err(Into::into)
        })
        .await
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

    pub async fn get_leaderboard(&self) -> Result<Vec<LeaderboardAgent>> {
        let url = format!("{}/agents/leaderboard", self.base_url);

        self.retry_request(|| async {
            let response = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<LeaderboardResponse>()
                .await?;
            Ok(response.leaderboard)
        })
        .await
    }

    pub async fn get_recent_agents(&self) -> Result<Vec<RecentAgent>> {
        let url = format!("{}/agents/recent", self.base_url);

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
        let url = format!("{}/submolts", self.base_url);

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
        let url = format!("{}/homepage", self.base_url);

        self.retry_request(|| async {
            let response: HomepageResponse = self
                .get_request(&url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(response.top_humans)
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
