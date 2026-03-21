use anyhow::Result;
use ratatui::{layout::Rect, style::Color, widgets::ListState};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Theme {
    pub focused: Color,
    pub unfocused: Color,
    pub selected_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            focused: Color::Magenta,
            unfocused: Color::White,
            selected_bg: Color::DarkGray,
        }
    }
}

use crate::models::{Job, JobList};
use crate::slurm::{SlurmCommands, SlurmParser};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FocusedPanel {
    JobList,
    LogView,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Refresh,
    JobSelected(String),
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppState {
    Normal,
    PartitionSearchPopup,
    UserSearchPopup,
    CancelJobPopup,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogViewMode {
    Stdout,
    Stderr,
}

impl fmt::Display for LogViewMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogViewMode::Stdout => write!(f, "stdout"),
            LogViewMode::Stderr => write!(f, "stderr"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewMode {
    ActiveJobs,
    HistoryJobs,
}

impl fmt::Display for ViewMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViewMode::ActiveJobs => write!(f, "Active Jobs"),
            ViewMode::HistoryJobs => write!(f, "History"),
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub job_list: JobList,
    pub history_list: JobList,
    pub state: AppState,
    pub selected_job_index: usize,
    pub selected_job: Option<Job>,
    pub current_user: Option<String>,
    pub current_partition: Option<String>,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub last_history_refresh: Instant,
    pub history_refresh_interval: Duration,
    pub view_mode: ViewMode,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub event_sender: mpsc::UnboundedSender<AppEvent>,
    pub event_receiver: mpsc::UnboundedReceiver<AppEvent>,
    pub confirm_action: bool,
    pub input: String,
    pub log_view_mode: LogViewMode,
    pub list_state: ListState,
    pub focused_panel: FocusedPanel,
    pub stdout_scroll_offsets: HashMap<String, usize>,
    pub stderr_scroll_offsets: HashMap<String, usize>,
    pub logs_area: Option<Rect>,
    pub log_content_stdout: String,
    pub log_content_stderr: String,
    pub theme: Theme,
}

impl App {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            job_list: JobList::new(),
            history_list: JobList::new(),
            state: AppState::Normal,
            selected_job_index: 0,
            selected_job: None,
            current_user: std::env::var("USER").ok(),
            current_partition: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            last_history_refresh: Instant::now(),
            history_refresh_interval: Duration::from_secs(30),
            view_mode: ViewMode::ActiveJobs,
            is_loading: false,
            error_message: None,
            event_sender,
            event_receiver,
            confirm_action: false,
            input: "".to_string(),
            log_view_mode: LogViewMode::Stdout,
            list_state: ListState::default().with_selected(Some(0)),
            focused_panel: FocusedPanel::JobList,
            stdout_scroll_offsets: HashMap::new(),
            stderr_scroll_offsets: HashMap::new(),
            logs_area: None,
            log_content_stdout: String::new(),
            log_content_stderr: String::new(),
            theme: Theme::default(),
        }
    }

    pub fn with_cli(user: Option<String>, partition: Option<String>) -> Self {
        let mut app = Self::new();
        if user.is_some() {
            app.current_user = user;
        }
        app.current_partition = partition;
        app
    }

    pub async fn refresh_jobs(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;

        match self.fetch_jobs().await {
            Ok(jobs) => {
                self.job_list.update(jobs);
                self.update_selected_job();
                self.last_refresh = Instant::now();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to fetch jobs: {}", e));
            }
        }

        self.is_loading = false;
        Ok(())
    }

    async fn fetch_jobs(&self) -> Result<Vec<Job>> {
        // Get basic job list from squeue
        let squeue_output = SlurmCommands::squeue(
            self.current_user.as_deref(),
            self.current_partition.as_deref(),
        )
        .await?;
        let mut jobs = SlurmParser::parse_squeue_output(&squeue_output)?;

        // For each job, get detailed info from scontrol (but only for first few to avoid overwhelming)
        for job in jobs.iter_mut().take(10) {
            if let Ok(scontrol_output) = SlurmCommands::scontrol_show_job(&job.job_id).await
                && let Ok(fields) = SlurmParser::parse_scontrol_output(&scontrol_output)
            {
                SlurmParser::enhance_job_with_scontrol_data(job, fields);
            }
        }

        Ok(jobs)
    }

    pub fn should_refresh(&self) -> bool {
        match self.view_mode {
            ViewMode::ActiveJobs => self.last_refresh.elapsed() >= self.refresh_interval,
            ViewMode::HistoryJobs => {
                self.last_history_refresh.elapsed() >= self.history_refresh_interval
            }
        }
    }

    pub async fn refresh(&mut self) -> Result<()> {
        match self.view_mode {
            ViewMode::ActiveJobs => self.refresh_jobs().await,
            ViewMode::HistoryJobs => self.refresh_history().await,
        }
    }

    pub async fn refresh_history(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;

        match self.fetch_history().await {
            Ok(jobs) => {
                self.history_list.update(jobs);
                self.update_selected_job();
                self.last_history_refresh = Instant::now();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to fetch history: {}", e));
            }
        }

        self.is_loading = false;
        Ok(())
    }

    async fn fetch_history(&self) -> Result<Vec<Job>> {
        let sacct_output = SlurmCommands::sacct(
            self.current_user.as_deref(),
            self.current_partition.as_deref(),
        )
        .await?;
        let jobs = SlurmParser::parse_sacct_output(&sacct_output)?;
        Ok(jobs)
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::ActiveJobs => ViewMode::HistoryJobs,
            ViewMode::HistoryJobs => ViewMode::ActiveJobs,
        };
        self.selected_job_index = 0;
        self.list_state.select(Some(0));
        self.update_selected_job();
    }

    pub fn current_job_list(&self) -> &JobList {
        match self.view_mode {
            ViewMode::ActiveJobs => &self.job_list,
            ViewMode::HistoryJobs => &self.history_list,
        }
    }

    pub fn select_next_job(&mut self) {
        let job_list = self.current_job_list();
        if !job_list.jobs.is_empty() && self.selected_job_index < job_list.jobs.len() - 1 {
            self.selected_job_index += 1;
            self.list_state.select(Some(self.selected_job_index));
            self.update_selected_job();
        }
    }

    pub fn select_previous_job(&mut self) {
        if self.selected_job_index > 0 {
            self.selected_job_index -= 1;
            self.list_state.select(Some(self.selected_job_index));
            self.update_selected_job();
        }
    }

    fn update_selected_job(&mut self) {
        let job_list = self.current_job_list();
        self.selected_job = job_list.jobs.get(self.selected_job_index).cloned();
        self.log_content_stdout.clear();
        self.log_content_stderr.clear();
    }

    pub fn get_selected_job(&self) -> Option<&Job> {
        self.selected_job.as_ref()
    }

    pub fn running_jobs(&self) -> Vec<&Job> {
        self.job_list.running_jobs()
    }

    pub fn pending_jobs(&self) -> Vec<&Job> {
        self.job_list.pending_jobs()
    }

    pub fn completed_jobs(&self) -> Vec<&Job> {
        self.job_list.completed_jobs()
    }

    pub async fn handle_cancel_popup(&mut self) -> Result<()> {
        if self.confirm_action && self.selected_job.is_some() {
            if let Err(e) = self.cancel_selected_job().await {
                self.error_message = Some(format!("Failed to cancel job: {}", e));
            }
            self.confirm_action = false;
        }
        Ok(())
    }

    pub async fn cancel_selected_job(&mut self) -> Result<()> {
        if let Some(job) = &self.selected_job {
            SlurmCommands::scancel(&job.job_id).await?;
            // Refresh immediately to show the change
            self.refresh_jobs().await?;
        }
        Ok(())
    }

    pub fn send_event(&self, event: AppEvent) -> Result<()> {
        self.event_sender.send(event)?;
        Ok(())
    }

    pub async fn receive_event(&mut self) -> Option<AppEvent> {
        self.event_receiver.recv().await
    }

    pub fn toggle_log_view(&mut self) {
        self.log_view_mode = match self.log_view_mode {
            LogViewMode::Stdout => LogViewMode::Stderr,
            LogViewMode::Stderr => LogViewMode::Stdout,
        };
    }

    pub fn log_view_mode_title(&self) -> &'static str {
        match self.log_view_mode {
            LogViewMode::Stdout => "stdout",
            LogViewMode::Stderr => "stderr",
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::JobList => FocusedPanel::LogView,
            FocusedPanel::LogView => FocusedPanel::JobList,
        };
    }

    fn get_scroll_offsets(&mut self) -> &mut HashMap<String, usize> {
        match self.log_view_mode {
            LogViewMode::Stdout => &mut self.stdout_scroll_offsets,
            LogViewMode::Stderr => &mut self.stderr_scroll_offsets,
        }
    }

    fn current_job_id(&self) -> String {
        self.selected_job
            .as_ref()
            .map(|j| j.display_id())
            .unwrap_or_default()
    }

    pub fn get_log_scroll_offset(&mut self) -> usize {
        let job_id = self.current_job_id();
        *self.get_scroll_offsets().get(&job_id).unwrap_or(&0)
    }

    fn set_log_scroll_offset(&mut self, offset: usize) {
        let job_id = self.current_job_id();
        if !job_id.is_empty() {
            self.get_scroll_offsets().insert(job_id, offset);
        }
    }

    pub fn scroll_log_up(&mut self) {
        let current = self.get_log_scroll_offset();
        if current > 0 {
            self.set_log_scroll_offset(current - 1);
        }
    }

    pub fn scroll_log_down(&mut self, max_offset: usize) {
        let current = self.get_log_scroll_offset();
        if current < max_offset {
            self.set_log_scroll_offset(current + 1);
        }
    }

    pub fn scroll_log_page_up(&mut self) {
        let current = self.get_log_scroll_offset();
        self.set_log_scroll_offset(current.saturating_sub(10));
    }

    pub fn scroll_log_page_down(&mut self, max_offset: usize) {
        let current = self.get_log_scroll_offset();
        self.set_log_scroll_offset((current + 10).min(max_offset));
    }

    pub fn scroll_log_to_start(&mut self) {
        self.set_log_scroll_offset(0);
    }

    pub fn scroll_log_to_end(&mut self, max_offset: usize) {
        self.set_log_scroll_offset(max_offset);
    }

    pub fn current_log_content(&self) -> &str {
        match self.log_view_mode {
            LogViewMode::Stdout => &self.log_content_stdout,
            LogViewMode::Stderr => &self.log_content_stderr,
        }
    }

    pub fn set_log_content(&mut self, content: String) {
        match self.log_view_mode {
            LogViewMode::Stdout => self.log_content_stdout = content,
            LogViewMode::Stderr => self.log_content_stderr = content,
        }
    }

    pub fn is_log_focused(&self) -> bool {
        self.focused_panel == FocusedPanel::LogView
    }

    pub fn is_mouse_in_logs_area(&self, row: u16, col: u16) -> bool {
        if let Some(area) = self.logs_area {
            row >= area.y
                && row < area.y + area.height
                && col >= area.x
                && col < area.x + area.width
        } else {
            false
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
