use mux::notification_store::NotificationRecord;
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::{WorkspaceLogRecord, WorkspaceProgressRecord, WorkspaceStatusRecord};
use mux::Mux;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub name: String,
    pub unread_count: usize,
    pub running_count: usize,
    pub failed_count: usize,
    pub status: Option<String>,
    pub progress: Option<u8>,
    pub log_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeSnapshot {
    pub active_workspace: String,
    pub workspaces: Vec<WorkspaceSummary>,
    pub notifications: Vec<NotificationRecord>,
    pub task_panes: Vec<TaskPaneRecord>,
    pub statuses: Vec<WorkspaceStatusRecord>,
    pub progresses: Vec<WorkspaceProgressRecord>,
    pub logs: Vec<WorkspaceLogRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellLayoutContract {
    pub chrome_height: i32,
    pub rail_width: i32,
}

impl Default for ShellLayoutContract {
    fn default() -> Self {
        Self {
            chrome_height: 46,
            rail_width: 176,
        }
    }
}

impl ShellLayoutContract {
    pub fn section_names(&self) -> Vec<&'static str> {
        vec!["chrome", "rail", "main", "context"]
    }
}

pub fn derive_runtime_snapshot(selected_workspace: Option<&str>) -> RuntimeSnapshot {
    let mux = Mux::get();
    let notifications = mux.list_notifications();
    let task_panes = mux.list_task_panes();
    let statuses = mux.list_workspace_status();
    let progresses = mux.list_workspace_progress();

    let mut names = mux.iter_workspaces();
    if names.is_empty() {
        names.push(mux.active_workspace());
    }
    names.sort();
    names.dedup();

    let active_workspace = match selected_workspace {
        Some(selected) if names.iter().any(|name| name == selected) => selected.to_string(),
        _ => names
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string()),
    };

    let logs = mux.list_workspace_log(&active_workspace);
    let workspaces = names
        .iter()
        .map(|name| WorkspaceSummary {
            name: name.clone(),
            unread_count: notifications
                .iter()
                .filter(|record| record.workspace == *name && record.unread)
                .count(),
            running_count: task_panes
                .iter()
                .filter(|record| {
                    record.workspace.as_deref() == Some(name.as_str()) && !record.is_dead
                })
                .count(),
            failed_count: task_panes
                .iter()
                .filter(|record| {
                    record.workspace.as_deref() == Some(name.as_str()) && record.is_failed
                })
                .count(),
            status: statuses
                .iter()
                .find(|record| record.workspace == *name)
                .map(|record| record.status.clone()),
            progress: progresses
                .iter()
                .find(|record| record.workspace == *name)
                .map(|record| record.value),
            log_count: if name == &active_workspace {
                logs.len()
            } else {
                mux.list_workspace_log(name).len()
            },
        })
        .collect::<Vec<_>>();

    RuntimeSnapshot {
        active_workspace,
        workspaces,
        notifications,
        task_panes,
        statuses,
        progresses,
        logs,
    }
}

