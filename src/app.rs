use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc;

use crate::auth::{self, TokenSource};
use crate::config;
use crate::config::types::AppConfig;
use crate::provider::types::{ListMrParams, MergeRequest, MrState, Pipeline, ProjectInfo, User};
use crate::provider::gitlab::GitLabProvider;
use crate::provider::{Provider, ProviderError};
use crate::theme::{self, Theme};

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

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MrFilter {
    #[default]
    Open,
    Merged,
    Closed,
    All,
}

impl MrFilter {
    pub const ALL_FILTERS: &[MrFilter] = &[
        MrFilter::Open,
        MrFilter::Merged,
        MrFilter::Closed,
        MrFilter::All,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            MrFilter::Open => "open",
            MrFilter::Merged => "merged",
            MrFilter::Closed => "closed",
            MrFilter::All => "all",
        }
    }

    pub fn matches(&self, state: &str) -> bool {
        match self {
            MrFilter::Open => state == "opened",
            MrFilter::Merged => state == "merged",
            MrFilter::Closed => state == "closed",
            MrFilter::All => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub is_favorite: bool,
}

pub enum AppMessage {
    TokenValidated(Result<User, ProviderError>),
    MergeRequestsLoaded(Result<Vec<MergeRequest>, ProviderError>),
    PipelinesLoaded(Result<Vec<Pipeline>, ProviderError>),
    ProjectsLoaded(Result<Vec<ProjectInfo>, ProviderError>),
    SearchResults(Result<Vec<ProjectInfo>, ProviderError>),
    Tick,
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
    pub mrs_loading: bool,
    pub pipelines: Vec<Pipeline>,
    pub pipelines_loading: bool,
    pub autoreload_pipelines: bool,
    pub refresh_interval_secs: u64,
    pub last_pipeline_refresh: Option<Instant>,
    pub autoreload_checkbox_area: Option<ratatui::prelude::Rect>,

    pub active_tab: Tab,
    pub mr_filter: MrFilter,
    pub projects: Vec<Project>,
    pub selected_project: usize,
    pub project_selector_open: bool,

    // Find modal
    pub find_modal_open: bool,
    pub find_input: String,
    pub find_results: Vec<Project>,
    pub find_selected: usize,
    pub find_loading: bool,

    pub message_tx: mpsc::UnboundedSender<AppMessage>,
    pub message_rx: mpsc::UnboundedReceiver<AppMessage>,

    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub logout_link_area: Option<ratatui::prelude::Rect>,
    pub settings_link_area: Option<ratatui::prelude::Rect>,

    // Settings modal
    pub settings_open: bool,
    pub settings_selected: usize,
    pub theme_selector_open: bool,
    pub theme_selected: usize,
    pub theme_confirmed: usize,
    pub theme: &'static Theme,

    // Click areas (set during render)
    pub tab_mr_area: Option<ratatui::prelude::Rect>,
    pub tab_pipelines_area: Option<ratatui::prelude::Rect>,
    pub project_selector_area: Option<ratatui::prelude::Rect>,
    pub project_items_areas: Vec<ratatui::prelude::Rect>,
    pub mr_filter_areas: Vec<ratatui::prelude::Rect>,
    pub find_link_area: Option<ratatui::prelude::Rect>,
    pub find_result_areas: Vec<ratatui::prelude::Rect>,
    pub find_star_areas: Vec<ratatui::prelude::Rect>,
}

impl App {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let config = config::load_config().unwrap_or_default();
        let http_client = reqwest::Client::new();


        let active_theme = config
            .ui
            .theme
            .as_deref()
            .map(theme::find_theme)
            .unwrap_or(&theme::ONE_DARK);

        let theme_selected = theme::ALL_THEMES
            .iter()
            .position(|t| t.name == active_theme.name)
            .unwrap_or(0);

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
            mrs_loading: false,
            pipelines: Vec::new(),
            pipelines_loading: false,
            autoreload_pipelines: true,
            refresh_interval_secs: 30,
            last_pipeline_refresh: None,
            autoreload_checkbox_area: None,
            active_tab: Tab::default(),
            mr_filter: MrFilter::default(),
            projects: config.gitlab.favorites.iter().map(|f| Project {
                id: f.id,
                name: f.name.clone(),
                path_with_namespace: f.path_with_namespace.clone(),
                is_favorite: true,
            }).collect(),
            selected_project: 0,
            project_selector_open: false,
            find_modal_open: false,
            find_input: String::new(),
            find_results: Vec::new(),
            find_selected: 0,
            find_loading: false,
            message_tx,
            message_rx,
            config,
            http_client,
            settings_open: false,
            settings_selected: 0,
            theme_selector_open: false,
            theme_selected,
            theme_confirmed: theme_selected,
            theme: active_theme,
            tab_mr_area: None,
            tab_pipelines_area: None,
            project_selector_area: None,
            project_items_areas: Vec::new(),
            mr_filter_areas: Vec::new(),
            logout_link_area: None,
            settings_link_area: None,
            find_link_area: None,
            find_result_areas: Vec::new(),
            find_star_areas: Vec::new(),
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
                if self.find_modal_open {
                    self.handle_find_key(key);
                } else if self.settings_open {
                    self.handle_settings_key(key);
                } else if self.project_selector_open {
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
                        KeyCode::Enter => {
                            self.project_selector_open = false;
                            self.load_merge_requests();
                            self.load_pipelines();
                        }
                        KeyCode::Char('s') => {
                            if let Some(project) = self.projects.get(self.selected_project) {
                                let id = project.id;
                                self.remove_favorite(id);
                            }
                        }
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
                        KeyCode::Left if self.active_tab == Tab::MergeRequests => {
                            self.cycle_mr_filter_back();
                        }
                        KeyCode::Right if self.active_tab == Tab::MergeRequests => {
                            self.cycle_mr_filter_forward();
                        }
                        KeyCode::Char('p') => self.project_selector_open = true,
                        KeyCode::Char('f') => {
                            self.find_modal_open = true;
                            self.find_input.clear();
                            self.find_results.clear();
                            self.find_selected = 0;
                        }
                        KeyCode::Char('r') => {
                            self.load_merge_requests();
                            self.load_pipelines();
                        }
                        KeyCode::Char(',') => self.settings_open = true,
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_find_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.find_modal_open = false;
            }
            KeyCode::Enter => {
                if self.find_results.is_empty() {
                    self.find_loading = true;
                    self.search_projects();
                } else if let Some(project) = self.find_results.get(self.find_selected).cloned() {
                    self.add_favorite(&project);
                    self.selected_project = self
                        .projects
                        .iter()
                        .position(|p| p.id == project.id)
                        .unwrap_or(0);
                    self.find_modal_open = false;
                    self.load_merge_requests();
                    self.load_pipelines();
                }
            }
            KeyCode::Up => {
                if self.find_selected > 0 {
                    self.find_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.find_selected < self.find_results.len().saturating_sub(1) {
                    self.find_selected += 1;
                }
            }
            KeyCode::Char('s') if !self.find_results.is_empty() => {
                if let Some(project) = self.find_results.get(self.find_selected).cloned() {
                    if project.is_favorite {
                        self.remove_favorite(project.id);
                    } else {
                        self.add_favorite(&project);
                    }
                }
            }
            KeyCode::Backspace => {
                self.find_input.pop();
                self.find_results.clear();
            }
            KeyCode::Char(c) => {
                self.find_input.push(c);
                self.find_results.clear();
            }
            _ => {}
        }
    }

    fn cycle_mr_filter_forward(&mut self) {
        let filters = MrFilter::ALL_FILTERS;
        let idx = filters.iter().position(|f| *f == self.mr_filter).unwrap_or(0);
        self.mr_filter = filters[(idx + 1) % filters.len()].clone();
    }

    fn cycle_mr_filter_back(&mut self) {
        let filters = MrFilter::ALL_FILTERS;
        let idx = filters.iter().position(|f| *f == self.mr_filter).unwrap_or(0);
        self.mr_filter = filters[(idx + filters.len() - 1) % filters.len()].clone();
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        if self.theme_selector_open {
            match key.code {
                KeyCode::Esc => {
                    self.theme = theme::ALL_THEMES[self.theme_confirmed];
                    self.theme_selected = self.theme_confirmed;
                    self.theme_selector_open = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.theme_selected > 0 {
                        self.theme_selected -= 1;
                        self.theme = theme::ALL_THEMES[self.theme_selected];
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.theme_selected < theme::ALL_THEMES.len().saturating_sub(1) {
                        self.theme_selected += 1;
                        self.theme = theme::ALL_THEMES[self.theme_selected];
                    }
                }
                KeyCode::Enter => {
                    self.theme_confirmed = self.theme_selected;
                    self.config.ui.theme = Some(self.theme.name.to_string());
                    let _ = config::save_config(&self.config);
                    self.theme_selector_open = false;
                    self.settings_open = false;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc => self.settings_open = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.settings_selected > 0 {
                        self.settings_selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Only 1 item for now
                    let _ = self.settings_selected;
                }
                KeyCode::Enter => {
                    if self.settings_selected == 0 {
                        self.theme_selector_open = true;
                    }
                }
                _ => {}
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
                    self.load_merge_requests();
                    self.load_pipelines();
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

        if let Some(area) = self.logout_link_area {
            if hit(pos, area) {
                self.logout();
                return;
            }
        }

        if let Some(area) = self.settings_link_area {
            if hit(pos, area) {
                self.settings_open = true;
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
                return;
            }
        }

        if self.active_tab == Tab::Pipelines {
            if let Some(area) = self.autoreload_checkbox_area {
                if hit(pos, area) {
                    self.autoreload_pipelines = !self.autoreload_pipelines;
                    return;
                }
            }
        }

        for (i, area) in self.mr_filter_areas.iter().enumerate() {
            if hit(pos, *area) {
                if let Some(f) = MrFilter::ALL_FILTERS.get(i) {
                    self.mr_filter = f.clone();
                }
                return;
            }
        }

        if let Some(area) = self.find_link_area {
            if hit(pos, area) {
                self.find_modal_open = true;
                self.find_input.clear();
                self.find_results.clear();
                self.find_selected = 0;
                return;
            }
        }

        if self.find_modal_open {
            for (i, area) in self.find_star_areas.iter().enumerate() {
                if hit(pos, *area) {
                    if let Some(project) = self.find_results.get(i).cloned() {
                        if project.is_favorite {
                            self.remove_favorite(project.id);
                        } else {
                            self.add_favorite(&project);
                        }
                    }
                    return;
                }
            }
            for (i, area) in self.find_result_areas.iter().enumerate() {
                if hit(pos, *area) {
                    self.find_selected = i;
                    if let Some(project) = self.find_results.get(i).cloned() {
                        self.add_favorite(&project);
                        self.selected_project = self
                            .projects
                            .iter()
                            .position(|p| p.id == project.id)
                            .unwrap_or(0);
                        self.find_modal_open = false;
                        self.load_merge_requests();
                    }
                    return;
                }
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

                if !self.projects.is_empty() {
                    self.load_merge_requests();
                    self.load_pipelines();
                }
            }
            AppMessage::TokenValidated(Err(e)) => {
                self.is_validating = false;
                self.auth_error = Some(e.to_string());
                self.screen = AppScreen::AuthModal;
            }
            AppMessage::ProjectsLoaded(_) => {}
            AppMessage::SearchResults(Ok(infos)) => {
                let favorites = &self.config.gitlab.favorites;
                self.find_results = infos
                    .into_iter()
                    .map(|p| Project {
                        is_favorite: favorites.iter().any(|f| f.id == p.id),
                        id: p.id,
                        name: p.name,
                        path_with_namespace: p.path_with_namespace,
                    })
                    .collect();
                self.find_loading = false;
                self.find_selected = 0;
            }
            AppMessage::SearchResults(Err(_)) => {
                self.find_loading = false;
            }
            AppMessage::MergeRequestsLoaded(Ok(mrs)) => {
                self.merge_requests = mrs;
                self.mrs_loading = false;
            }
            AppMessage::MergeRequestsLoaded(Err(_)) => {
                self.mrs_loading = false;
            }
            AppMessage::PipelinesLoaded(Ok(pipelines)) => {
                self.pipelines = pipelines;
                self.pipelines_loading = false;
            }
            AppMessage::PipelinesLoaded(Err(_)) => {
                self.pipelines_loading = false;
            }
            AppMessage::Tick => {
                if self.screen == AppScreen::Main
                    && self.active_tab == Tab::Pipelines
                    && self.autoreload_pipelines
                    && !self.projects.is_empty()
                {
                    self.load_pipelines();
                }
            }
        }
    }

    fn search_projects(&self) {
        if self.find_input.is_empty() {
            return;
        }
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        let query = self.find_input.clone();

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, String::new());
            let result = provider.search_projects(&query).await;
            let _ = tx.send(AppMessage::SearchResults(result));
        });
    }

    pub fn load_pipelines(&mut self) {
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        self.pipelines_loading = true;
        self.last_pipeline_refresh = Some(Instant::now());

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project.path_with_namespace);
            let result = provider.list_pipelines(Default::default()).await;
            let _ = tx.send(AppMessage::PipelinesLoaded(result));
        });
    }

    pub fn logout(&mut self) {
        self.config.auth.token = None;
        let _ = config::save_config(&self.config);
        self.token_input.clear();
        self.current_user = None;
        self.merge_requests.clear();
        self.pipelines.clear();
        self.screen = AppScreen::AuthModal;
        self.auth_error = None;
        self.token_source_warning = None;
    }

    fn load_merge_requests(&mut self) {
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        self.mrs_loading = true;
        self.merge_requests.clear();

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project.path_with_namespace);
            let params = ListMrParams { state: MrState::All, page: 1, per_page: 50 };
            let result = provider.list_merge_requests(params).await;
            let _ = tx.send(AppMessage::MergeRequestsLoaded(result));
        });
    }

    fn add_favorite(&mut self, project: &Project) {
        use crate::config::types::FavoriteProject;
        let already = self.config.gitlab.favorites.iter().any(|f| f.id == project.id);
        if already {
            return;
        }
        self.config.gitlab.favorites.push(FavoriteProject {
            id: project.id,
            name: project.name.clone(),
            path_with_namespace: project.path_with_namespace.clone(),
        });
        let _ = config::save_config(&self.config);

        if !self.projects.iter().any(|p| p.id == project.id) {
            self.projects.push(Project {
                id: project.id,
                name: project.name.clone(),
                path_with_namespace: project.path_with_namespace.clone(),
                is_favorite: true,
            });
        }

        for p in &mut self.find_results {
            if p.id == project.id {
                p.is_favorite = true;
            }
        }
    }

    fn remove_favorite(&mut self, project_id: u64) {
        self.config.gitlab.favorites.retain(|f| f.id != project_id);
        let _ = config::save_config(&self.config);

        self.projects.retain(|p| p.id != project_id);
        if self.selected_project >= self.projects.len() && !self.projects.is_empty() {
            self.selected_project = self.projects.len() - 1;
        }

        for p in &mut self.find_results {
            if p.id == project_id {
                p.is_favorite = false;
            }
        }
    }
}

fn hit(pos: (u16, u16), area: ratatui::prelude::Rect) -> bool {
    pos.0 >= area.x
        && pos.0 < area.x + area.width
        && pos.1 >= area.y
        && pos.1 < area.y + area.height
}
