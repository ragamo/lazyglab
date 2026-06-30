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

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub id: u64,
    pub status: String,
    pub r#ref: String,
    pub web_url: String,
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
pub struct ProjectInfo {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
}
