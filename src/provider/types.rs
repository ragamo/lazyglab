use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ListMrParams {
    pub state: MrState,
    pub page: u32,
    pub per_page: u32,
}

impl Default for ListMrParams {
    fn default() -> Self {
        Self {
            state: MrState::Open,
            page: 1,
            per_page: 20,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum MrState {
    #[default]
    Open,
    Closed,
    Merged,
    All,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MergeRequest {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub author: User,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub web_url: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pipeline {
    pub id: u64,
    pub status: String,
    pub r#ref: String,
    pub web_url: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub user: Option<PipelineUser>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineMrRef {
    pub iid: u64,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct PipelineStatus {
    pub overall: String,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ListPipelineParams {
    pub page: u32,
    pub per_page: u32,
}

impl Default for ListPipelineParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    pub short_id: String,
    pub title: String,
    pub author_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MrPipeline {
    pub id: u64,
    pub status: String,
    pub r#ref: String,
    pub web_url: String,
    pub created_at: String,
    #[serde(default)]
    pub duration: Option<u64>,
    #[serde(default)]
    pub user: Option<PipelineUser>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineUser {
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct PipelineEnrichedData {
    pub duration: Option<u64>,
    pub user: Option<PipelineUser>,
    pub stages: Vec<StageStatus>,
    pub mr_ref: Option<PipelineMrRef>,
}

#[derive(Debug, Clone)]
pub struct StageStatus {
    pub name: String,
    pub status: String,
    pub jobs: Vec<JobInfo>,
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub name: String,
    pub status: String,
    pub sub_jobs: Vec<JobInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectInfo {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
}
