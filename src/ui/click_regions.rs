use ratatui::prelude::Rect;

#[derive(Default)]
pub struct ClickRegions {
    pub header: HeaderRegion,
    pub main: MainRegion,
    pub mr_detail: MrDetailRegion,
    pub pipeline_detail: PipelineDetailRegion,
    pub find_modal: FindModalRegion,
    pub project_dropdown: ProjectDropdownRegion,
}

impl ClickRegions {
    pub fn clear(&mut self) {
        self.header = HeaderRegion::default();
        self.main = MainRegion::default();
        self.mr_detail = MrDetailRegion::default();
        self.pipeline_detail = PipelineDetailRegion::default();
        self.find_modal = FindModalRegion::default();
        self.project_dropdown = ProjectDropdownRegion::default();
    }
}

#[derive(Default)]
pub struct HeaderRegion {
    pub project_selector: Option<Rect>,
    pub find_link: Option<Rect>,
    pub logout_link: Option<Rect>,
    pub settings_link: Option<Rect>,
    pub tab_mr: Option<Rect>,
    pub tab_pipelines: Option<Rect>,
}

#[derive(Default)]
pub struct MainRegion {
    pub mr_filter_areas: Vec<Rect>,
    pub mr_row_areas: Vec<Rect>,
    pub pipeline_row_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct MrDetailRegion {
    pub bounds: Option<Rect>,
    pub close: Option<Rect>,
    pub resize: Option<Rect>,
    pub tab_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct PipelineDetailRegion {
    pub bounds: Option<Rect>,
    pub close: Option<Rect>,
    pub job_areas: Vec<(Rect, u64)>,  // (area, job_id)
}

#[derive(Default)]
pub struct FindModalRegion {
    pub bounds: Option<Rect>,
    pub result_areas: Vec<Rect>,
    pub star_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct ProjectDropdownRegion {
    pub bounds: Option<Rect>,
    pub items: Vec<Rect>,
}
