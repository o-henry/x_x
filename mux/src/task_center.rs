use crate::notification_store::NotificationRecord;
use crate::pane::PaneId;
use crate::tab::TabId;
use crate::task_panes::TaskPaneRecord;
use crate::window::WindowId;
use crate::workspace_state::{WorkspaceProgressRecord, WorkspaceStatusRecord};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use wezterm_term::Progress;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaskCenterSource {
    Workspace,
    Tab,
    Pane,
    Notification,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaskCenterKind {
    Workspace,
    Tab,
    Pane,
    Notification,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskCenterEntry {
    pub label: String,
    pub workspace: String,
    pub source: TaskCenterSource,
    pub kind: TaskCenterKind,
    pub source_label: Option<String>,
    pub kind_label: Option<String>,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
    pub unread_count: usize,
    pub notification_ids: Vec<String>,
    pub is_failed: bool,
    pub is_running: bool,
    pub rerun_available: bool,
    pub workspace_status: Option<String>,
    pub workspace_progress: Option<u8>,
}

#[derive(Clone, Debug)]
pub(crate) struct TaskCenterPaneSnapshot {
    pub pane_id: PaneId,
    pub title: String,
    pub progress: Progress,
    pub is_dead: bool,
    pub user_vars: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub(crate) struct TaskCenterTabSnapshot {
    pub tab_id: TabId,
    pub title: String,
    pub panes: Vec<TaskCenterPaneSnapshot>,
}

#[derive(Clone, Debug)]
pub(crate) struct TaskCenterWindowSnapshot {
    pub window_id: WindowId,
    pub workspace: String,
    pub tabs: Vec<TaskCenterTabSnapshot>,
}

pub(crate) fn build_task_center_entries(
    workspaces: &[String],
    statuses: &[WorkspaceStatusRecord],
    progresses: &[WorkspaceProgressRecord],
    log_workspaces: &[String],
    task_panes: &[TaskPaneRecord],
    notifications: &[NotificationRecord],
    notification_anchor_panes: &HashMap<String, PaneId>,
    windows: &[TaskCenterWindowSnapshot],
) -> Vec<TaskCenterEntry> {
    let status_by_workspace = statuses
        .iter()
        .map(|record| (record.workspace.clone(), record.status.clone()))
        .collect::<HashMap<_, _>>();
    let progress_by_workspace = progresses
        .iter()
        .map(|record| (record.workspace.clone(), record.value))
        .collect::<HashMap<_, _>>();

    let mut workspace_names = BTreeSet::new();
    for workspace in workspaces {
        workspace_names.insert(workspace.clone());
    }
    for workspace in log_workspaces {
        workspace_names.insert(workspace.clone());
    }
    for workspace in status_by_workspace.keys() {
        workspace_names.insert(workspace.clone());
    }
    for workspace in progress_by_workspace.keys() {
        workspace_names.insert(workspace.clone());
    }
    for notification in notifications {
        workspace_names.insert(notification.workspace.clone());
    }
    for task_pane in task_panes {
        if let Some(workspace) = &task_pane.workspace {
            workspace_names.insert(workspace.clone());
        }
    }
    for window in windows {
        workspace_names.insert(window.workspace.clone());
    }

    let task_panes_by_id = task_panes
        .iter()
        .map(|record| (record.pane_id, record))
        .collect::<HashMap<_, _>>();

    let mut unread_notification_ids_by_workspace: HashMap<String, Vec<String>> = HashMap::new();
    let mut unread_notification_ids_by_pane: HashMap<PaneId, Vec<String>> = HashMap::new();
    let mut notification_entries_by_workspace: HashMap<String, Vec<TaskCenterEntry>> =
        HashMap::new();

    for notification in notifications {
        if !notification.unread {
            continue;
        }

        unread_notification_ids_by_workspace
            .entry(notification.workspace.clone())
            .or_default()
            .push(notification.notification_id.clone());

        let anchor_pane_id = notification_anchor_panes
            .get(&notification.notification_id)
            .copied()
            .or(notification.pane_id);
        if let Some(anchor_pane_id) = anchor_pane_id {
            unread_notification_ids_by_pane
                .entry(anchor_pane_id)
                .or_default()
                .push(notification.notification_id.clone());
        }

        notification_entries_by_workspace
            .entry(notification.workspace.clone())
            .or_default()
            .push(TaskCenterEntry {
                label: notification.title.clone(),
                workspace: notification.workspace.clone(),
                source: TaskCenterSource::Notification,
                kind: TaskCenterKind::Notification,
                source_label: Some("notification".to_string()),
                kind_label: Some(notification.kind.clone()),
                window_id: notification.window_id,
                tab_id: notification.tab_id,
                pane_id: notification.pane_id.or(anchor_pane_id),
                unread_count: 1,
                notification_ids: vec![notification.notification_id.clone()],
                is_failed: false,
                is_running: false,
                rerun_available: false,
                workspace_status: None,
                workspace_progress: None,
            });
    }

    let mut windows_by_workspace: HashMap<String, Vec<&TaskCenterWindowSnapshot>> = HashMap::new();
    for window in windows {
        windows_by_workspace
            .entry(window.workspace.clone())
            .or_default()
            .push(window);
    }
    for windows in windows_by_workspace.values_mut() {
        windows.sort_by_key(|window| window.window_id);
    }

    let mut entries = vec![];

    for workspace in workspace_names {
        let workspace_status = status_by_workspace.get(&workspace).cloned();
        let workspace_progress = progress_by_workspace.get(&workspace).copied();
        let workspace_unread_ids = unread_notification_ids_by_workspace
            .get(&workspace)
            .cloned()
            .unwrap_or_default();

        let windows_for_workspace = windows_by_workspace
            .get(&workspace)
            .cloned()
            .unwrap_or_default();
        let mut workspace_failed = false;
        let mut workspace_running = false;
        let mut workspace_rerun = false;

        let mut tab_entries = vec![];
        let mut pane_entries = vec![];
        let mut seen_panes = BTreeSet::new();

        for window in windows_for_workspace {
            for tab in &window.tabs {
                let mut tab_unread_ids = workspace_unread_ids.clone();
                let mut tab_failed = false;
                let mut tab_running = false;
                let mut tab_rerun = false;

                for pane in &tab.panes {
                    seen_panes.insert(pane.pane_id);
                    let pane_unread_ids = unread_notification_ids_by_pane
                        .get(&pane.pane_id)
                        .cloned()
                        .unwrap_or_default();
                    tab_unread_ids.extend(pane_unread_ids.iter().cloned());

                    let lifecycle = task_panes_by_id.get(&pane.pane_id).copied();
                    let is_failed = lifecycle
                        .map(|record| record.is_failed)
                        .unwrap_or(pane.is_dead);
                    let is_running = lifecycle.map(|record| !record.is_dead).unwrap_or(
                        !pane.is_dead
                            && !matches!(pane.progress, Progress::None | Progress::Error(_)),
                    );
                    let rerun_available = lifecycle
                        .map(|record| !record.rerun.is_empty())
                        .unwrap_or_else(|| {
                            is_failed && rerun_available_from_user_vars(&pane.user_vars)
                        });

                    tab_failed |= is_failed;
                    tab_running |= is_running;
                    tab_rerun |= rerun_available;

                    pane_entries.push(TaskCenterEntry {
                        label: pane.title.clone(),
                        workspace: workspace.clone(),
                        source: TaskCenterSource::Pane,
                        kind: TaskCenterKind::Pane,
                        source_label: Some("pane".to_string()),
                        kind_label: None,
                        window_id: Some(window.window_id),
                        tab_id: Some(tab.tab_id),
                        pane_id: Some(pane.pane_id),
                        unread_count: pane_unread_ids.len(),
                        notification_ids: pane_unread_ids,
                        is_failed,
                        is_running,
                        rerun_available,
                        workspace_status: workspace_status.clone(),
                        workspace_progress,
                    });
                }

                dedup_strings(&mut tab_unread_ids);
                workspace_failed |= tab_failed;
                workspace_running |= tab_running;
                workspace_rerun |= tab_rerun;

                let tab_label = if tab.title.is_empty() {
                    tab.panes
                        .first()
                        .map(|pane| pane.title.clone())
                        .unwrap_or_else(|| "no pane".to_string())
                } else {
                    tab.title.clone()
                };

                tab_entries.push(TaskCenterEntry {
                    label: tab_label,
                    workspace: workspace.clone(),
                    source: TaskCenterSource::Tab,
                    kind: TaskCenterKind::Tab,
                    source_label: Some("tab".to_string()),
                    kind_label: None,
                    window_id: Some(window.window_id),
                    tab_id: Some(tab.tab_id),
                    pane_id: None,
                    unread_count: tab_unread_ids.len(),
                    notification_ids: tab_unread_ids,
                    is_failed: tab_failed,
                    is_running: tab_running,
                    rerun_available: tab_rerun,
                    workspace_status: workspace_status.clone(),
                    workspace_progress,
                });
            }
        }

        for task_pane in task_panes
            .iter()
            .filter(|record| record.workspace.as_deref() == Some(workspace.as_str()))
            .filter(|record| !seen_panes.contains(&record.pane_id))
        {
            let pane_unread_ids = unread_notification_ids_by_pane
                .get(&task_pane.pane_id)
                .cloned()
                .unwrap_or_default();
            let label = task_pane
                .current_working_dir
                .clone()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("pane {}", task_pane.pane_id));
            workspace_failed |= task_pane.is_failed;
            workspace_running |= !task_pane.is_dead;
            workspace_rerun |= !task_pane.rerun.is_empty();

            pane_entries.push(TaskCenterEntry {
                label,
                workspace: workspace.clone(),
                source: TaskCenterSource::Pane,
                kind: TaskCenterKind::Pane,
                source_label: Some("pane".to_string()),
                kind_label: Some("task-pane".to_string()),
                window_id: task_pane.window_id,
                tab_id: task_pane.tab_id,
                pane_id: Some(task_pane.pane_id),
                unread_count: pane_unread_ids.len(),
                notification_ids: pane_unread_ids,
                is_failed: task_pane.is_failed,
                is_running: !task_pane.is_dead,
                rerun_available: !task_pane.rerun.is_empty(),
                workspace_status: workspace_status.clone(),
                workspace_progress,
            });
        }

        entries.push(TaskCenterEntry {
            label: workspace.clone(),
            workspace: workspace.clone(),
            source: TaskCenterSource::Workspace,
            kind: TaskCenterKind::Workspace,
            source_label: Some("workspace".to_string()),
            kind_label: None,
            window_id: None,
            tab_id: None,
            pane_id: None,
            unread_count: workspace_unread_ids.len(),
            notification_ids: workspace_unread_ids,
            is_failed: workspace_failed,
            is_running: workspace_running,
            rerun_available: workspace_rerun,
            workspace_status: workspace_status.clone(),
            workspace_progress,
        });

        entries.extend(tab_entries);
        entries.extend(pane_entries);

        if let Some(notification_entries) = notification_entries_by_workspace.get(&workspace) {
            entries.extend(notification_entries.clone());
        }
    }

    entries
}

fn rerun_available_from_user_vars(user_vars: &HashMap<String, String>) -> bool {
    user_vars.keys().any(|key| {
        key == "KAKU_RERUN_COMMAND"
            || key == "KAKU_RERUN_PAYLOAD"
            || key.starts_with("KAKU_RERUN_")
            || key.starts_with("kaku.rerun.")
    })
}

fn dedup_strings(values: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification_store::NotificationUnreadMode;
    use chrono::{DateTime, Utc};

    fn timestamp() -> DateTime<Utc> {
        DateTime::from_timestamp(1_774_578_000, 0).unwrap()
    }

    fn unread_notification(
        id: &str,
        workspace: &str,
        pane_id: Option<PaneId>,
        kind: &str,
        title: &str,
    ) -> NotificationRecord {
        NotificationRecord {
            notification_id: id.to_string(),
            workspace: workspace.to_string(),
            window_id: None,
            tab_id: None,
            pane_id,
            kind: kind.to_string(),
            title: title.to_string(),
            body: None,
            unread: true,
            unread_mode: NotificationUnreadMode::Sticky,
            created_at: timestamp(),
            updated_at: timestamp(),
        }
    }

    #[test]
    fn task_center_snapshot_represents_workspace_tab_and_pane_rows() {
        let entries = build_task_center_entries(
            &["unity-main".to_string()],
            &[WorkspaceStatusRecord {
                workspace: "unity-main".to_string(),
                status: "blocked".to_string(),
                updated_at: timestamp(),
            }],
            &[WorkspaceProgressRecord {
                workspace: "unity-main".to_string(),
                value: 37,
                updated_at: timestamp(),
            }],
            &[],
            &[],
            &[],
            &HashMap::new(),
            &[TaskCenterWindowSnapshot {
                window_id: 1,
                workspace: "unity-main".to_string(),
                tabs: vec![TaskCenterTabSnapshot {
                    tab_id: TabId::new(9),
                    title: "Build".to_string(),
                    panes: vec![TaskCenterPaneSnapshot {
                        pane_id: PaneId::new(12),
                        title: "Compiler".to_string(),
                        progress: Progress::None,
                        is_dead: false,
                        user_vars: HashMap::new(),
                    }],
                }],
            }],
        );

        assert!(entries
            .iter()
            .any(|entry| entry.source == TaskCenterSource::Workspace));
        assert!(entries
            .iter()
            .any(|entry| entry.source == TaskCenterSource::Tab));
        assert!(entries
            .iter()
            .any(|entry| entry.source == TaskCenterSource::Pane));

        let workspace_entry = entries
            .iter()
            .find(|entry| entry.source == TaskCenterSource::Workspace)
            .unwrap();
        assert_eq!(workspace_entry.workspace_status.as_deref(), Some("blocked"));
        assert_eq!(workspace_entry.workspace_progress, Some(37));
    }

    #[test]
    fn task_center_snapshot_derives_unread_from_notification_records() {
        let pane_id = PaneId::new(12);
        let notifications = vec![
            unread_notification(
                "notif-pane",
                "unity-main",
                Some(pane_id),
                "build.failed",
                "Pane",
            ),
            unread_notification(
                "notif-workspace",
                "unity-main",
                None,
                "workspace",
                "Workspace",
            ),
        ];
        let anchor_panes = HashMap::from([("notif-workspace".to_string(), pane_id)]);

        let entries = build_task_center_entries(
            &["unity-main".to_string()],
            &[],
            &[],
            &[],
            &[],
            &notifications,
            &anchor_panes,
            &[TaskCenterWindowSnapshot {
                window_id: 1,
                workspace: "unity-main".to_string(),
                tabs: vec![TaskCenterTabSnapshot {
                    tab_id: TabId::new(9),
                    title: "Build".to_string(),
                    panes: vec![TaskCenterPaneSnapshot {
                        pane_id,
                        title: "Compiler".to_string(),
                        progress: Progress::None,
                        is_dead: false,
                        user_vars: HashMap::new(),
                    }],
                }],
            }],
        );

        let workspace_entry = entries
            .iter()
            .find(|entry| entry.source == TaskCenterSource::Workspace)
            .unwrap();
        assert_eq!(workspace_entry.unread_count, 2);
        assert_eq!(workspace_entry.notification_ids.len(), 2);

        let tab_entry = entries
            .iter()
            .find(|entry| entry.source == TaskCenterSource::Tab)
            .unwrap();
        assert_eq!(tab_entry.unread_count, 2);

        let pane_entry = entries
            .iter()
            .find(|entry| entry.source == TaskCenterSource::Pane)
            .unwrap();
        assert_eq!(pane_entry.unread_count, 2);

        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.source == TaskCenterSource::Notification)
                .count(),
            2
        );
    }

    #[test]
    fn task_center_snapshot_uses_limited_failed_running_and_rerun_signals() {
        let entries = build_task_center_entries(
            &["unity-main".to_string()],
            &[],
            &[],
            &[],
            &[],
            &[],
            &HashMap::new(),
            &[TaskCenterWindowSnapshot {
                window_id: 1,
                workspace: "unity-main".to_string(),
                tabs: vec![TaskCenterTabSnapshot {
                    tab_id: TabId::new(9),
                    title: "Tasks".to_string(),
                    panes: vec![
                        TaskCenterPaneSnapshot {
                            pane_id: PaneId::new(12),
                            title: "Runner".to_string(),
                            progress: Progress::Percentage(55),
                            is_dead: false,
                            user_vars: HashMap::new(),
                        },
                        TaskCenterPaneSnapshot {
                            pane_id: PaneId::new(13),
                            title: "Failed".to_string(),
                            progress: Progress::None,
                            is_dead: true,
                            user_vars: HashMap::from([(
                                "KAKU_RERUN_COMMAND".to_string(),
                                "cargo test".to_string(),
                            )]),
                        },
                    ],
                }],
            }],
        );

        let running_entry = entries
            .iter()
            .find(|entry| entry.pane_id == Some(PaneId::new(12)))
            .unwrap();
        assert!(running_entry.is_running);
        assert!(!running_entry.is_failed);

        let failed_entry = entries
            .iter()
            .find(|entry| entry.pane_id == Some(PaneId::new(13)))
            .unwrap();
        assert!(failed_entry.is_failed);
        assert!(!failed_entry.is_running);
        assert!(failed_entry.rerun_available);
    }
}
