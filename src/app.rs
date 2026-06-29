use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc;

use crate::auth::{self, TokenSource};
use crate::config;
use crate::config::types::AppConfig;
use crate::provider::types::{MergeRequest, Pipeline, User};
use crate::provider::gitlab::GitLabProvider;
use crate::provider::{Provider, ProviderError};

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Splash,
    AuthModal,
    Main,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Tab {
    #[default]
    MergeRequests,
    Pipelines,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
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
    pub pipelines: Vec<Pipeline>,

    pub active_tab: Tab,
    pub projects: Vec<Project>,
    pub selected_project: usize,
    pub project_selector_open: bool,

    pub message_tx: mpsc::UnboundedSender<AppMessage>,
    pub message_rx: mpsc::UnboundedReceiver<AppMessage>,

    pub config: AppConfig,
    pub http_client: reqwest::Client,

    // Click areas (set during render)
    pub tab_mr_area: Option<ratatui::prelude::Rect>,
    pub tab_pipelines_area: Option<ratatui::prelude::Rect>,
    pub project_selector_area: Option<ratatui::prelude::Rect>,
    pub project_items_areas: Vec<ratatui::prelude::Rect>,
}

impl App {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let config = config::load_config().unwrap_or_default();
        let http_client = reqwest::Client::new();

        let mock_projects = vec![
            Project { id: 1, name: "lazyglab".into(), path_with_namespace: "ragamo/lazyglab".into() },
            Project { id: 2, name: "backend-api".into(), path_with_namespace: "ragamo/backend-api".into() },
            Project { id: 3, name: "frontend-app".into(), path_with_namespace: "team/frontend-app".into() },
            Project { id: 4, name: "infra-deploy".into(), path_with_namespace: "devops/infra-deploy".into() },
        ];

        let mock_mrs = vec![
            MergeRequest {
                id: 101, iid: 42, title: "feat: add dark mode support".into(),
                author: User { id: 1, username: "ragamo".into(), name: "Christian".into() },
                state: "opened".into(), source_branch: "feat/dark-mode".into(),
                target_branch: "main".into(), web_url: "https://gitlab.com/ragamo/lazyglab/-/merge_requests/42".into(),
                created_at: "2026-06-28T10:00:00Z".into(), updated_at: "2026-06-29T08:30:00Z".into(),
            },
            MergeRequest {
                id: 102, iid: 41, title: "fix: resolve pipeline timeout on large repos".into(),
                author: User { id: 2, username: "alice".into(), name: "Alice Dev".into() },
                state: "opened".into(), source_branch: "fix/pipeline-timeout".into(),
                target_branch: "main".into(), web_url: "https://gitlab.com/ragamo/lazyglab/-/merge_requests/41".into(),
                created_at: "2026-06-27T14:00:00Z".into(), updated_at: "2026-06-28T16:00:00Z".into(),
            },
            MergeRequest {
                id: 103, iid: 40, title: "refactor: extract provider trait".into(),
                author: User { id: 1, username: "ragamo".into(), name: "Christian".into() },
                state: "opened".into(), source_branch: "refactor/provider-trait".into(),
                target_branch: "main".into(), web_url: "https://gitlab.com/ragamo/lazyglab/-/merge_requests/40".into(),
                created_at: "2026-06-25T09:00:00Z".into(), updated_at: "2026-06-26T11:00:00Z".into(),
            },
        ];

        let mock_pipelines = vec![
            Pipeline { id: 501, status: "success".into(), r#ref: "main".into(), web_url: "https://gitlab.com/pipelines/501".into() },
            Pipeline { id: 500, status: "running".into(), r#ref: "feat/dark-mode".into(), web_url: "https://gitlab.com/pipelines/500".into() },
            Pipeline { id: 499, status: "failed".into(), r#ref: "fix/pipeline-timeout".into(), web_url: "https://gitlab.com/pipelines/499".into() },
            Pipeline { id: 498, status: "success".into(), r#ref: "refactor/provider-trait".into(), web_url: "https://gitlab.com/pipelines/498".into() },
            Pipeline { id: 497, status: "canceled".into(), r#ref: "main".into(), web_url: "https://gitlab.com/pipelines/497".into() },
        ];

        let mut app = Self {
            screen: AppScreen::Splash,
            should_quit: false,
            token_input: String::new(),
            token_source_warning: None,
            auth_error: None,
            is_validating: false,
            provider: None,
            current_user: None,
            merge_requests: mock_mrs,
            pipelines: mock_pipelines,
            active_tab: Tab::default(),
            projects: mock_projects,
            selected_project: 0,
            project_selector_open: false,
            message_tx,
            message_rx,
            config,
            http_client,
            tab_mr_area: None,
            tab_pipelines_area: None,
            project_selector_area: None,
            project_items_areas: Vec::new(),
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
                // Skip auth for now — go straight to main with mock data
                self.screen = AppScreen::Main;
                self.current_user = Some(User {
                    id: 1,
                    username: "ragamo".into(),
                    name: "Christian".into(),
                });
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
                if self.project_selector_open {
                    match key.code {
                        KeyCode::Esc => self.project_selector_open = false,
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.selected_project > 0 {
                                self.selected_project -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.selected_project < self.projects.len().saturating_sub(1) {
                                self.selected_project += 1;
                            }
                        }
                        KeyCode::Enter => self.project_selector_open = false,
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Char('1') => self.active_tab = Tab::MergeRequests,
                        KeyCode::Char('2') => self.active_tab = Tab::Pipelines,
                        KeyCode::Tab => {
                            self.active_tab = match self.active_tab {
                                Tab::MergeRequests => Tab::Pipelines,
                                Tab::Pipelines => Tab::MergeRequests,
                            };
                        }
                        KeyCode::Char('p') => self.project_selector_open = true,
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
            self.should_quit = true;
            return;
        }

        if self.screen != AppScreen::Main {
            return;
        }

        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }

        let pos = (mouse.column, mouse.row);

        if self.project_selector_open {
            for (i, area) in self.project_items_areas.iter().enumerate() {
                if hit(pos, *area) {
                    self.selected_project = i;
                    self.project_selector_open = false;
                    return;
                }
            }
            self.project_selector_open = false;
            return;
        }

        if let Some(area) = self.project_selector_area {
            if hit(pos, area) {
                self.project_selector_open = true;
                return;
            }
        }

        if let Some(area) = self.tab_mr_area {
            if hit(pos, area) {
                self.active_tab = Tab::MergeRequests;
                return;
            }
        }

        if let Some(area) = self.tab_pipelines_area {
            if hit(pos, area) {
                self.active_tab = Tab::Pipelines;
            }
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

fn hit(pos: (u16, u16), area: ratatui::prelude::Rect) -> bool {
    pos.0 >= area.x
        && pos.0 < area.x + area.width
        && pos.1 >= area.y
        && pos.1 < area.y + area.height
}
