use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

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

    async fn list_user_projects(&self, page: u32) -> ProviderResult<Vec<ProjectInfo>> {
        let url = self.api_url("/projects");

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[
                ("membership", "true"),
                ("order_by", "last_activity_at"),
                ("page", &page.to_string()),
                ("per_page", "20"),
            ])
            .send()
            .await?;

        let projects: Vec<ProjectInfo> = resp.error_for_status()?.json().await?;
        Ok(projects)
    }

    async fn search_projects(&self, query: &str) -> ProviderResult<Vec<ProjectInfo>> {
        let url = self.api_url("/projects");

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[
                ("search", query),
                ("per_page", "20"),
            ])
            .send()
            .await?;

        let projects: Vec<ProjectInfo> = resp.error_for_status()?.json().await?;
        Ok(projects)
    }

    async fn list_mr_commits(&self, mr_iid: u64) -> ProviderResult<Vec<Commit>> {
        let url = self.api_url(&format!(
            "/projects/{}/merge_requests/{}/commits",
            self.encoded_project_id(),
            mr_iid
        ));

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Err(ProviderError::NotFound("MR commits not found".into()));
        }

        let commits: Vec<Commit> = resp.error_for_status()?.json().await?;
        Ok(commits)
    }

    async fn list_mr_pipelines(&self, mr_iid: u64) -> ProviderResult<Vec<MrPipeline>> {
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

        if resp.status().as_u16() == 404 {
            return Err(ProviderError::NotFound("MR pipelines not found".into()));
        }

        let pipelines: Vec<MrPipeline> = resp.error_for_status()?.json().await?;
        Ok(pipelines)
    }

    async fn get_pipeline_enriched(&self, pipeline_id: u64) -> ProviderResult<PipelineEnrichedData> {
        // Fetch pipeline detail for duration, user, and stage names
        let detail_url = self.api_url(&format!(
            "/projects/{}/pipelines/{}",
            self.encoded_project_id(),
            pipeline_id
        ));

        #[derive(Deserialize)]
        struct PipelineDetailResp {
            #[serde(default)]
            duration: Option<u64>,
            #[serde(default)]
            user: Option<PipelineUser>,
            #[serde(default)]
            status: String,
        }

        let detail_resp = self
            .client
            .get(&detail_url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        let detail: PipelineDetailResp = detail_resp.error_for_status()?.json().await?;

        // Fetch jobs AND bridges for stage statuses
        let jobs_url = self.api_url(&format!(
            "/projects/{}/pipelines/{}/jobs",
            self.encoded_project_id(),
            pipeline_id
        ));

        #[derive(Deserialize, Clone)]
        struct Job {
            name: String,
            stage: String,
            status: String,
        }

        let jobs_resp = self
            .client
            .get(&jobs_url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[("per_page", "100"), ("include_retried", "false")])
            .send()
            .await?;

        let mut jobs: Vec<Job> = match jobs_resp.error_for_status() {
            Ok(resp) => resp.json().await.unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        // Also fetch bridges (child pipeline triggers count as stages)
        let bridges_url = self.api_url(&format!(
            "/projects/{}/pipelines/{}/bridges",
            self.encoded_project_id(),
            pipeline_id
        ));

        let bridges_resp = self
            .client
            .get(&bridges_url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[("per_page", "100")])
            .send()
            .await;

        if let Ok(resp) = bridges_resp {
            if let Ok(bridges) = resp.json::<Vec<Job>>().await {
                jobs.extend(bridges);
            }
        }

        // Group by stage, preserve order, collect jobs per stage
        let mut stage_map: Vec<StageStatus> = Vec::new();
        for job in &jobs {
            if let Some(entry) = stage_map.iter_mut().find(|s| s.name == job.stage) {
                entry.status = worse_status(&entry.status, &job.status);
                entry.jobs.push(JobInfo { name: job.name.clone(), status: job.status.clone() });
            } else {
                stage_map.push(StageStatus {
                    name: job.stage.clone(),
                    status: job.status.clone(),
                    jobs: vec![JobInfo { name: job.name.clone(), status: job.status.clone() }],
                });
            }
        }

        let stages = if stage_map.is_empty() {
            vec![StageStatus { name: "pipeline".to_string(), status: detail.status, jobs: Vec::new() }]
        } else {
            stage_map
        };

        Ok(PipelineEnrichedData {
            duration: detail.duration,
            user: detail.user,
            stages,
        })
    }
}

fn worse_status(a: &str, b: &str) -> String {
    let priority = |s: &str| match s {
        "failed" => 0,
        "running" => 1,
        "pending" => 2,
        "canceled" => 3,
        "skipped" => 4,
        "success" | "passed" => 5,
        _ => 3,
    };
    if priority(b) < priority(a) { b.to_string() } else { a.to_string() }
}
