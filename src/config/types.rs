use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub gitlab: GitLabConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitLabConfig {
    pub base_url: Option<String>,
    pub project: Option<String>,
}

impl GitLabConfig {
    pub fn base_url_or_default(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://gitlab.com")
    }
}
