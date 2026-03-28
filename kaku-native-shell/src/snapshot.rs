use mux::notification_store::NotificationRecord;
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::{WorkspaceLogRecord, WorkspaceProgressRecord, WorkspaceStatusRecord};
use mux::{Mux, MuxNotification};

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
    pub collapsed_rail_width: i32,
    pub body_split: i32,
    pub workspace_split: i32,
    pub lower_split: i32,
    pub side_split: i32,
}

impl Default for ShellLayoutContract {
    fn default() -> Self {
        Self {
            chrome_height: 32,
            rail_width: 256,
            collapsed_rail_width: 72,
            body_split: 940,
            workspace_split: 430,
            lower_split: 620,
            side_split: 316,
        }
    }
}

impl ShellLayoutContract {
    pub fn section_names(&self) -> Vec<&'static str> {
        vec!["chrome", "rail", "main", "context"]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotRefreshScope {
    Ignore,
    WorkspaceList,
    WorkspaceContext,
    Notifications,
    TaskPanes,
}

pub trait RuntimeSnapshotSource {
    fn active_workspace(&self) -> String;
    fn iter_workspaces(&self) -> Vec<String>;
    fn list_notifications(&self) -> Vec<NotificationRecord>;
    fn list_task_panes(&self) -> Vec<TaskPaneRecord>;
    fn list_workspace_status(&self) -> Vec<WorkspaceStatusRecord>;
    fn list_workspace_progress(&self) -> Vec<WorkspaceProgressRecord>;
    fn list_workspace_log(&self, workspace: &str) -> Vec<WorkspaceLogRecord>;
}

impl RuntimeSnapshotSource for Mux {
    fn active_workspace(&self) -> String {
        Mux::active_workspace(self)
    }

    fn iter_workspaces(&self) -> Vec<String> {
        Mux::iter_workspaces(self)
    }

    fn list_notifications(&self) -> Vec<NotificationRecord> {
        Mux::list_notifications(self)
    }

    fn list_task_panes(&self) -> Vec<TaskPaneRecord> {
        Mux::list_task_panes(self)
    }

    fn list_workspace_status(&self) -> Vec<WorkspaceStatusRecord> {
        Mux::list_workspace_status(self)
    }

    fn list_workspace_progress(&self) -> Vec<WorkspaceProgressRecord> {
        Mux::list_workspace_progress(self)
    }

    fn list_workspace_log(&self, workspace: &str) -> Vec<WorkspaceLogRecord> {
        Mux::list_workspace_log(self, workspace)
    }
}

impl RuntimeSnapshot {
    pub fn from_source<S: RuntimeSnapshotSource>(
        source: &S,
        selected_workspace: Option<&str>,
    ) -> RuntimeSnapshot {
        let notifications = source.list_notifications();
        let task_panes = source.list_task_panes();
        let statuses = source.list_workspace_status();
        let progresses = source.list_workspace_progress();

        let mut names = source.iter_workspaces();
        if names.is_empty() {
            names.push(source.active_workspace());
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

        let logs = source.list_workspace_log(&active_workspace);
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
                    source.list_workspace_log(name).len()
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
}

pub fn refresh_scope_for_notification(notification: &MuxNotification) -> SnapshotRefreshScope {
    match notification {
        MuxNotification::NotificationsChanged => SnapshotRefreshScope::Notifications,
        MuxNotification::WorkspaceMetadataChanged => SnapshotRefreshScope::WorkspaceContext,
        MuxNotification::TaskPaneLifecycleChanged(_)
        | MuxNotification::PaneAdded(_)
        | MuxNotification::PaneRemoved(_) => SnapshotRefreshScope::TaskPanes,
        MuxNotification::ActiveWorkspaceChanged(_)
        | MuxNotification::WorkspaceRenamed { .. }
        | MuxNotification::WindowWorkspaceChanged(_) => SnapshotRefreshScope::WorkspaceList,
        _ => SnapshotRefreshScope::Ignore,
    }
}

pub fn derive_runtime_snapshot(selected_workspace: Option<&str>) -> RuntimeSnapshot {
    let mux = Mux::get();
    RuntimeSnapshot::from_source(mux.as_ref(), selected_workspace)
}
