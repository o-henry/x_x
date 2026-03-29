use chrono::{DateTime, Utc};
use kaku_native_shell::app_controller::shell_ui_contract;
use kaku_native_shell::snapshot::{
    refresh_scope_for_notification, RuntimeSnapshot, RuntimeSnapshotSource, ShellLayoutContract,
    SnapshotRefreshScope, WorkspaceSummary,
};
use kaku_native_shell::view::workspace::shortcut_entries;
use kaku_native_shell::view::{context_panel_titles, shell_slot_order};
use mux::client::ClientId;
use mux::notification_store::{NotificationRecord, NotificationUnreadMode};
use mux::pane::PaneId;
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::{WorkspaceLogRecord, WorkspaceProgressRecord, WorkspaceStatusRecord};
use mux::{MuxNotification, DEFAULT_WORKSPACE};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn shell_layout_contract() {
    let layout = ShellLayoutContract::default();
    assert_eq!(layout.chrome_height, 28);
    assert_eq!(layout.rail_width, 284);
    assert!(layout.section_names().contains(&"chrome"));
    assert!(layout.section_names().contains(&"rail"));
    assert!(layout.section_names().contains(&"main"));
    assert!(layout.section_names().contains(&"context"));

    let workspace = WorkspaceSummary {
        name: "default".to_string(),
        detail: Some("MAIN • 2 SHELLS".to_string()),
        unread_count: 1,
        running_count: 2,
        failed_count: 0,
        status: Some("running".to_string()),
        progress: Some(32),
        log_count: 5,
    };
    assert_eq!(workspace.name, "default");
    assert_eq!(workspace.progress, Some(32));

    let contract = shell_ui_contract();
    assert_eq!(contract.primary_surface, "workspace");
    assert_eq!(
        contract.persistent_context_slots,
        ["inbox", "tasks", "metadata"]
    );
    assert!(contract.layout_slots.contains(&"chrome"));
    assert!(contract.layout_slots.contains(&"rail"));
    assert!(contract.layout_slots.contains(&"activity"));
    assert_eq!(
        shell_slot_order(),
        [
            "chrome",
            "rail",
            "workspace",
            "activity",
            "metadata",
            "inbox",
            "tasks"
        ]
    );
    assert_eq!(
        context_panel_titles(),
        ["Inbox", "Tasks", "Activity", "Metadata"]
    );
}

#[test]
fn shell_typography_contract() {
    let contract = shell_ui_contract();
    assert_eq!(
        contract.typography.primary_mono_family,
        ["DMMono Nerd Font", "1984대화나눔_본문체_Regular", "monospace"]
    );
    assert!(contract
        .typography
        .operator_classes
        .contains(&"chrome-title"));
    assert!(contract.typography.operator_classes.contains(&"rail-name"));
    assert!(contract.typography.operator_classes.contains(&"pane-title"));
}

#[test]
fn shell_affordance_contract() {
    let contract = shell_ui_contract();
    assert!(contract
        .affordances
        .compact_count_labels
        .iter()
        .all(|label| !label.contains("unread ") && !label.contains("running ")));
    assert!(contract
        .affordances
        .header_badges
        .iter()
        .all(|label| label.chars().count() <= 12));
    assert!(contract
        .affordances
        .header_badges
        .iter()
        .any(|label| label.chars().any(|ch| !ch.is_ascii_alphanumeric())));
    assert!(contract.affordances.action_labels.contains(&"↻"));
    assert!(contract.affordances.action_labels.contains(&"shell"));
    assert!(contract.affordances.action_labels.contains(&"next"));
    assert!(contract.affordances.action_labels.contains(&"close"));
}

#[test]
fn native_shell_shortcut_strip_tracks_real_pane_actions() {
    let entries = shortcut_entries();
    assert!(entries.contains(&("NEW SHELL", "CMD+T")));
    assert!(entries.contains(&("SPLIT RIGHT", "CMD+D")));
    assert!(entries.contains(&("SPLIT DOWN", "CMD+SHIFT+D")));
    assert!(entries.contains(&("TOGGLE SPLIT", "CMD+SHIFT+S")));
    assert!(entries.contains(&("ZOOM", "CMD+SHIFT+ENTER")));
    assert!(entries.contains(&("NEXT PANE", "CMD+]")));
    assert!(entries.contains(&("PREV PANE", "CMD+[")));
    assert!(entries.contains(&("CLOSE PANE", "CMD+W")));
    assert!(entries.contains(&("LAZYGIT", "CMD+SHIFT+G")));
    assert!(entries.contains(&("YAZI", "CMD+SHIFT+Y")));
    assert!(entries.contains(&("DOCTOR", "CMD+SHIFT+O")));
    assert!(entries.contains(&("CONFIG", "CMD+,")));
    assert!(entries.contains(&("CLEAR", "CMD+SHIFT+X")));
    assert!(entries.contains(&("PROGRESS", "CMD+SHIFT+P")));
    assert!(entries.contains(&("LOG", "CMD+SHIFT+L")));
}

#[derive(Clone, Default)]
struct SeededSource {
    active_workspace: String,
    workspaces: Vec<String>,
    notifications: Vec<NotificationRecord>,
    task_panes: Vec<TaskPaneRecord>,
    statuses: Vec<WorkspaceStatusRecord>,
    progresses: Vec<WorkspaceProgressRecord>,
    logs: HashMap<String, Vec<WorkspaceLogRecord>>,
}

impl RuntimeSnapshotSource for SeededSource {
    fn active_workspace(&self) -> String {
        self.active_workspace.clone()
    }

    fn iter_workspaces(&self) -> Vec<String> {
        self.workspaces.clone()
    }

    fn list_notifications(&self) -> Vec<NotificationRecord> {
        self.notifications.clone()
    }

    fn list_task_panes(&self) -> Vec<TaskPaneRecord> {
        self.task_panes.clone()
    }

    fn list_workspace_status(&self) -> Vec<WorkspaceStatusRecord> {
        self.statuses.clone()
    }

    fn list_workspace_progress(&self) -> Vec<WorkspaceProgressRecord> {
        self.progresses.clone()
    }

    fn list_workspace_log(&self, workspace: &str) -> Vec<WorkspaceLogRecord> {
        self.logs.get(workspace).cloned().unwrap_or_default()
    }
}

#[test]
fn shell_uses_mux_truth() {
    let now = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("fixed timestamp");
    let source = SeededSource {
        active_workspace: DEFAULT_WORKSPACE.to_string(),
        workspaces: vec![DEFAULT_WORKSPACE.to_string(), "unity-main".to_string()],
        notifications: vec![
            NotificationRecord {
                notification_id: "notif-1".to_string(),
                workspace: "unity-main".to_string(),
                window_id: None,
                tab_id: None,
                pane_id: None,
                kind: "build".to_string(),
                title: "Build failed".to_string(),
                body: Some("worker pane needs attention".to_string()),
                unread: true,
                unread_mode: NotificationUnreadMode::Sticky,
                created_at: now,
                updated_at: now,
            },
            NotificationRecord {
                notification_id: "notif-2".to_string(),
                workspace: "unity-main".to_string(),
                window_id: None,
                tab_id: None,
                pane_id: None,
                kind: "review".to_string(),
                title: "Review ready".to_string(),
                body: None,
                unread: false,
                unread_mode: NotificationUnreadMode::ClearOnFocus,
                created_at: now,
                updated_at: now,
            },
        ],
        task_panes: vec![
            TaskPaneRecord {
                pane_id: PaneId::new(11),
                workspace: Some("unity-main".to_string()),
                window_id: None,
                tab_id: None,
                remain_on_exit: true,
                silenced: false,
                is_dead: false,
                is_failed: false,
                exit_behavior: None,
                current_working_dir: Some("file:///tmp/unity-main".to_string()),
                rerun: HashMap::new(),
                tee_path: None,
                updated_at: now,
            },
            TaskPaneRecord {
                pane_id: PaneId::new(12),
                workspace: Some("unity-main".to_string()),
                window_id: None,
                tab_id: None,
                remain_on_exit: true,
                silenced: false,
                is_dead: true,
                is_failed: true,
                exit_behavior: Some("CloseOnCleanExit".to_string()),
                current_working_dir: Some("file:///tmp/unity-main".to_string()),
                rerun: HashMap::new(),
                tee_path: None,
                updated_at: now,
            },
        ],
        statuses: vec![WorkspaceStatusRecord {
            workspace: "unity-main".to_string(),
            status: "Building".to_string(),
            updated_at: now,
        }],
        progresses: vec![WorkspaceProgressRecord {
            workspace: "unity-main".to_string(),
            value: 70,
            updated_at: now,
        }],
        logs: HashMap::from([(
            "unity-main".to_string(),
            vec![WorkspaceLogRecord {
                workspace: "unity-main".to_string(),
                seq: 1,
                message: "build started".to_string(),
                created_at: now,
            }],
        )]),
    };

    let snapshot = RuntimeSnapshot::from_source(&source, Some("unity-main"));

    assert_eq!(snapshot.active_workspace, "unity-main");
    assert_eq!(snapshot.notifications.len(), 2);
    assert_eq!(snapshot.task_panes.len(), 2);
    assert_eq!(snapshot.logs.len(), 1);
    let unity = snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.name == "unity-main")
        .expect("unity-main workspace summary");
    assert_eq!(unity.unread_count, 1);
    assert_eq!(unity.running_count, 1);
    assert_eq!(unity.failed_count, 1);
    assert_eq!(unity.status.as_deref(), Some("Building"));
    assert_eq!(unity.progress, Some(70));
    assert_eq!(unity.log_count, 1);
}

#[test]
fn shell_snapshot_context_updates() {
    let client_id = Arc::new(ClientId::new());

    assert_eq!(
        refresh_scope_for_notification(&MuxNotification::NotificationsChanged),
        SnapshotRefreshScope::Notifications
    );
    assert_eq!(
        refresh_scope_for_notification(&MuxNotification::WorkspaceMetadataChanged),
        SnapshotRefreshScope::WorkspaceContext
    );
    assert_eq!(
        refresh_scope_for_notification(&MuxNotification::TaskPaneLifecycleChanged(PaneId::new(7))),
        SnapshotRefreshScope::TaskPanes
    );
    assert_eq!(
        refresh_scope_for_notification(&MuxNotification::ActiveWorkspaceChanged(client_id)),
        SnapshotRefreshScope::WorkspaceList
    );
    assert_eq!(
        refresh_scope_for_notification(&MuxNotification::WindowInvalidated(2)),
        SnapshotRefreshScope::Ignore
    );
}
