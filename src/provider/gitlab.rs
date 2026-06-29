use async_trait::async_trait;
use reqwest::Client;

use super::{Provider, ProviderError, ProviderResult};
use super::types::*;

pub struct GitLabProvider {
    client: Client,
    base_url: String,
    token: String,
    project_id: String,
}

impl GitLabProvider {
    pub fn new(client: Client, token: String, base_url: String, project_id: String) -> Self {
        Self {
            client,
            base_url,
            token,
            project_id,
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/api/v4{}", self.base_url, path)
    }

    fn encoded_project_id(&self) -> String {
        self.project_id.replace("/", "%2F")
    }
}

#[async_trait]
impl Provider for GitLabProvider {
    fn name(&self) -> &'static str {
        "GitLab"
    }

    async fn validate_token(&self) -> ProviderResult<User> {
        let resp = self
            .client
            .get(self.api_url("/user"))
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::AuthFailed("Invalid token".into()));
        }

        let user: User = resp.error_for_status()?.json().await?;
        Ok(user)
    }

    async fn list_merge_requests(&self, params: ListMrParams) -> ProviderResult<Vec<MergeRequest>> {
        let state = match params.state {
            MrState::Open => "opened",
            MrState::Closed => "closed",
            MrState::Merged => "merged",
            MrState::All => "all",
        };

        let url = self.api_url(&format!(
            "/projects/{}/merge_requests",
            self.encoded_project_id()
        ));

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[
                ("state", state),
                ("page", &params.page.to_string()),
                ("per_page", &params.per_page.to_string()),
            ])
            .send()
            .await?;

        let mrs: Vec<MergeRequest> = resp.error_for_status()?.json().await?;
        Ok(mrs)
    }

    async fn get_merge_request(&self, mr_iid: u64) -> ProviderResult<MergeRequest> {
        let url = self.api_url(&format!(
            "/projects/{}/merge_requests/{}",
            self.encoded_project_id(),
            mr_iid
        ));

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ProviderError::NotFound(format!("MR !{}", mr_iid)));
        }

        let mr: MergeRequest = resp.error_for_status()?.json().await?;
        Ok(mr)
    }

    async fn get_pipeline_status(&self, mr_iid: u64) -> ProviderResult<PipelineStatus> {
        let url = self.api_url(&format!(
            "/projects/{}/merge_requests/{}/pipelines",
            self.encoded_project_id(),
            mr_iid
        ));

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        let pipelines: Vec<serde_json::Value> = resp.error_for_status()?.json().await?;

        let Some(latest) = pipelines.first() else {
            return Ok(PipelineStatus {
                overall: "none".into(),
                stages: vec![],
            });
        };

        let overall = latest["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(PipelineStatus {
            overall,
            stages: vec![],
        })
    }

    async fn list_pipelines(&self, params: ListPipelineParams) -> ProviderResult<Vec<Pipeline>> {
        let url = self.api_url(&format!(
            "/projects/{}/pipelines",
            self.encoded_project_id()
        ));

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[
                ("page", &params.page.to_string()),
                ("per_page", &params.per_page.to_string()),
            ])
            .send()
            .await?;

        #[derive(serde::Deserialize)]
        struct PipelineResponse {
            id: u64,
            status: String,
            r#ref: String,
            web_url: String,
        }

        let raw: Vec<PipelineResponse> = resp.error_for_status()?.json().await?;
        let pipelines = raw
            .into_iter()
            .map(|p| Pipeline {
                id: p.id,
                status: p.status,
                r#ref: p.r#ref,
                web_url: p.web_url,
            })
            .collect();

        Ok(pipelines)
    }
}
