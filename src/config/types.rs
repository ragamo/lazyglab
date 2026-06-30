use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub gitlab: GitLabConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitLabConfig {
    pub base_url: Option<String>,
    pub project: Option<String>,
    #[serde(default)]
    pub favorites: Vec<FavoriteProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
}

impl GitLabConfig {
    pub fn base_url_or_default(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://gitlab.com")
    }
}
