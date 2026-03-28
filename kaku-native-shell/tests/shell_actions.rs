use kaku_native_shell::actions::{
    ShellAction, ShellActionContext, ShellActionOutcome, ShellActionTarget,
};
use mux::Mux;
use std::cell::RefCell;

#[derive(Default)]
struct RecordingTarget {
    calls: RefCell<Vec<String>>,
}

impl RecordingTarget {
    fn take_calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl ShellActionTarget for RecordingTarget {
    fn set_workspace_status(&self, workspace: &str, status: &str) {
        self.calls
            .borrow_mut()
            .push(format!("set-status:{workspace}:{status}"));
    }

    fn clear_workspace_status(&self, workspace: &str) {
        self.calls
            .borrow_mut()
            .push(format!("clear-status:{workspace}"));
    }

    fn set_workspace_progress(&self, workspace: &str, value: u8) {
        self.calls
            .borrow_mut()
            .push(format!("set-progress:{workspace}:{value}"));
    }

    fn clear_workspace_progress(&self, workspace: &str) {
        self.calls
            .borrow_mut()
            .push(format!("clear-progress:{workspace}"));
    }

    fn append_workspace_log(&self, workspace: &str, message: &str) {
        self.calls
            .borrow_mut()
            .push(format!("append-log:{workspace}:{message}"));
    }

    fn mark_notifications_read(&self, notification_ids: &[String]) {
        self.calls.borrow_mut().push(format!(
            "mark-read:{}",
            notification_ids.join(",")
        ));
    }
}

#[test]
fn shell_actions_visible() {
    let actions = ShellAction::operator_actions();
    assert!(actions.contains(&ShellAction::Refresh));
    assert!(actions.contains(&ShellAction::LaunchTerminal));
    assert!(actions.contains(&ShellAction::SetStatus));
    assert!(actions.contains(&ShellAction::ClearStatus));
    assert!(actions.contains(&ShellAction::SetProgress));
    assert!(actions.contains(&ShellAction::ClearProgress));
    assert!(actions.contains(&ShellAction::AppendLog));
    assert!(actions.contains(&ShellAction::MarkVisibleRead));
}

#[test]
fn shell_action_mutations() {
    let target = RecordingTarget::default();
    let context = ShellActionContext {
        workspace: "unity-main".to_string(),
        current_status: None,
        current_progress: None,
        visible_notification_ids: vec!["notif-1".to_string(), "notif-2".to_string()],
        unix_timestamp_secs: 42,
    };

    assert_eq!(
        ShellAction::MarkVisibleRead.execute(&target, &context),
        ShellActionOutcome::Mutated
    );
    assert_eq!(
        ShellAction::AppendLog.execute(&target, &context),
        ShellActionOutcome::Mutated
    );

    let calls = target.take_calls();
    assert_eq!(calls[0], "mark-read:notif-1,notif-2");
    assert_eq!(calls[1], "append-log:unity-main:native-shell check-in 42");
}

#[test]
fn shell_metadata_actions() {
    let mux = Mux::new(None);
    let context = ShellActionContext {
        workspace: "unity-main".to_string(),
        current_status: Some("Building".to_string()),
        current_progress: Some(35),
        visible_notification_ids: Vec::new(),
        unix_timestamp_secs: 100,
    };

    assert_eq!(
        ShellAction::SetStatus.execute(&mux, &context),
        ShellActionOutcome::Mutated
    );
    assert_eq!(
        mux.workspace_status_for_workspace("unity-main")
            .map(|record| record.status),
        Some("Reviewing".to_string())
    );

    assert_eq!(
        ShellAction::SetProgress.execute(&mux, &context),
        ShellActionOutcome::Mutated
    );
    assert_eq!(
        mux.workspace_progress_for_workspace("unity-main")
            .map(|record| record.value),
        Some(70)
    );

    assert_eq!(
        ShellAction::AppendLog.execute(&mux, &context),
        ShellActionOutcome::Mutated
    );
    assert_eq!(
        mux.list_workspace_log("unity-main")
            .last()
            .map(|record| record.message.clone()),
        Some("native-shell check-in 100".to_string())
    );

    assert_eq!(
        ShellAction::ClearStatus.execute(&mux, &context),
        ShellActionOutcome::Mutated
    );
    assert_eq!(mux.workspace_status_for_workspace("unity-main"), None);

    assert_eq!(
        ShellAction::ClearProgress.execute(&mux, &context),
        ShellActionOutcome::Mutated
    );
    assert_eq!(mux.workspace_progress_for_workspace("unity-main"), None);
}
