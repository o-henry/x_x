use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

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

    pub fn set_status(&mut self, workspace: &str, status: &str) -> bool {
        let now = utc_now();
        match self.status_by_workspace.get_mut(workspace) {
            Some(record) if record.status == status => false,
            Some(record) => {
                record.status = status.to_string();
                record.updated_at = now;
                true
            }
            None => {
                self.status_by_workspace.insert(
                    workspace.to_string(),
                    WorkspaceStatusRecord {
                        workspace: workspace.to_string(),
                        status: status.to_string(),
                        updated_at: now,
                    },
                );
                true
            }
        }
    }

    pub fn clear_status(&mut self, workspace: &str) -> bool {
        self.status_by_workspace.remove(workspace).is_some()
    }

    pub fn list_status(&self) -> Vec<WorkspaceStatusRecord> {
        let mut records = self
            .status_by_workspace
            .values()
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.workspace.cmp(&right.workspace));
        records
    }

    pub fn status_for_workspace(&self, workspace: &str) -> Option<WorkspaceStatusRecord> {
        self.status_by_workspace.get(workspace).cloned()
    }

    pub fn set_progress(
        &mut self,
        workspace: &str,
        value: u8,
    ) -> Result<bool, WorkspaceProgressError> {
        if value > 100 {
            return Err(WorkspaceProgressError::OutOfRange(value));
        }

        let now = utc_now();
        let changed = match self.progress_by_workspace.get_mut(workspace) {
            Some(record) if record.value == value => false,
            Some(record) => {
                record.value = value;
                record.updated_at = now;
                true
            }
            None => {
                self.progress_by_workspace.insert(
                    workspace.to_string(),
                    WorkspaceProgressRecord {
                        workspace: workspace.to_string(),
                        value,
                        updated_at: now,
                    },
                );
                true
            }
        };
        Ok(changed)
    }

    pub fn clear_progress(&mut self, workspace: &str) -> bool {
        self.progress_by_workspace.remove(workspace).is_some()
    }

    pub fn progress_for_workspace(&self, workspace: &str) -> Option<WorkspaceProgressRecord> {
        self.progress_by_workspace.get(workspace).cloned()
    }

    pub fn append_log(&mut self, workspace: &str, message: &str) -> WorkspaceLogRecord {
        self.next_log_seq += 1;
        let record = WorkspaceLogRecord {
            workspace: workspace.to_string(),
            seq: self.next_log_seq,
            message: message.to_string(),
            created_at: utc_now(),
        };

        let logs = self.logs_by_workspace.entry(workspace.to_string()).or_default();
        logs.push(record.clone());
        if logs.len() > WORKSPACE_LOG_CAP {
            let overflow = logs.len() - WORKSPACE_LOG_CAP;
            logs.drain(0..overflow);
        }

        record
    }

    pub fn clear_log(&mut self, workspace: &str) -> bool {
        self.logs_by_workspace.remove(workspace).is_some()
    }

    pub fn list_log(&self, workspace: &str) -> Vec<WorkspaceLogRecord> {
        self.logs_by_workspace
            .get(workspace)
            .cloned()
            .unwrap_or_default()
    }

    pub fn rename_workspace(&mut self, old_workspace: &str, new_workspace: &str) -> bool {
        if old_workspace == new_workspace {
            return false;
        }

        let mut changed = false;
        if let Some(mut status) = self.status_by_workspace.remove(old_workspace) {
            status.workspace = new_workspace.to_string();
            status.updated_at = utc_now();
            self.status_by_workspace
                .insert(new_workspace.to_string(), status);
            changed = true;
        }

        if let Some(mut progress) = self.progress_by_workspace.remove(old_workspace) {
            progress.workspace = new_workspace.to_string();
            progress.updated_at = utc_now();
            self.progress_by_workspace
                .insert(new_workspace.to_string(), progress);
            changed = true;
        }

        if let Some(mut logs) = self.logs_by_workspace.remove(old_workspace) {
            for record in &mut logs {
                record.workspace = new_workspace.to_string();
            }
            self.logs_by_workspace.insert(new_workspace.to_string(), logs);
            changed = true;
        }

        changed
    }
}

fn utc_now() -> DateTime<Utc> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    DateTime::from_timestamp(now.as_secs() as i64, now.subsec_nanos())
        .expect("system time out of range")
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
