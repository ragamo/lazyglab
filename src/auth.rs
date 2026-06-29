use crate::config::types::AppConfig;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenSource {
    Environment,
    ConfigFile,
    UserInput,
}

pub struct TokenResolution {
    pub token: Option<String>,
    pub source: Option<TokenSource>,
}

pub fn resolve_token(config: &AppConfig) -> TokenResolution {
    if let Ok(token) = std::env::var("LAZYGLAB_TOKEN") {
        if !token.is_empty() {
            return TokenResolution {
                token: Some(token),
                source: Some(TokenSource::Environment),
            };
        }
    }

    if let Ok(token) = std::env::var("GITLAB_TOKEN") {
        if !token.is_empty() {
            return TokenResolution {
                token: Some(token),
                source: Some(TokenSource::Environment),
            };
        }
    }

    if let Some(ref token) = config.auth.token {
        if !token.is_empty() {
            return TokenResolution {
                token: Some(token.clone()),
                source: Some(TokenSource::ConfigFile),
            };
        }
    }

    TokenResolution {
        token: None,
        source: None,
    }
}
