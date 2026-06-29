use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc;

use crate::auth::{self, TokenSource};
use crate::config;
use crate::config::types::AppConfig;
use crate::provider::types::{MergeRequest, User};
use crate::provider::gitlab::GitLabProvider;
use crate::provider::{Provider, ProviderError};

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Splash,
    AuthModal,
    Main,
}

pub enum AppMessage {
    TokenValidated(Result<User, ProviderError>),
    MergeRequestsLoaded(Result<Vec<MergeRequest>, ProviderError>),
}

pub struct App {
    pub screen: AppScreen,
    pub should_quit: bool,

    pub token_input: String,
    pub token_source_warning: Option<String>,
    pub auth_error: Option<String>,
    pub is_validating: bool,

    pub provider: Option<Box<dyn Provider>>,
    pub current_user: Option<User>,
    pub merge_requests: Vec<MergeRequest>,

    pub message_tx: mpsc::UnboundedSender<AppMessage>,
    pub message_rx: mpsc::UnboundedReceiver<AppMessage>,

    pub config: AppConfig,
    pub http_client: reqwest::Client,
}

impl App {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let config = config::load_config().unwrap_or_default();
        let http_client = reqwest::Client::new();

        let mut app = Self {
            screen: AppScreen::Splash,
            should_quit: false,
            token_input: String::new(),
            token_source_warning: None,
            auth_error: None,
            is_validating: false,
            provider: None,
            current_user: None,
            merge_requests: Vec::new(),
            message_tx,
            message_rx,
            config,
            http_client,
        };

        app.try_auto_auth();
        app
    }

    fn try_auto_auth(&mut self) {
        let resolution = auth::resolve_token(&self.config);

        match resolution.token {
            Some(token) => {
                if resolution.source == Some(TokenSource::ConfigFile) {
                    self.token_source_warning =
                        Some("⚠ Token loaded from config file".into());
                }
                self.token_input = token;
                self.submit_token();
            }
            None => {
                self.screen = AppScreen::AuthModal;
            }
        }
    }

    fn submit_token(&mut self) {
        if self.token_input.is_empty() {
            return;
        }

        self.is_validating = true;
        self.auth_error = None;

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        let project_id = self.config.gitlab.project.clone().unwrap_or_default();

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project_id);
            let result = provider.validate_token().await;
            let _ = tx.send(AppMessage::TokenValidated(result));
        });
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.screen {
            AppScreen::Splash => {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    self.should_quit = true;
                }
            }
            AppScreen::AuthModal => {
                if self.is_validating {
                    return;
                }
                match key.code {
                    KeyCode::Esc => self.should_quit = true,
                    KeyCode::Enter => self.submit_token(),
                    KeyCode::Backspace => {
                        self.token_input.pop();
                    }
                    KeyCode::Char(c) => {
                        self.token_input.push(c);
                    }
                    _ => {}
                }
            }
            AppScreen::Main => {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    self.should_quit = true;
                }
            }
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
            self.should_quit = true;
        }
    }

    pub fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::TokenValidated(Ok(user)) => {
                self.is_validating = false;
                self.current_user = Some(user);
                self.screen = AppScreen::Main;

                self.config.auth.token = Some(self.token_input.clone());
                let _ = config::save_config(&self.config);

                let tx = self.message_tx.clone();
                let client = self.http_client.clone();
                let token = self.token_input.clone();
                let base_url = self.config.gitlab.base_url_or_default().to_string();
                let project_id = self.config.gitlab.project.clone().unwrap_or_default();

                tokio::spawn(async move {
                    let provider = GitLabProvider::new(client, token, base_url, project_id);
                    let result = provider
                        .list_merge_requests(Default::default())
                        .await;
                    let _ = tx.send(AppMessage::MergeRequestsLoaded(result));
                });
            }
            AppMessage::TokenValidated(Err(e)) => {
                self.is_validating = false;
                self.auth_error = Some(e.to_string());
                self.screen = AppScreen::AuthModal;
            }
            AppMessage::MergeRequestsLoaded(Ok(mrs)) => {
                self.merge_requests = mrs;
            }
            AppMessage::MergeRequestsLoaded(Err(_)) => {}
        }
    }
}
