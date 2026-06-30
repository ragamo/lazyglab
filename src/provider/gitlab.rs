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

        let pipelines: Vec<Pipeline> = resp.error_for_status()?.json().await?;
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
        // Extract path from full URL (e.g. https://gitlab.com/group/project → group/project)
        let path = if query.starts_with("http://") || query.starts_with("https://") {
            query
                .trim_end_matches('/')
                .split('/')
                .skip(3) // skip scheme + empty + host
                .collect::<Vec<_>>()
                .join("/")
        } else {
            query.to_string()
        };

        // If the query looks like a namespace path (contains /), try direct lookup first
        if path.contains('/') {
            let encoded = path.replace('/', "%2F");
            let direct_url = self.api_url(&format!("/projects/{}", encoded));
            let resp = self
                .client
                .get(&direct_url)
                .header("PRIVATE-TOKEN", &self.token)
                .send()
                .await?;
            if resp.status().is_success() {
                if let Ok(project) = resp.json::<ProjectInfo>().await {
                    return Ok(vec![project]);
                }
            }
        }

        // Fall back to name/keyword search
        let url = self.api_url("/projects");
        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .query(&[
                ("search", path.as_str()),
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
        struct MrRefResp {
            iid: u64,
            title: String,
        }

        #[derive(Deserialize)]
        struct PipelineDetailResp {
            #[serde(default)]
            duration: Option<u64>,
            #[serde(default)]
            user: Option<PipelineUser>,
            #[serde(default)]
            status: String,
            #[serde(default)]
            merge_request: Option<MrRefResp>,
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
            id: u64,
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

        // Fetch bridges — each may have a downstream child pipeline
        #[derive(Deserialize)]
        struct DownstreamPipeline {
            id: u64,
        }

        #[derive(Deserialize)]
        struct Bridge {
            id: u64,
            name: String,
            stage: String,
            status: String,
            #[serde(default)]
            downstream_pipeline: Option<DownstreamPipeline>,
        }

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

        // For each bridge with a downstream pipeline, fetch its jobs
        let mut bridge_jobs: Vec<JobInfo> = Vec::new();
        if let Ok(resp) = bridges_resp {
            if let Ok(bridges) = resp.json::<Vec<Bridge>>().await {
                for bridge in &bridges {
                    let mut sub_jobs: Vec<JobInfo> = Vec::new();
                    if let Some(ref downstream) = bridge.downstream_pipeline {
                        let child_jobs_url = self.api_url(&format!(
                            "/projects/{}/pipelines/{}/jobs",
                            self.encoded_project_id(),
                            downstream.id
                        ));
                        let child_resp = self
                            .client
                            .get(&child_jobs_url)
                            .header("PRIVATE-TOKEN", &self.token)
                            .query(&[("per_page", "100")])
                            .send()
                            .await;
                        if let Ok(r) = child_resp {
                            if let Ok(child_jobs) = r.json::<Vec<Job>>().await {
                                sub_jobs = child_jobs.into_iter().rev()
                                    .map(|j| JobInfo { id: j.id, name: j.name, status: j.status, sub_jobs: Vec::new() })
                                    .collect();
                            }
                        }
                    }
                    bridge_jobs.push(JobInfo {
                        id: bridge.id,
                        name: bridge.name.clone(),
                        status: bridge.status.clone(),
                        sub_jobs,
                    });
                    // Add bridge as a regular job entry for stage grouping
                    jobs.push(Job { id: bridge.id, name: bridge.name.clone(), stage: bridge.stage.clone(), status: bridge.status.clone() });
                }
            }
        }

        // Group by stage, preserve order, collect jobs per stage
        let mut stage_map: Vec<StageStatus> = Vec::new();
        for job in &jobs {
            // Check if this job is a bridge (has sub_jobs)
            let job_info = if let Some(bj) = bridge_jobs.iter().find(|b| b.name == job.name) {
                bj.clone()
            } else {
                JobInfo { id: job.id, name: job.name.clone(), status: job.status.clone(), sub_jobs: Vec::new() }
            };

            if let Some(entry) = stage_map.iter_mut().find(|s| s.name == job.stage) {
                entry.status = worse_status(&entry.status, &job.status);
                if !entry.jobs.iter().any(|j| j.name == job.name) {
                    entry.jobs.push(job_info);
                }
            } else {
                stage_map.push(StageStatus {
                    name: job.stage.clone(),
                    status: job.status.clone(),
                    jobs: vec![job_info],
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
            mr_ref: detail.merge_request.map(|mr| PipelineMrRef { iid: mr.iid, title: mr.title }),
        })
    }

    async fn get_job_log(&self, job_id: u64) -> ProviderResult<String> {
        let url = self.api_url(&format!(
            "/projects/{}/jobs/{}/trace",
            self.encoded_project_id(),
            job_id
        ));

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Err(ProviderError::NotFound("Job log not found".into()));
        }

        let log = resp.error_for_status()?.text().await?;
        Ok(log)
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
