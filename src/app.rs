use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;
use tokio::sync::mpsc;

use crate::auth::{self, TokenSource};
use crate::config;
use crate::config::types::AppConfig;
use std::collections::HashMap;
use crate::provider::types::{Commit, ListMrParams, MergeRequest, MrChange, MrPipeline, MrState, PipelineEnrichedData, Pipeline, ProjectInfo, User};
use crate::provider::gitlab::GitLabProvider;
use crate::provider::{Provider, ProviderError};
use crate::table_nav::TableNav;
use crate::theme::{self, Theme};
use crate::ui::click_regions::ClickRegions;

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Splash,
    Main,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Tab {
    #[default]
    MergeRequests,
    Pipelines,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MrDetailTab {
    #[default]
    Overview,
    Commits,
    Pipelines,
    Changes,
}

impl MrDetailTab {
    pub const ALL: &[MrDetailTab] = &[
        MrDetailTab::Overview,
        MrDetailTab::Commits,
        MrDetailTab::Pipelines,
        MrDetailTab::Changes,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            MrDetailTab::Overview => " overview ",
            MrDetailTab::Commits => " commits ",
            MrDetailTab::Pipelines => " pipelines ",
            MrDetailTab::Changes => " changes ",
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusLayer {
    Main,
    MrDetail,
    FindModal,
    SettingsModal,
    AuthModal,
    ProjectDropdown,
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
    MrDetailLoaded(Result<MergeRequest, ProviderError>),
    MrCommitsLoaded(Result<Vec<Commit>, ProviderError>),
    MrChangesLoaded(Result<Vec<MrChange>, ProviderError>),
    MrPipelinesLoaded(Result<Vec<MrPipeline>, ProviderError>),
    PipelineEnrichedLoaded(u64, Result<PipelineEnrichedData, ProviderError>),
    PipelineEnrichedRefreshed(u64, Result<PipelineEnrichedData, ProviderError>),
    PipelineDetailEnrichedLoaded(Result<PipelineEnrichedData, ProviderError>),
    PipelineDetailRefreshed(Result<PipelineEnrichedData, ProviderError>),
    JobLogLoaded(Result<String, ProviderError>),
    Tick,
    SpinnerTick,
}

pub struct App {
    pub screen: AppScreen,
    pub should_quit: bool,

    pub token_input: String,
    pub token_source_warning: Option<String>,
    pub auth_error: Option<String>,
    pub is_validating: bool,
    pub auth_open: bool,

    pub provider: Option<Box<dyn Provider>>,
    pub current_user: Option<User>,
    pub merge_requests: Vec<MergeRequest>,
    pub mrs_loading: bool,
    pub pipelines: Vec<Pipeline>,
    pub pipelines_loading: bool,
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

    // Settings modal
    pub settings_open: bool,
    pub settings_selected: usize,
    pub settings_tab_areas: Vec<ratatui::prelude::Rect>,
    pub settings_theme_areas: Vec<ratatui::prelude::Rect>,
    pub settings_config_field: usize,
    pub settings_refresh_dec_area: Option<ratatui::prelude::Rect>,
    pub settings_refresh_inc_area: Option<ratatui::prelude::Rect>,
    pub settings_header_soft_area: Option<ratatui::prelude::Rect>,
    pub settings_header_hard_area: Option<ratatui::prelude::Rect>,
    pub settings_apply_area: Option<ratatui::prelude::Rect>,
    pub settings_close_area: Option<ratatui::prelude::Rect>,
    pub theme_selected: usize,
    pub theme_confirmed: usize,
    pub theme: &'static Theme,

    // Header background: true = soft (theme bg), false = hard (theme header_bg)
    pub header_bg_soft: bool,
    pub header_bg_confirmed: bool,

    // Table navigation (separate state per tab)
    pub mr_nav: TableNav,
    pub pipeline_nav: TableNav,

    // MR detail state
    pub mr_detail_open: bool,
    pub mr_detail_height: u16,
    pub mr_detail_dragging: bool,
    pub mr_detail_tab: MrDetailTab,
    pub mr_detail_loading: bool,
    pub mr_detail_full: Option<MergeRequest>,
    pub mr_desc_scroll: u16,
    pub mr_commits: Vec<Commit>,
    pub mr_commits_loading: bool,
    pub mr_commits_scroll: u16,
    pub mr_changes: Vec<MrChange>,
    pub mr_changes_loading: bool,
    pub mr_changes_scroll: u16,
    pub mr_pipelines: Vec<MrPipeline>,
    pub mr_pipelines_loading: bool,
    pub mr_pipelines_scroll: u16,
    pub mr_pipeline_enriched: HashMap<u64, PipelineEnrichedData>,

    // Pipeline detail state
    pub pipeline_detail_open: bool,
    pub pipeline_detail_height: u16,
    pub pipeline_detail_enriched: Option<PipelineEnrichedData>,
    pub pipeline_detail_loading: bool,
    pub pipeline_detail_scroll: u16,

    // Job log panel
    pub job_log_open: bool,
    pub job_log_loading: bool,
    pub job_log: String,
    pub job_log_scroll: u16,
    pub selected_job_id: Option<u64>,
    pub selected_job_name: Option<String>,
    pub job_log_is_refresh: bool,

    // Auto-refresh
    pub tick_frame: u8,
    pub settings_refresh_interval: u64,

    // Click areas (grouped by region)
    pub click_regions: ClickRegions,
}

impl App {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let config = config::load_config().unwrap_or_default();
        let http_client = reqwest::Client::new();
        let refresh_interval = config.ui.refresh_interval_secs.unwrap_or(10);


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

        // Default to "soft" (theme bg) unless explicitly set to "hard".
        let header_bg_soft = config.ui.header_bg.as_deref() != Some("hard");

        let mut app = Self {
            screen: AppScreen::Splash,
            should_quit: false,
            token_input: String::new(),
            token_source_warning: None,
            auth_error: None,
            is_validating: false,
            auth_open: false,
            provider: None,
            current_user: None,
            merge_requests: Vec::new(),
            mrs_loading: false,
            pipelines: Vec::new(),
            pipelines_loading: false,
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
            settings_tab_areas: Vec::new(),
            settings_theme_areas: Vec::new(),
            settings_config_field: 0,
            settings_refresh_dec_area: None,
            settings_refresh_inc_area: None,
            settings_header_soft_area: None,
            settings_header_hard_area: None,
            settings_apply_area: None,
            settings_close_area: None,
            theme_selected,
            theme_confirmed: theme_selected,
            theme: active_theme,
            header_bg_soft,
            header_bg_confirmed: header_bg_soft,
            mr_nav: TableNav::default(),
            pipeline_nav: TableNav::default(),
            mr_detail_open: false,
            mr_detail_height: 0,
            mr_detail_dragging: false,
            mr_detail_tab: MrDetailTab::default(),
            mr_detail_loading: false,
            mr_detail_full: None,
            mr_desc_scroll: 0,
            mr_commits: Vec::new(),
            mr_commits_loading: false,
            mr_commits_scroll: 0,
            mr_changes: Vec::new(),
            mr_changes_loading: false,
            mr_changes_scroll: 0,
            mr_pipelines: Vec::new(),
            mr_pipelines_loading: false,
            mr_pipelines_scroll: 0,
            mr_pipeline_enriched: HashMap::new(),
            pipeline_detail_open: false,
            pipeline_detail_height: 0,
            pipeline_detail_enriched: None,
            pipeline_detail_loading: false,
            pipeline_detail_scroll: 0,
            job_log_open: false,
            job_log_loading: false,
            job_log: String::new(),
            job_log_scroll: 0,
            selected_job_id: None,
            selected_job_name: None,
            job_log_is_refresh: false,
            tick_frame: 0,
            settings_refresh_interval: refresh_interval,
            click_regions: ClickRegions::default(),
        };

        app.try_auto_auth();
        app
    }

    pub fn active_layer(&self) -> FocusLayer {
        if self.project_selector_open { return FocusLayer::ProjectDropdown; }
        if self.find_modal_open { return FocusLayer::FindModal; }
        if self.settings_open { return FocusLayer::SettingsModal; }
        if self.auth_open { return FocusLayer::AuthModal; }
        if self.mr_detail_open { return FocusLayer::MrDetail; }
        FocusLayer::Main
    }

    fn filtered_mr_count(&self) -> usize {
        self.merge_requests.iter()
            .filter(|mr| self.mr_filter.matches(&mr.state))
            .count()
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
                self.screen = AppScreen::Main;
                // Not logged in: still load the selected favorite so public
                // projects show their data without a token.
                if !self.projects.is_empty() {
                    self.load_merge_requests();
                    self.load_pipelines();
                }
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
            AppScreen::Main => {
                if self.auth_open {
                    self.handle_auth_key(key);
                } else if self.find_modal_open {
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
                        KeyCode::Esc if self.mr_detail_open => {
                            self.mr_detail_open = false;
                            self.mr_detail_full = None;
                            self.mr_detail_tab = MrDetailTab::default();
                        }
                        KeyCode::Esc if self.pipeline_detail_open => {
                            self.pipeline_detail_open = false;
                        }
                        KeyCode::Esc if self.mr_nav.selected.is_some() => {
                            self.mr_nav.selected = None;
                        }
                        KeyCode::Esc if self.pipeline_nav.selected.is_some() => {
                            self.pipeline_nav.selected = None;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Char('1') => self.active_tab = Tab::MergeRequests,
                        KeyCode::Char('2') => self.active_tab = Tab::Pipelines,
                        KeyCode::Tab => {
                            self.active_tab = match self.active_tab {
                                Tab::MergeRequests => Tab::Pipelines,
                                Tab::Pipelines => Tab::MergeRequests,
                            };
                        }
                        KeyCode::Left if self.mr_detail_open => {
                            self.cycle_mr_detail_tab_back();
                        }
                        KeyCode::Right if self.mr_detail_open => {
                            self.cycle_mr_detail_tab_forward();
                        }
                        KeyCode::Left if self.active_tab == Tab::MergeRequests => {
                            self.cycle_mr_filter_back();
                        }
                        KeyCode::Right if self.active_tab == Tab::MergeRequests => {
                            self.cycle_mr_filter_forward();
                        }
                        KeyCode::Up | KeyCode::Char('k') if self.active_tab == Tab::MergeRequests => {
                            let count = self.filtered_mr_count();
                            if self.mr_nav.move_up(count) && self.mr_detail_open {
                                self.load_mr_detail();
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') if self.active_tab == Tab::MergeRequests => {
                            let count = self.filtered_mr_count();
                            if self.mr_nav.move_down(count) && self.mr_detail_open {
                                self.load_mr_detail();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') if self.active_tab == Tab::Pipelines => {
                            let count = self.pipelines.len();
                            self.pipeline_nav.move_up(count);
                        }
                        KeyCode::Down | KeyCode::Char('j') if self.active_tab == Tab::Pipelines => {
                            let count = self.pipelines.len();
                            self.pipeline_nav.move_down(count);
                        }
                        KeyCode::Enter if self.active_tab == Tab::Pipelines && self.pipeline_nav.selected.is_some() => {
                            self.open_pipeline_detail();
                        }
                        KeyCode::Enter if self.active_tab == Tab::MergeRequests && self.mr_nav.selected.is_some() => {
                            self.load_mr_detail();
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
                        KeyCode::Char(',') => {
                            self.settings_open = true;
                            self.settings_config_field = 0;
                            self.settings_refresh_interval = self.refresh_interval_secs();
                            self.header_bg_confirmed = self.header_bg_soft;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_auth_key(&mut self, key: KeyEvent) {
        if self.is_validating {
            return;
        }
        match key.code {
            KeyCode::Esc => {
                self.auth_open = false;
                self.auth_error = None;
            }
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
                    self.reset_panels();
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
        self.close_mr_detail();
    }

    fn cycle_mr_filter_back(&mut self) {
        let filters = MrFilter::ALL_FILTERS;
        let idx = filters.iter().position(|f| *f == self.mr_filter).unwrap_or(0);
        self.mr_filter = filters[(idx + filters.len() - 1) % filters.len()].clone();
        self.close_mr_detail();
    }

    fn cycle_mr_detail_tab_forward(&mut self) {
        let all = MrDetailTab::ALL;
        let idx = all.iter().position(|t| *t == self.mr_detail_tab).unwrap_or(0);
        self.mr_detail_tab = all[(idx + 1) % all.len()].clone();
    }

    fn cycle_mr_detail_tab_back(&mut self) {
        let all = MrDetailTab::ALL;
        let idx = all.iter().position(|t| *t == self.mr_detail_tab).unwrap_or(0);
        self.mr_detail_tab = all[(idx + all.len() - 1) % all.len()].clone();
    }

    fn close_mr_detail(&mut self) {
        self.mr_detail_open = false;
        self.mr_nav.reset();
        self.mr_detail_full = None;
        self.mr_detail_tab = MrDetailTab::default();
        self.job_log_open = false;
        self.selected_job_id = None;
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        const NUM_TABS: usize = 2;
        const NUM_CONFIG_FIELDS: usize = 2;
        match key.code {
            KeyCode::Esc => {
                // Cancel: revert live previews to confirmed values
                self.theme = theme::ALL_THEMES[self.theme_confirmed];
                self.theme_selected = self.theme_confirmed;
                self.header_bg_soft = self.header_bg_confirmed;
                self.settings_open = false;
            }
            // Switch section tabs with Left/Right (or Tab)
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.settings_selected = (self.settings_selected + 1) % NUM_TABS;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.settings_selected = (self.settings_selected + NUM_TABS - 1) % NUM_TABS;
            }
            KeyCode::Up | KeyCode::Char('k') if self.settings_selected == 0 => {
                if self.theme_selected > 0 {
                    self.theme_selected -= 1;
                    self.theme = theme::ALL_THEMES[self.theme_selected];
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.settings_selected == 0 => {
                if self.theme_selected < theme::ALL_THEMES.len().saturating_sub(1) {
                    self.theme_selected += 1;
                    self.theme = theme::ALL_THEMES[self.theme_selected];
                }
            }
            // Config tab: Up/Down moves between fields
            KeyCode::Up | KeyCode::Char('k') if self.settings_selected == 1 => {
                self.settings_config_field =
                    (self.settings_config_field + NUM_CONFIG_FIELDS - 1) % NUM_CONFIG_FIELDS;
            }
            KeyCode::Down | KeyCode::Char('j') if self.settings_selected == 1 => {
                self.settings_config_field =
                    (self.settings_config_field + 1) % NUM_CONFIG_FIELDS;
            }
            // Config tab: +/- adjusts the refresh timer (live preview)
            KeyCode::Char('+') | KeyCode::Char('=') if self.settings_selected == 1 => {
                if self.settings_refresh_interval < 120 { self.settings_refresh_interval += 1; }
            }
            KeyCode::Char('-') | KeyCode::Char('_') if self.settings_selected == 1 => {
                if self.settings_refresh_interval > 5 { self.settings_refresh_interval -= 1; }
            }
            // Config tab: Space toggles the header background (live preview)
            KeyCode::Char(' ') if self.settings_selected == 1 && self.settings_config_field == 1 => {
                self.header_bg_soft = !self.header_bg_soft;
            }
            KeyCode::Enter => {
                self.apply_settings();
            }
            _ => {}
        }
    }

    fn apply_settings(&mut self) {
        self.theme_confirmed = self.theme_selected;
        self.config.ui.theme = Some(self.theme.name.to_string());
        self.config.ui.refresh_interval_secs = Some(self.settings_refresh_interval);
        self.header_bg_confirmed = self.header_bg_soft;
        self.config.ui.header_bg =
            Some(if self.header_bg_soft { "soft" } else { "hard" }.to_string());
        let _ = config::save_config(&self.config);
        self.settings_open = false;
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
            self.should_quit = true;
            return;
        }

        if self.screen != AppScreen::Main {
            return;
        }

        let pos = (mouse.column, mouse.row);

        // Handle drag-to-resize (global — active regardless of layer)
        if mouse.kind == MouseEventKind::Drag(MouseButton::Left) {
            if self.mr_detail_dragging {
                let content_start = 5u16;
                let screen_height = self.click_regions.mr_detail.resize
                    .map(|a| a.y + self.mr_detail_height)
                    .unwrap_or(24);
                let new_height = screen_height.saturating_sub(pos.1);
                self.mr_detail_height = new_height.max(8).min(screen_height.saturating_sub(content_start + 4));
            }
            return;
        }

        if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
            self.mr_detail_dragging = false;
            return;
        }

        if mouse.kind == MouseEventKind::ScrollDown {
            if !self.find_modal_open && !self.settings_open && !self.auth_open {
                if self.mr_detail_open && self.active_tab == Tab::MergeRequests {
                    if let Some(bounds) = self.click_regions.mr_detail.bounds {
                        if hit(pos, bounds) {
                            match self.mr_detail_tab {
                                MrDetailTab::Overview => self.mr_desc_scroll = self.mr_desc_scroll.saturating_add(3),
                                MrDetailTab::Commits => self.mr_commits_scroll = self.mr_commits_scroll.saturating_add(3),
                                MrDetailTab::Changes => self.mr_changes_scroll = self.mr_changes_scroll.saturating_add(3),
                                MrDetailTab::Pipelines => {
                                    let mid = bounds.x + bounds.width * 2 / 5;
                                    if self.job_log_open && pos.0 > mid {
                                        self.job_log_scroll = self.job_log_scroll.saturating_add(3);
                                    } else {
                                        self.mr_pipelines_scroll = self.mr_pipelines_scroll.saturating_add(3);
                                    }
                                }
                            }
                            return;
                        }
                    }
                }
                if self.pipeline_detail_open && self.active_tab == Tab::Pipelines {
                    if let Some(bounds) = self.click_regions.pipeline_detail.bounds {
                        if hit(pos, bounds) {
                            if self.job_log_open && pos.0 > bounds.x + bounds.width / 2 {
                                self.job_log_scroll = self.job_log_scroll.saturating_add(3);
                            } else {
                                self.pipeline_detail_scroll = self.pipeline_detail_scroll.saturating_add(3);
                            }
                            return;
                        }
                    }
                }
                match self.active_tab {
                    Tab::MergeRequests => self.mr_nav.scroll_down(self.filtered_mr_count()),
                    Tab::Pipelines => self.pipeline_nav.scroll_down(self.pipelines.len()),
                }
            }
            return;
        }

        if mouse.kind == MouseEventKind::ScrollUp {
            if !self.find_modal_open && !self.settings_open && !self.auth_open {
                if self.mr_detail_open && self.active_tab == Tab::MergeRequests {
                    if let Some(bounds) = self.click_regions.mr_detail.bounds {
                        if hit(pos, bounds) {
                            match self.mr_detail_tab {
                                MrDetailTab::Overview => self.mr_desc_scroll = self.mr_desc_scroll.saturating_sub(3),
                                MrDetailTab::Commits => self.mr_commits_scroll = self.mr_commits_scroll.saturating_sub(3),
                                MrDetailTab::Changes => self.mr_changes_scroll = self.mr_changes_scroll.saturating_sub(3),
                                MrDetailTab::Pipelines => {
                                    let mid = bounds.x + bounds.width * 2 / 5;
                                    if self.job_log_open && pos.0 > mid {
                                        self.job_log_scroll = self.job_log_scroll.saturating_sub(3);
                                    } else {
                                        self.mr_pipelines_scroll = self.mr_pipelines_scroll.saturating_sub(3);
                                    }
                                }
                            }
                            return;
                        }
                    }
                }
                if self.pipeline_detail_open && self.active_tab == Tab::Pipelines {
                    if let Some(bounds) = self.click_regions.pipeline_detail.bounds {
                        if hit(pos, bounds) {
                            if self.job_log_open && pos.0 > bounds.x + bounds.width / 2 {
                                self.job_log_scroll = self.job_log_scroll.saturating_sub(3);
                            } else {
                                self.pipeline_detail_scroll = self.pipeline_detail_scroll.saturating_sub(3);
                            }
                            return;
                        }
                    }
                }
                match self.active_tab {
                    Tab::MergeRequests => self.mr_nav.scroll_up(),
                    Tab::Pipelines => self.pipeline_nav.scroll_up(),
                }
            }
            return;
        }

        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }

        match self.active_layer() {
            FocusLayer::ProjectDropdown => self.handle_mouse_dropdown(pos),
            FocusLayer::FindModal       => self.handle_mouse_find(pos),
            FocusLayer::SettingsModal   => self.handle_mouse_settings(pos),
            FocusLayer::AuthModal       => {} // consume clicks; modal is keyboard-driven
            FocusLayer::MrDetail        => self.handle_mouse_detail(pos),
            FocusLayer::Main            => self.handle_mouse_main(pos),
        }
    }

    fn handle_mouse_dropdown(&mut self, pos: (u16, u16)) {
        for (i, area) in self.click_regions.project_dropdown.items.iter().enumerate() {
            if hit(pos, *area) {
                self.selected_project = i;
                self.project_selector_open = false;
                self.reset_panels();
                self.load_merge_requests();
                self.load_pipelines();
                return;
            }
        }
        self.project_selector_open = false;
    }

    fn handle_mouse_find(&mut self, pos: (u16, u16)) {
        for (i, area) in self.click_regions.find_modal.star_areas.iter().enumerate() {
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
        for (i, area) in self.click_regions.find_modal.result_areas.iter().enumerate() {
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
                    self.reset_panels();
                    self.load_merge_requests();
                    self.load_pipelines();
                }
                return;
            }
        }
        // Click outside modal bounds — consume (don't pass through)
    }

    fn handle_mouse_settings(&mut self, pos: (u16, u16)) {
        // Section tabs
        for (i, area) in self.settings_tab_areas.iter().enumerate() {
            if hit(pos, *area) {
                self.settings_selected = i;
                return;
            }
        }

        // Footer: apply / close
        if let Some(area) = self.settings_apply_area {
            if hit(pos, area) {
                self.apply_settings();
                return;
            }
        }
        if let Some(area) = self.settings_close_area {
            if hit(pos, area) {
                self.theme = theme::ALL_THEMES[self.theme_confirmed];
                self.theme_selected = self.theme_confirmed;
                self.header_bg_soft = self.header_bg_confirmed;
                self.settings_open = false;
                return;
            }
        }

        // Themes tab: click a theme to select + live-preview it
        if self.settings_selected == 0 {
            for (i, area) in self.settings_theme_areas.iter().enumerate() {
                if hit(pos, *area) {
                    self.theme_selected = i;
                    self.theme = theme::ALL_THEMES[i];
                    return;
                }
            }
        }

        // Config tab: refresh +/- and header background soft/hard (live preview)
        if self.settings_selected == 1 {
            if let Some(area) = self.settings_refresh_dec_area {
                if hit(pos, area) {
                    self.settings_config_field = 0;
                    if self.settings_refresh_interval > 5 { self.settings_refresh_interval -= 1; }
                    return;
                }
            }
            if let Some(area) = self.settings_refresh_inc_area {
                if hit(pos, area) {
                    self.settings_config_field = 0;
                    if self.settings_refresh_interval < 120 { self.settings_refresh_interval += 1; }
                    return;
                }
            }
            if let Some(area) = self.settings_header_soft_area {
                if hit(pos, area) {
                    self.settings_config_field = 1;
                    self.header_bg_soft = true;
                    return;
                }
            }
            if let Some(area) = self.settings_header_hard_area {
                if hit(pos, area) {
                    self.settings_config_field = 1;
                    self.header_bg_soft = false;
                    return;
                }
            }
        }
        // Consume all other clicks (block pass-through)
    }

    fn handle_mouse_detail(&mut self, pos: (u16, u16)) {
        // Detail-specific areas
        for (i, area) in self.click_regions.mr_detail.tab_areas.iter().enumerate() {
            if hit(pos, *area) {
                if let Some(tab) = MrDetailTab::ALL.get(i) {
                    self.mr_detail_tab = tab.clone();
                }
                return;
            }
        }

        if let Some(area) = self.click_regions.mr_detail.close {
            if hit(pos, area) {
                self.mr_detail_open = false;
                self.mr_detail_full = None;
                self.mr_detail_tab = MrDetailTab::default();
                return;
            }
        }

        if let Some(area) = self.click_regions.mr_detail.resize {
            if hit(pos, area) {
                self.mr_detail_dragging = true;
                return;
            }
        }

        // Job log close button
        if let Some(area) = self.click_regions.mr_detail.job_log_close {
            if hit(pos, area) {
                self.job_log_open = false;
                self.selected_job_id = None;
                return;
            }
        }

        // Job click areas (MR Pipelines tab)
        if self.mr_detail_tab == MrDetailTab::Pipelines {
            let job_areas: Vec<(Rect, u64, String)> = self.click_regions.mr_detail.job_areas.clone();
            for (area, job_id, job_name) in job_areas {
                if hit(pos, area) {
                    if self.selected_job_id == Some(job_id) && self.job_log_open {
                        self.job_log_open = false;
                        self.selected_job_id = None;
                    } else {
                        self.load_job_log(job_id, job_name);
                    }
                    return;
                }
            }
        }

        // If click is within the detail panel bounds, consume it
        if let Some(bounds) = self.click_regions.mr_detail.bounds {
            if hit(pos, bounds) {
                return;
            }
        }

        // Click is above the detail panel — fall through to shared areas
        self.handle_mouse_shared(pos);
    }

    fn handle_mouse_main(&mut self, pos: (u16, u16)) {
        self.handle_mouse_shared(pos);
    }

    fn handle_mouse_shared(&mut self, pos: (u16, u16)) {
        if let Some(area) = self.click_regions.header.project_selector {
            if hit(pos, area) {
                self.project_selector_open = true;
                return;
            }
        }

        if let Some(area) = self.click_regions.header.logout_link {
            if hit(pos, area) {
                self.logout();
                return;
            }
        }

        if let Some(area) = self.click_regions.header.login_link {
            if hit(pos, area) {
                self.auth_open = true;
                self.token_input.clear();
                self.auth_error = None;
                return;
            }
        }

        if let Some(area) = self.click_regions.header.settings_link {
            if hit(pos, area) {
                self.settings_open = true;
                self.settings_config_field = 0;
                self.settings_refresh_interval = self.refresh_interval_secs();
                self.header_bg_confirmed = self.header_bg_soft;
                return;
            }
        }

        if let Some(area) = self.click_regions.header.tab_mr {
            if hit(pos, area) {
                self.active_tab = Tab::MergeRequests;
                return;
            }
        }

        if let Some(area) = self.click_regions.header.tab_pipelines {
            if hit(pos, area) {
                self.active_tab = Tab::Pipelines;
                return;
            }
        }

        if let Some(area) = self.click_regions.header.find_link {
            if hit(pos, area) {
                self.find_modal_open = true;
                self.find_input.clear();
                self.find_results.clear();
                self.find_selected = 0;
                return;
            }
        }



        for (i, area) in self.click_regions.main.mr_filter_areas.iter().enumerate() {
            if hit(pos, *area) {
                if let Some(f) = MrFilter::ALL_FILTERS.get(i) {
                    self.mr_filter = f.clone();
                    self.close_mr_detail();
                }
                return;
            }
        }

        if self.active_tab == Tab::MergeRequests {
            for (i, area) in self.click_regions.main.mr_row_areas.iter().enumerate() {
                if hit(pos, *area) {
                    let actual_index = self.mr_nav.offset + i;
                    if self.mr_nav.selected == Some(actual_index) && self.mr_detail_open {
                        self.mr_detail_open = false;
                        self.mr_detail_full = None;
                        self.mr_detail_tab = MrDetailTab::default();
                    } else {
                        self.mr_nav.selected = Some(actual_index);
                        self.load_mr_detail();
                    }
                    return;
                }
            }
        }

        if self.active_tab == Tab::Pipelines {
            // Job log close button
            if let Some(area) = self.click_regions.pipeline_detail.job_log_close {
                if hit(pos, area) {
                    self.job_log_open = false;
                    self.selected_job_id = None;
                    return;
                }
            }

            // Pipeline detail close button
            if let Some(area) = self.click_regions.pipeline_detail.close {
                if hit(pos, area) {
                    self.pipeline_detail_open = false;
                    self.job_log_open = false;
                    return;
                }
            }

            // Job click areas
            if self.pipeline_detail_open {
                let job_areas: Vec<(Rect, u64, String)> = self.click_regions.pipeline_detail.job_areas.clone();
                for (area, job_id, job_name) in job_areas {
                    if hit(pos, area) {
                        if self.selected_job_id == Some(job_id) && self.job_log_open {
                            self.job_log_open = false;
                            self.selected_job_id = None;
                        } else {
                            self.load_job_log(job_id, job_name);
                        }
                        return;
                    }
                }
            }

            // Pipeline detail bounds — consume click
            if self.pipeline_detail_open {
                if let Some(bounds) = self.click_regions.pipeline_detail.bounds {
                    if hit(pos, bounds) {
                        return;
                    }
                }
            }

            for (i, area) in self.click_regions.main.pipeline_row_areas.iter().enumerate() {
                if hit(pos, *area) {
                    let actual_index = self.pipeline_nav.offset + i;
                    if self.pipeline_nav.selected == Some(actual_index) && self.pipeline_detail_open {
                        self.pipeline_detail_open = false;
                    } else {
                        self.pipeline_nav.selected = Some(actual_index);
                        self.open_pipeline_detail();
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
                self.auth_open = false;
                self.auth_error = None;
                self.token_source_warning = None;

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
                self.screen = AppScreen::Main;
                self.auth_open = true;
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
            AppMessage::MrDetailLoaded(Ok(mr)) => {
                self.mr_detail_full = Some(mr);
                self.mr_detail_loading = false;
            }
            AppMessage::MrDetailLoaded(Err(_)) => {
                self.mr_detail_loading = false;
            }
            AppMessage::MrCommitsLoaded(Ok(commits)) => {
                self.mr_commits = commits;
                self.mr_commits_loading = false;
            }
            AppMessage::MrCommitsLoaded(Err(_)) => {
                self.mr_commits_loading = false;
            }
            AppMessage::MrChangesLoaded(Ok(changes)) => {
                self.mr_changes = changes;
                self.mr_changes_loading = false;
            }
            AppMessage::MrChangesLoaded(Err(_)) => {
                self.mr_changes_loading = false;
            }
            AppMessage::MrPipelinesLoaded(Ok(pipelines)) => {
                self.mr_pipelines = pipelines;
                self.mr_pipelines_loading = false;
                // Load enriched data (duration, user, stages) for each pipeline
                for pipeline in self.mr_pipelines.iter().take(5) {
                    let pipeline_id = pipeline.id;
                    let tx = self.message_tx.clone();
                    let client = self.http_client.clone();
                    let token = self.token_input.clone();
                    let base_url = self.config.gitlab.base_url_or_default().to_string();
                    let project_path = self.projects.get(self.selected_project)
                        .map(|p| p.path_with_namespace.clone())
                        .unwrap_or_default();
                    tokio::spawn(async move {
                        let provider = GitLabProvider::new(client, token, base_url, project_path);
                        let result = provider.get_pipeline_enriched(pipeline_id).await;
                        let _ = tx.send(AppMessage::PipelineEnrichedLoaded(pipeline_id, result));
                    });
                }
            }
            AppMessage::MrPipelinesLoaded(Err(_)) => {
                self.mr_pipelines_loading = false;
            }
            AppMessage::PipelineEnrichedLoaded(pipeline_id, Ok(data)) => {
                self.mr_pipeline_enriched.insert(pipeline_id, data);
            }
            AppMessage::PipelineEnrichedLoaded(_, Err(_)) => {}
            AppMessage::PipelineEnrichedRefreshed(pipeline_id, Ok(data)) => {
                // Update status on the mr_pipelines list entry too
                if let Some(p) = self.mr_pipelines.iter_mut().find(|p| p.id == pipeline_id) {
                    p.status = data.status.clone();
                }
                self.mr_pipeline_enriched.insert(pipeline_id, data);
            }
            AppMessage::PipelineEnrichedRefreshed(_, Err(_)) => {}
            AppMessage::PipelineDetailEnrichedLoaded(Ok(data)) => {
                if let Some(idx) = self.pipeline_nav.selected {
                    if let Some(p) = self.pipelines.get_mut(idx) {
                        p.status = data.status.clone();
                    }
                }
                self.pipeline_detail_enriched = Some(data);
                self.pipeline_detail_loading = false;
            }
            AppMessage::PipelineDetailRefreshed(Ok(data)) => {
                if let Some(idx) = self.pipeline_nav.selected {
                    if let Some(p) = self.pipelines.get_mut(idx) {
                        p.status = data.status.clone();
                    }
                }
                self.pipeline_detail_enriched = Some(data);
            }
            AppMessage::PipelineDetailRefreshed(Err(_)) => {}
            AppMessage::PipelineDetailEnrichedLoaded(Err(_)) => {
                self.pipeline_detail_loading = false;
            }
            AppMessage::JobLogLoaded(Ok(log)) => {
                let is_refresh = self.job_log_is_refresh;
                self.job_log = log;
                self.job_log_loading = false;
                self.job_log_is_refresh = false;
                if is_refresh {
                    let total_lines = self.job_log.lines().count() as u16;
                    self.job_log_scroll = total_lines.saturating_sub(1);
                }
            }
            AppMessage::JobLogLoaded(Err(_)) => {
                self.job_log = "Failed to load log.".to_string();
                self.job_log_loading = false;
            }
            AppMessage::SpinnerTick => {
                self.tick_frame = self.tick_frame.wrapping_add(1);
            }
            AppMessage::Tick => {
                if self.pipeline_detail_open && self.has_running_jobs() {
                    self.refresh_pipeline_detail();
                }
                if !self.mr_pipeline_enriched.is_empty() && self.has_running_jobs() {
                    self.refresh_mr_pipelines();
                }
                if self.job_log_open && !self.job_log_loading {
                    self.refresh_job_log();
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

    pub fn open_pipeline_detail(&mut self) {
        let pipeline_id = match self.pipeline_nav.selected.and_then(|i| self.pipelines.get(i)) {
            Some(p) => p.id,
            None => return,
        };
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        self.pipeline_detail_open = true;
        self.pipeline_detail_enriched = None;
        self.pipeline_detail_loading = true;
        self.pipeline_detail_scroll = 0;
        self.job_log_open = false;
        self.selected_job_id = None;

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project.path_with_namespace);
            let result = provider.get_pipeline_enriched(pipeline_id).await;
            let _ = tx.send(AppMessage::PipelineDetailEnrichedLoaded(result));
        });
    }

    pub fn load_job_log(&mut self, job_id: u64, job_name: String) {
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        self.selected_job_id = Some(job_id);
        self.selected_job_name = Some(job_name);
        self.job_log_open = true;
        self.job_log_loading = true;
        self.job_log.clear();
        self.job_log_scroll = 0;

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project.path_with_namespace);
            let result = provider.get_job_log(job_id).await;
            let _ = tx.send(AppMessage::JobLogLoaded(result));
        });
    }

    pub fn refresh_pipeline_detail(&mut self) {
        let pipeline_id = match self.pipeline_nav.selected.and_then(|i| self.pipelines.get(i)) {
            Some(p) => p.id,
            None => return,
        };
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project.path_with_namespace);
            let result = provider.get_pipeline_enriched(pipeline_id).await;
            let _ = tx.send(AppMessage::PipelineDetailRefreshed(result));
        });
    }

    pub fn refresh_job_log(&mut self) {
        let job_id = match self.selected_job_id {
            Some(id) => id,
            None => return,
        };
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        self.job_log_is_refresh = true;

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let token = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project.path_with_namespace);
            let result = provider.get_job_log(job_id).await;
            let _ = tx.send(AppMessage::JobLogLoaded(result));
        });
    }

    pub fn refresh_mr_pipelines(&mut self) {
        let project_path = match self.projects.get(self.selected_project) {
            Some(p) => p.path_with_namespace.clone(),
            None => return,
        };
        let running_ids: Vec<u64> = self.mr_pipeline_enriched
            .iter()
            .filter(|(_, data)| data.stages.iter().any(|s| {
                s.jobs.iter().any(|j| j.status == "running" || j.sub_jobs.iter().any(|sj| sj.status == "running"))
            }))
            .map(|(id, _)| *id)
            .collect();
        for pipeline_id in running_ids {
            let tx = self.message_tx.clone();
            let client = self.http_client.clone();
            let token = self.token_input.clone();
            let base_url = self.config.gitlab.base_url_or_default().to_string();
            let path = project_path.clone();
            tokio::spawn(async move {
                let provider = GitLabProvider::new(client, token, base_url, path);
                let result = provider.get_pipeline_enriched(pipeline_id).await;
                let _ = tx.send(AppMessage::PipelineEnrichedRefreshed(pipeline_id, result));
            });
        }
    }

    pub fn refresh_interval_secs(&self) -> u64 {
        self.config.ui.refresh_interval_secs.unwrap_or(10)
    }

    pub fn has_running_jobs(&self) -> bool {
        let in_detail = self.pipeline_detail_enriched.as_ref().map_or(false, |data| {
            data.stages.iter().any(|stage| {
                stage.jobs.iter().any(|job| job.status == "running" || job.sub_jobs.iter().any(|s| s.status == "running"))
            })
        });
        let in_mr = self.mr_pipeline_enriched.values().any(|data| {
            data.stages.iter().any(|stage| {
                stage.jobs.iter().any(|job| job.status == "running" || job.sub_jobs.iter().any(|s| s.status == "running"))
            })
        });
        in_detail || in_mr
    }

    pub fn load_pipelines(&mut self) {
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        self.pipelines_loading = true;
        self.pipeline_nav.reset();
        self.pipeline_detail_open = false;

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

    pub fn reset_panels(&mut self) {
        self.mr_detail_open = false;
        self.mr_detail_full = None;
        self.mr_detail_tab = MrDetailTab::default();
        self.mr_commits.clear();
        self.mr_changes.clear();
        self.mr_pipelines.clear();
        self.mr_pipeline_enriched.clear();
        self.mr_nav.reset();
        self.pipeline_detail_open = false;
        self.pipeline_detail_enriched = None;
        self.pipeline_detail_loading = false;
        self.pipeline_detail_scroll = 0;
        self.job_log_open = false;
        self.job_log.clear();
        self.job_log_loading = false;
        self.job_log_scroll = 0;
        self.selected_job_id = None;
        self.selected_job_name = None;
        self.pipeline_nav.reset();
    }

    pub fn logout(&mut self) {
        self.config.auth.token = None;
        let _ = config::save_config(&self.config);
        self.token_input.clear();
        self.current_user = None;
        self.merge_requests.clear();
        self.pipelines.clear();
        self.screen = AppScreen::Main;
        self.auth_open = false;
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
        self.mr_nav.reset();

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

    pub fn load_mr_detail(&mut self) {
        let filtered: Vec<&MergeRequest> = self
            .merge_requests
            .iter()
            .filter(|mr| self.mr_filter.matches(&mr.state))
            .collect();

        let mr_iid = match self.mr_nav.selected.and_then(|i| filtered.get(i)) {
            Some(mr) => mr.iid,
            None => return,
        };

        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };

        self.mr_detail_open = true;
        self.mr_detail_loading = true;
        self.mr_detail_full = None;
        self.mr_desc_scroll = 0;
        self.job_log_open = false;
        self.selected_job_id = None;
        self.mr_commits_loading = true;
        self.mr_commits.clear();
        self.mr_commits_scroll = 0;
        self.mr_changes_loading = true;
        self.mr_changes.clear();
        self.mr_changes_scroll = 0;
        self.mr_pipelines_loading = true;
        self.mr_pipelines.clear();
        self.mr_pipelines_scroll = 0;
        self.mr_pipeline_enriched.clear();

        let tx = self.message_tx.clone();
        let tx2 = self.message_tx.clone();
        let tx3 = self.message_tx.clone();
        let tx4 = self.message_tx.clone();
        let client = self.http_client.clone();
        let client2 = self.http_client.clone();
        let client3 = self.http_client.clone();
        let client4 = self.http_client.clone();
        let token = self.token_input.clone();
        let token2 = self.token_input.clone();
        let token3 = self.token_input.clone();
        let token4 = self.token_input.clone();
        let base_url = self.config.gitlab.base_url_or_default().to_string();
        let base_url2 = base_url.clone();
        let base_url3 = base_url.clone();
        let base_url4 = base_url.clone();
        let project_path = project.path_with_namespace.clone();
        let project_path2 = project_path.clone();
        let project_path3 = project_path.clone();
        let project_path4 = project_path.clone();

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client, token, base_url, project_path);
            let result = provider.get_merge_request(mr_iid).await;
            let _ = tx.send(AppMessage::MrDetailLoaded(result));
        });

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client2, token2, base_url2, project_path2);
            let result = provider.list_mr_commits(mr_iid).await;
            let _ = tx2.send(AppMessage::MrCommitsLoaded(result));
        });

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client3, token3, base_url3, project_path3);
            let result = provider.list_mr_pipelines(mr_iid).await;
            let _ = tx3.send(AppMessage::MrPipelinesLoaded(result));
        });

        tokio::spawn(async move {
            let provider = GitLabProvider::new(client4, token4, base_url4, project_path4);
            let result = provider.list_mr_changes(mr_iid).await;
            let _ = tx4.send(AppMessage::MrChangesLoaded(result));
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
