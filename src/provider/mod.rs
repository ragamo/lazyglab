pub mod types;
pub mod gitlab;

use async_trait::async_trait;
use types::*;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("{0}")]
    Other(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn validate_token(&self) -> ProviderResult<User>;
    async fn list_merge_requests(&self, params: ListMrParams) -> ProviderResult<Vec<MergeRequest>>;
    async fn get_merge_request(&self, mr_id: u64) -> ProviderResult<MergeRequest>;
    async fn get_pipeline_status(&self, mr_id: u64) -> ProviderResult<PipelineStatus>;
    async fn list_pipelines(&self, params: ListPipelineParams) -> ProviderResult<Vec<Pipeline>>;
    async fn list_user_projects(&self, page: u32) -> ProviderResult<Vec<ProjectInfo>>;
    async fn search_projects(&self, query: &str) -> ProviderResult<Vec<ProjectInfo>>;
    async fn list_mr_commits(&self, mr_iid: u64) -> ProviderResult<Vec<Commit>>;
}
