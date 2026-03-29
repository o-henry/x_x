use mux::notification_store::NotificationRecord;
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::{WorkspaceLogRecord, WorkspaceProgressRecord, WorkspaceStatusRecord};
use mux::{Mux, MuxNotification};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub name: String,
    pub detail: Option<String>,
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
            chrome_height: 28,
            rail_width: 284,
            collapsed_rail_width: 0,
            body_split: 872,
            workspace_split: 356,
            lower_split: 618,
            side_split: 248,
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
            .map(|name| {
                let unread_count = notifications
                    .iter()
                    .filter(|record| record.workspace == *name && record.unread)
                    .count();
                let running_count = task_panes
                    .iter()
                    .filter(|record| {
                        record.workspace.as_deref() == Some(name.as_str()) && !record.is_dead
                    })
                    .count();
                let failed_count = task_panes
                    .iter()
                    .filter(|record| {
                        record.workspace.as_deref() == Some(name.as_str()) && record.is_failed
                    })
                    .count();
                WorkspaceSummary {
                    name: name.clone(),
                    detail: workspace_detail(&task_panes, name, running_count, failed_count),
                    unread_count,
                    running_count,
                    failed_count,
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
                        0
                    },
                }
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

fn workspace_detail(
    task_panes: &[TaskPaneRecord],
    workspace: &str,
    running_count: usize,
    failed_count: usize,
) -> Option<String> {
    let pane = task_panes
        .iter()
        .filter(|record| record.workspace.as_deref() == Some(workspace))
        .max_by_key(|record| (usize::from(!record.is_dead), record.updated_at));

    let location = pane
        .and_then(|record| record.current_working_dir.as_deref())
        .and_then(|cwd| git_branch_for_cwd(cwd).or_else(|| Some(friendly_display_path(cwd))));

    let shell_summary = match running_count {
        0 if failed_count > 0 => Some("FAILED".to_string()),
        0 => None,
        1 => Some("1 SHELL".to_string()),
        n => Some(format!("{n} SHELLS")),
    };

    let mut parts = Vec::new();
    if let Some(location) = location {
        parts.push(location);
    }
    if let Some(shell_summary) = shell_summary {
        parts.push(shell_summary);
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" • "))
    }
}

fn git_branch_for_cwd(cwd: &str) -> Option<String> {
    let mut path = PathBuf::from(cwd.strip_prefix("file://").unwrap_or(cwd));
    if path.is_file() {
        path = path.parent()?.to_path_buf();
    }

    let git_dir = discover_git_dir(&path)?;
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref: ") {
        return reference
            .rsplit('/')
            .next()
            .map(|branch| branch.replace('-', " ").to_uppercase());
    }
    None
}

fn discover_git_dir(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if dot_git.is_file() {
            let gitdir = fs::read_to_string(&dot_git).ok()?;
            let target = gitdir.trim().strip_prefix("gitdir: ")?.trim();
            let resolved = if Path::new(target).is_absolute() {
                PathBuf::from(target)
            } else {
                dir.join(target)
            };
            return Some(resolved);
        }
        current = dir.parent();
    }
    None
}

fn friendly_display_path(path: &str) -> String {
    let without_scheme = path.strip_prefix("file://").unwrap_or(path);
    let candidate = PathBuf::from(without_scheme);
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs_next::home_dir);
    if let Some(home) = home {
        if candidate == home {
            return "~".to_string();
        }
        if let Ok(stripped) = candidate.strip_prefix(&home) {
            let stripped = stripped.to_string_lossy();
            if stripped.is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", stripped.trim_start_matches('/'));
        }
    }
    without_scheme.to_string()
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
