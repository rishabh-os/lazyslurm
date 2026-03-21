use crate::ui::{AppState, ViewMode};

#[derive(Debug, Clone, Copy)]
pub struct HelpAction {
    pub key: &'static str,
    pub description: &'static str,
}

impl HelpAction {
    pub const fn new(key: &'static str, description: &'static str) -> Self {
        Self { key, description }
    }

    pub fn format(&self) -> String {
        format!("{}: {}", self.key, self.description)
    }
}

pub const QUIT: HelpAction = HelpAction::new("q", "quit");
pub const TAB: HelpAction = HelpAction::new("tab", "focus");
pub const NAV: HelpAction = HelpAction::new("↑↓", "nav/scroll");
pub const REFRESH: HelpAction = HelpAction::new("r", "refresh");
pub const HISTORY: HelpAction = HelpAction::new("h", "toggle history");
pub const CANCEL: HelpAction = HelpAction::new("c", "cancel");
pub const TOGGLE_LOGS: HelpAction = HelpAction::new("l", "toggle logs");
pub const CONFIRM: HelpAction = HelpAction::new("y", "confirm");
pub const REJECT: HelpAction = HelpAction::new("n/esc", "reject");
pub const CLOSE: HelpAction = HelpAction::new("esc", "close");
pub const SUBMIT: HelpAction = HelpAction::new("Enter", "submit");
pub const USER_SEARCH: HelpAction = HelpAction::new("u", "user filter");
pub const PARTITION_SEARCH: HelpAction = HelpAction::new("p", "partition filter");

const BASE_NAV_ACTIONS: [&HelpAction; 8] = [
    &TAB,
    &NAV,
    &REFRESH,
    &HISTORY,
    &TOGGLE_LOGS,
    &USER_SEARCH,
    &PARTITION_SEARCH,
    &QUIT,
];
const BASE_POPUP_ACTIONS: [&HelpAction; 2] = [&CLOSE, &SUBMIT];
const SEPARATOR: &str = " | ";

pub fn format_actions(actions: &[&HelpAction]) -> String {
    actions
        .iter()
        .map(|a| a.format())
        .collect::<Vec<_>>()
        .join(SEPARATOR)
}

pub fn get_help_text(app_state: AppState, view_mode: ViewMode) -> String {
    match app_state {
        AppState::Normal => {
            let mut actions: Vec<&HelpAction> = BASE_NAV_ACTIONS.into();
            if view_mode == ViewMode::ActiveJobs {
                actions.insert(actions.len() - 1, &CANCEL);
            }
            format_actions(&actions)
        }
        AppState::CancelJobPopup => format_actions(&[&CONFIRM, &REJECT]),
        AppState::PartitionSearchPopup | AppState::UserSearchPopup => {
            format_actions(&BASE_POPUP_ACTIONS)
        }
    }
}

pub const CLI_AFTER_HELP: &str = r#"Keyboard shortcuts:
  q: quit
  ↑/↓ or j/k: navigate jobs
  r: refresh jobs
  h: toggle history view
  u: filter by user
  p: filter by partition
  c: cancel selected job

Notes:
  - SLURM tools required for normal operation: squeue, scontrol, scancel, sacct.
"#;

pub fn cli_help_text() -> &'static str {
    CLI_AFTER_HELP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_action_format() {
        assert_eq!(QUIT.format(), "q: quit");
        assert_eq!(NAV.format(), "↑↓: nav/scroll");
    }

    #[test]
    fn test_get_help_text_normal_active() {
        let help = get_help_text(AppState::Normal, ViewMode::ActiveJobs);
        assert!(help.contains("q: quit"));
        assert!(help.contains("c: cancel"));
        assert!(help.contains("user filter"));
        assert!(help.contains("partition filter"));
    }

    #[test]
    fn test_get_help_text_normal_history() {
        let help = get_help_text(AppState::Normal, ViewMode::HistoryJobs);
        assert!(help.contains("q: quit"));
        assert!(!help.contains("c: cancel"));
        assert!(help.contains("user filter"));
        assert!(help.contains("partition filter"));
    }

    #[test]
    fn test_get_help_text_cancel_popup() {
        let help = get_help_text(AppState::CancelJobPopup, ViewMode::ActiveJobs);
        assert_eq!(help, "y: confirm | n/esc: reject");
        assert_eq!(help, CONFIRM.format() + " | " + &REJECT.format());
    }

    #[test]
    fn test_cli_help_text() {
        let help = cli_help_text();
        assert!(help.contains("Keyboard shortcuts:"));
        assert!(help.contains("q: quit"));
        assert!(help.contains("Notes:"));
    }
}
