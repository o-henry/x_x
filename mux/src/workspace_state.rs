use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const WORKSPACE_LOG_CAP: usize = 200;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceStatusRecord {
    pub workspace: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceProgressRecord {
    pub workspace: String,
    pub value: u8,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceLogRecord {
    pub workspace: String,
    pub seq: u64,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorkspaceProgressError {
    OutOfRange(u8),
}

#[derive(Default)]
pub struct WorkspaceStateStore {
    status_by_workspace: HashMap<String, WorkspaceStatusRecord>,
    progress_by_workspace: HashMap<String, WorkspaceProgressRecord>,
    logs_by_workspace: HashMap<String, Vec<WorkspaceLogRecord>>,
    next_log_seq: u64,
}

impl WorkspaceStateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_status(&mut self, _workspace: &str, _status: &str) -> bool {
        false
    }

    pub fn clear_status(&mut self, _workspace: &str) -> bool {
        false
    }

    pub fn list_status(&self) -> Vec<WorkspaceStatusRecord> {
        vec![]
    }

    pub fn status_for_workspace(&self, _workspace: &str) -> Option<WorkspaceStatusRecord> {
        None
    }

    pub fn set_progress(
        &mut self,
        _workspace: &str,
        _value: u8,
    ) -> Result<bool, WorkspaceProgressError> {
        Ok(false)
    }

    pub fn clear_progress(&mut self, _workspace: &str) -> bool {
        false
    }

    pub fn progress_for_workspace(&self, _workspace: &str) -> Option<WorkspaceProgressRecord> {
        None
    }

    pub fn append_log(&mut self, _workspace: &str, _message: &str) -> WorkspaceLogRecord {
        unimplemented!()
    }

    pub fn clear_log(&mut self, _workspace: &str) -> bool {
        false
    }

    pub fn list_log(&self, _workspace: &str) -> Vec<WorkspaceLogRecord> {
        vec![]
    }

    pub fn rename_workspace(&mut self, _old_workspace: &str, _new_workspace: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_status_replaces_existing_record_without_duplicates() {
        let mut store = WorkspaceStateStore::new();

        assert!(store.set_status("unity-main", "idle"));
        let first = store
            .status_for_workspace("unity-main")
            .expect("status record");

        assert!(store.set_status("unity-main", "running"));

        let statuses = store.list_status();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].workspace, "unity-main");
        assert_eq!(statuses[0].status, "running");
        assert!(statuses[0].updated_at >= first.updated_at);
    }

    #[test]
    fn set_progress_rejects_out_of_range_and_clear_removes_record() {
        let mut store = WorkspaceStateStore::new();

        assert!(store.set_progress("unity-main", 0).expect("0 should work"));
        assert!(store
            .set_progress("unity-main", 100)
            .expect("100 should work"));
        assert_eq!(
            store.set_progress("unity-main", 101),
            Err(WorkspaceProgressError::OutOfRange(101))
        );

        let progress = store
            .progress_for_workspace("unity-main")
            .expect("progress record");
        assert_eq!(progress.value, 100);

        assert!(store.clear_progress("unity-main"));
        assert_eq!(store.progress_for_workspace("unity-main"), None);
    }

    #[test]
    fn append_log_truncates_oldest_entries_and_clear_is_workspace_scoped() {
        let mut store = WorkspaceStateStore::new();

        for idx in 0..=WORKSPACE_LOG_CAP {
            store.append_log("unity-main", &format!("main-{idx}"));
        }
        store.append_log("unity-sidecar", "other");

        let main_logs = store.list_log("unity-main");
        assert_eq!(main_logs.len(), WORKSPACE_LOG_CAP);
        assert_eq!(main_logs.first().map(|record| record.message.as_str()), Some("main-1"));
        assert_eq!(
            main_logs.last().map(|record| record.message.as_str()),
            Some("main-200")
        );
        assert!(main_logs.windows(2).all(|pair| pair[0].seq < pair[1].seq));

        assert!(store.clear_log("unity-main"));
        assert!(store.list_log("unity-main").is_empty());
        assert_eq!(store.list_log("unity-sidecar").len(), 1);
    }

    #[test]
    fn rename_workspace_moves_all_metadata_without_losing_values() {
        let mut store = WorkspaceStateStore::new();
        store.set_status("old-name", "running");
        store.set_progress("old-name", 42).expect("progress should set");
        let first = store.append_log("old-name", "first");
        let second = store.append_log("old-name", "second");

        assert!(store.rename_workspace("old-name", "new-name"));

        assert_eq!(store.status_for_workspace("old-name"), None);
        assert_eq!(store.progress_for_workspace("old-name"), None);
        assert!(store.list_log("old-name").is_empty());

        let status = store
            .status_for_workspace("new-name")
            .expect("renamed status");
        assert_eq!(status.status, "running");

        let progress = store
            .progress_for_workspace("new-name")
            .expect("renamed progress");
        assert_eq!(progress.value, 42);

        let logs = store.list_log("new-name");
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].workspace, "new-name");
        assert_eq!(logs[0].seq, first.seq);
        assert_eq!(logs[0].message, "first");
        assert_eq!(logs[1].workspace, "new-name");
        assert_eq!(logs[1].seq, second.seq);
        assert_eq!(logs[1].message, "second");
    }
}
