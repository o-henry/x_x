use mux::Mux;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellAction {
    Refresh,
    LaunchTerminal,
    SetStatus,
    ClearStatus,
    SetProgress,
    ClearProgress,
    AppendLog,
    MarkVisibleRead,
}

impl ShellAction {
    pub fn operator_actions() -> Vec<Self> {
        vec![
            Self::Refresh,
            Self::LaunchTerminal,
            Self::SetStatus,
            Self::ClearStatus,
            Self::SetProgress,
            Self::ClearProgress,
            Self::AppendLog,
            Self::MarkVisibleRead,
        ]
    }

    pub fn execute<T: ShellActionTarget>(
        self,
        target: &T,
        context: &ShellActionContext,
    ) -> ShellActionOutcome {
        match self {
            Self::Refresh => ShellActionOutcome::NoMutation,
            Self::LaunchTerminal => ShellActionOutcome::NoMutation,
            Self::SetStatus => {
                target.set_workspace_status(&context.workspace, context.next_status());
                ShellActionOutcome::Mutated
            }
            Self::ClearStatus => {
                target.clear_workspace_status(&context.workspace);
                ShellActionOutcome::Mutated
            }
            Self::SetProgress => {
                target.set_workspace_progress(&context.workspace, context.next_progress());
                ShellActionOutcome::Mutated
            }
            Self::ClearProgress => {
                target.clear_workspace_progress(&context.workspace);
                ShellActionOutcome::Mutated
            }
            Self::AppendLog => {
                target.append_workspace_log(&context.workspace, &context.next_log_message());
                ShellActionOutcome::Mutated
            }
            Self::MarkVisibleRead => {
                if context.visible_notification_ids.is_empty() {
                    ShellActionOutcome::NoMutation
                } else {
                    target.mark_notifications_read(&context.visible_notification_ids);
                    ShellActionOutcome::Mutated
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellActionOutcome {
    NoMutation,
    Mutated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellActionContext {
    pub workspace: String,
    pub current_status: Option<String>,
    pub current_progress: Option<u8>,
    pub visible_notification_ids: Vec<String>,
    pub unix_timestamp_secs: u64,
}

impl ShellActionContext {
    pub fn new(
        workspace: impl Into<String>,
        current_status: Option<String>,
        current_progress: Option<u8>,
        visible_notification_ids: Vec<String>,
    ) -> Self {
        let unix_timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            workspace: workspace.into(),
            current_status,
            current_progress,
            visible_notification_ids,
            unix_timestamp_secs,
        }
    }

    pub fn next_status(&self) -> &'static str {
        match self.current_status.as_deref() {
            Some("Planning") => "Building",
            Some("Building") => "Reviewing",
            Some("Reviewing") => "Blocked",
            _ => "Planning",
        }
    }

    pub fn next_progress(&self) -> u8 {
        match self.current_progress.unwrap_or(0) {
            0..=24 => 35,
            25..=64 => 70,
            65..=99 => 100,
            _ => 15,
        }
    }

    pub fn next_log_message(&self) -> String {
        format!("native-shell check-in {}", self.unix_timestamp_secs)
    }
}

pub trait ShellActionTarget {
    fn set_workspace_status(&self, workspace: &str, status: &str);
    fn clear_workspace_status(&self, workspace: &str);
    fn set_workspace_progress(&self, workspace: &str, value: u8);
    fn clear_workspace_progress(&self, workspace: &str);
    fn append_workspace_log(&self, workspace: &str, message: &str);
    fn mark_notifications_read(&self, notification_ids: &[String]);
}

impl ShellActionTarget for Mux {
    fn set_workspace_status(&self, workspace: &str, status: &str) {
        let _ = Mux::set_workspace_status(self, workspace, status);
    }

    fn clear_workspace_status(&self, workspace: &str) {
        let _ = Mux::clear_workspace_status(self, workspace);
    }

    fn set_workspace_progress(&self, workspace: &str, value: u8) {
        let _ = Mux::set_workspace_progress(self, workspace, value);
    }

    fn clear_workspace_progress(&self, workspace: &str) {
        let _ = Mux::clear_workspace_progress(self, workspace);
    }

    fn append_workspace_log(&self, workspace: &str, message: &str) {
        let _ = Mux::append_workspace_log(self, workspace, message);
    }

    fn mark_notifications_read(&self, notification_ids: &[String]) {
        let _ = Mux::mark_notifications_read(self, notification_ids);
    }
}
