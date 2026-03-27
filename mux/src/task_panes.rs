use crate::pane::PaneId;
use crate::tab::TabId;
use crate::window::WindowId;
use chrono::{DateTime, Utc};
use config::ExitBehavior;
use portable_pty::CommandBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskPaneRecord {
    pub pane_id: PaneId,
    pub workspace: Option<String>,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub remain_on_exit: bool,
    pub silenced: bool,
    pub is_dead: bool,
    pub is_failed: bool,
    pub exit_behavior: Option<String>,
    pub current_working_dir: Option<String>,
    pub rerun: HashMap<String, String>,
    pub tee_path: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPaneExitRecord {
    pub pane_id: PaneId,
    pub workspace: Option<String>,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub remain_on_exit: bool,
    pub is_failed: bool,
    pub exit_behavior: ExitBehavior,
    pub current_working_dir: Option<String>,
    pub rerun: HashMap<String, String>,
}

#[derive(Default)]
pub struct TaskPaneStore {
    records: HashMap<PaneId, TaskPaneRecord>,
}

impl TaskPaneStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_exit(&mut self, exit: TaskPaneExitRecord) -> TaskPaneRecord {
        let now = utc_now();
        let silenced = self
            .records
            .get(&exit.pane_id)
            .map(|record| record.silenced)
            .unwrap_or(false);
        let tee_path = self
            .records
            .get(&exit.pane_id)
            .and_then(|record| record.tee_path.clone());

        let record = TaskPaneRecord {
            pane_id: exit.pane_id,
            workspace: exit.workspace,
            window_id: exit.window_id,
            tab_id: exit.tab_id,
            remain_on_exit: exit.remain_on_exit,
            silenced,
            is_dead: true,
            is_failed: exit.is_failed,
            exit_behavior: Some(format!("{:?}", exit.exit_behavior)),
            current_working_dir: exit.current_working_dir,
            rerun: exit.rerun,
            tee_path,
            updated_at: now,
        };
        self.records.insert(exit.pane_id, record.clone());
        record
    }

    pub fn upsert_live(
        &mut self,
        pane_id: PaneId,
        workspace: Option<String>,
        window_id: Option<WindowId>,
        tab_id: Option<TabId>,
        remain_on_exit: bool,
        silenced: bool,
    ) -> TaskPaneRecord {
        let mut record = self.records.get(&pane_id).cloned().unwrap_or(TaskPaneRecord {
            pane_id,
            workspace: None,
            window_id: None,
            tab_id: None,
            remain_on_exit: false,
            silenced: false,
            is_dead: false,
            is_failed: false,
            exit_behavior: None,
            current_working_dir: None,
            rerun: HashMap::new(),
            tee_path: None,
            updated_at: utc_now(),
        });

        record.workspace = workspace;
        record.window_id = window_id;
        record.tab_id = tab_id;
        record.remain_on_exit = remain_on_exit;
        record.silenced = silenced;
        record.is_dead = false;
        record.is_failed = false;
        record.exit_behavior = None;
        record.updated_at = utc_now();
        self.records.insert(pane_id, record.clone());
        record
    }

    pub fn refresh_live_metadata(
        &mut self,
        pane_id: PaneId,
        current_working_dir: Option<String>,
        rerun: HashMap<String, String>,
    ) -> Option<TaskPaneRecord> {
        let record = self.records.get_mut(&pane_id)?;
        record.current_working_dir = current_working_dir;
        record.rerun = rerun;
        record.updated_at = utc_now();
        Some(record.clone())
    }

    pub fn set_silenced(&mut self, pane_id: PaneId, silenced: bool) -> bool {
        match self.records.get_mut(&pane_id) {
            Some(record) if record.silenced == silenced => false,
            Some(record) => {
                record.silenced = silenced;
                record.updated_at = utc_now();
                true
            }
            None => false,
        }
    }

    pub fn set_tee_path(&mut self, pane_id: PaneId, tee_path: Option<String>) -> bool {
        match self.records.get_mut(&pane_id) {
            Some(record) if record.tee_path == tee_path => false,
            Some(record) => {
                record.tee_path = tee_path;
                record.updated_at = utc_now();
                true
            }
            None => false,
        }
    }

    pub fn list_task_panes(&self) -> Vec<TaskPaneRecord> {
        let mut records = self.records.values().cloned().collect::<Vec<_>>();
        records.sort_by_key(|record| record.pane_id);
        records
    }

    pub fn task_pane(&self, pane_id: PaneId) -> Option<TaskPaneRecord> {
        self.records.get(&pane_id).cloned()
    }

    pub fn remove_task_pane(&mut self, pane_id: PaneId) -> bool {
        self.records.remove(&pane_id).is_some()
    }

    pub fn prune_missing(&mut self, live_panes: &HashSet<PaneId>) -> usize {
        let before = self.records.len();
        self.records
            .retain(|pane_id, _| live_panes.contains(pane_id));
        before.saturating_sub(self.records.len())
    }
}

pub fn rerun_metadata_from_user_vars(user_vars: &HashMap<String, String>) -> HashMap<String, String> {
    let mut rerun = user_vars
        .iter()
        .filter(|(key, _)| is_rerun_user_var(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<HashMap<_, _>>();
    let mut keys = rerun.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys.into_iter()
        .filter_map(|key| rerun.remove(&key).map(|value| (key, value)))
        .collect()
}

pub fn rerun_metadata_from_command_builder(cmd: &CommandBuilder) -> HashMap<String, String> {
    let mut rerun = cmd
        .iter_full_env_as_str()
        .filter(|(key, _)| is_rerun_user_var(key))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<HashMap<_, _>>();
    if rerun.is_empty() {
        let argv = cmd
            .get_argv()
            .iter()
            .filter_map(|arg| arg.to_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        if !argv.is_empty() {
            if let Ok(joined) = shlex::try_join(argv.iter().map(|arg| arg.as_str())) {
                rerun.insert("KAKU_RERUN_COMMAND".to_string(), joined);
            }
        }
    }
    let mut keys = rerun.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys.into_iter()
        .filter_map(|key| rerun.remove(&key).map(|value| (key, value)))
        .collect()
}

fn is_rerun_user_var(key: &str) -> bool {
    matches!(key, "KAKU_RERUN_COMMAND" | "KAKU_RERUN_PAYLOAD")
        || key.starts_with("KAKU_RERUN_")
        || key.starts_with("kaku.rerun.")
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
    fn record_exit_persists_remain_on_exit_failure_and_rerun_metadata() {
        let mut store = TaskPaneStore::new();
        let pane_id = PaneId::new(7);
        let mut rerun = HashMap::new();
        rerun.insert("KAKU_RERUN_COMMAND".to_string(), "make test".to_string());
        rerun.insert("kaku.rerun.cwd".to_string(), "/tmp/project".to_string());

        let record = store.record_exit(TaskPaneExitRecord {
            pane_id,
            workspace: Some("unity-main".to_string()),
            window_id: Some(3),
            tab_id: Some(TabId::new(4)),
            remain_on_exit: true,
            is_failed: true,
            exit_behavior: ExitBehavior::Hold,
            current_working_dir: Some("file:///tmp/project".to_string()),
            rerun: rerun.clone(),
        });

        assert_eq!(record.pane_id, pane_id);
        assert_eq!(record.workspace.as_deref(), Some("unity-main"));
        assert_eq!(record.window_id, Some(3));
        assert_eq!(record.tab_id, Some(TabId::new(4)));
        assert!(record.remain_on_exit);
        assert!(record.is_dead);
        assert!(record.is_failed);
        assert_eq!(record.exit_behavior.as_deref(), Some("Hold"));
        assert_eq!(record.current_working_dir.as_deref(), Some("file:///tmp/project"));
        assert_eq!(record.rerun, rerun);
    }

    #[test]
    fn record_exit_replaces_existing_pane_entry_without_duplicates() {
        let mut store = TaskPaneStore::new();
        let pane_id = PaneId::new(9);

        store.record_exit(TaskPaneExitRecord {
            pane_id,
            workspace: Some("default".to_string()),
            window_id: Some(1),
            tab_id: Some(TabId::new(1)),
            remain_on_exit: false,
            is_failed: true,
            exit_behavior: ExitBehavior::CloseOnCleanExit,
            current_working_dir: None,
            rerun: HashMap::new(),
        });
        let first = store.task_pane(pane_id).expect("task-pane record");

        store.record_exit(TaskPaneExitRecord {
            pane_id,
            workspace: Some("default".to_string()),
            window_id: Some(1),
            tab_id: Some(TabId::new(1)),
            remain_on_exit: true,
            is_failed: false,
            exit_behavior: ExitBehavior::Hold,
            current_working_dir: Some("file:///tmp".to_string()),
            rerun: HashMap::new(),
        });

        let records = store.list_task_panes();
        assert_eq!(records.len(), 1);
        assert!(records[0].updated_at >= first.updated_at);
        assert!(records[0].remain_on_exit);
        assert!(!records[0].is_failed);
    }

    #[test]
    fn prune_missing_and_remove_task_pane_are_explicit() {
        let mut store = TaskPaneStore::new();
        let first = PaneId::new(1);
        let second = PaneId::new(2);

        for pane_id in [first, second] {
            store.record_exit(TaskPaneExitRecord {
                pane_id,
                workspace: Some("default".to_string()),
                window_id: Some(0),
                tab_id: Some(TabId::new(0)),
                remain_on_exit: false,
                is_failed: true,
                exit_behavior: ExitBehavior::CloseOnCleanExit,
                current_working_dir: None,
                rerun: HashMap::new(),
            });
        }

        let mut live = HashSet::new();
        live.insert(second);
        assert_eq!(store.prune_missing(&live), 1);
        assert!(store.task_pane(first).is_none());
        assert!(store.task_pane(second).is_some());
        assert!(store.remove_task_pane(second));
        assert!(store.list_task_panes().is_empty());
    }

    #[test]
    fn prune_missing_keeps_dead_records_with_retention_signals() {
        let mut store = TaskPaneStore::new();
        let failed = PaneId::new(3);
        let rerunnable = PaneId::new(4);
        let stale_live = PaneId::new(5);

        store.record_exit(TaskPaneExitRecord {
            pane_id: failed,
            workspace: Some("default".to_string()),
            window_id: Some(0),
            tab_id: Some(TabId::new(0)),
            remain_on_exit: false,
            is_failed: true,
            exit_behavior: ExitBehavior::CloseOnCleanExit,
            current_working_dir: None,
            rerun: HashMap::new(),
        });

        let mut rerun = HashMap::new();
        rerun.insert("KAKU_RERUN_COMMAND".to_string(), "cargo test".to_string());
        store.record_exit(TaskPaneExitRecord {
            pane_id: rerunnable,
            workspace: Some("default".to_string()),
            window_id: Some(0),
            tab_id: Some(TabId::new(0)),
            remain_on_exit: false,
            is_failed: false,
            exit_behavior: ExitBehavior::CloseOnCleanExit,
            current_working_dir: None,
            rerun,
        });
        store.upsert_live(
            stale_live,
            Some("default".to_string()),
            Some(0),
            Some(TabId::new(0)),
            false,
            false,
        );

        assert_eq!(store.prune_missing(&HashSet::new()), 1);
        assert!(store.task_pane(failed).is_some(), "failed panes should be retained");
        assert!(
            store.task_pane(rerunnable).is_some(),
            "rerunnable panes should be retained"
        );
        assert!(
            store.task_pane(stale_live).is_none(),
            "live panes missing from mux should be pruned"
        );
    }

    #[test]
    fn rerun_metadata_filter_keeps_only_rerun_keys() {
        let user_vars = HashMap::from([
            ("KAKU_RERUN_COMMAND".to_string(), "cargo test".to_string()),
            ("kaku.rerun.payload".to_string(), "{\"a\":1}".to_string()),
            ("TERM".to_string(), "xterm-256color".to_string()),
        ]);

        let rerun = rerun_metadata_from_user_vars(&user_vars);

        assert_eq!(rerun.len(), 2);
        assert_eq!(
            rerun.get("KAKU_RERUN_COMMAND").map(String::as_str),
            Some("cargo test")
        );
        assert_eq!(
            rerun.get("kaku.rerun.payload").map(String::as_str),
            Some("{\"a\":1}")
        );
        assert!(!rerun.contains_key("TERM"));
    }

    #[test]
    fn rerun_metadata_filter_keeps_only_rerun_env_keys() {
        let mut cmd = CommandBuilder::from_argv(vec![
            "/bin/sh".into(),
            "-lc".into(),
            "false".into(),
        ]);
        cmd.env("KAKU_RERUN_COMMAND", "cargo test");
        cmd.env("kaku.rerun.payload", "{\"a\":1}");
        cmd.env("TERM", "xterm-256color");

        let rerun = rerun_metadata_from_command_builder(&cmd);

        assert_eq!(rerun.len(), 2);
        assert_eq!(
            rerun.get("KAKU_RERUN_COMMAND").map(String::as_str),
            Some("cargo test")
        );
        assert_eq!(
            rerun.get("kaku.rerun.payload").map(String::as_str),
            Some("{\"a\":1}")
        );
        assert!(!rerun.contains_key("TERM"));
    }

    #[test]
    fn rerun_metadata_from_command_builder_falls_back_to_joined_argv() {
        let cmd = CommandBuilder::from_argv(vec![
            "/bin/sh".into(),
            "-lc".into(),
            "echo PHASE4_FAILING_TASK; false".into(),
        ]);

        let rerun = rerun_metadata_from_command_builder(&cmd);

        assert_eq!(
            rerun.get("KAKU_RERUN_COMMAND").map(String::as_str),
            Some("/bin/sh -lc 'echo PHASE4_FAILING_TASK; false'")
        );
    }

    #[test]
    fn upsert_live_updates_intent_without_marking_pane_dead() {
        let mut store = TaskPaneStore::new();
        let pane_id = PaneId::new(11);

        let record = store.upsert_live(
            pane_id,
            Some("default".to_string()),
            Some(0),
            Some(TabId::new(0)),
            true,
            true,
        );

        assert_eq!(record.workspace.as_deref(), Some("default"));
        assert!(record.remain_on_exit);
        assert!(record.silenced);
        assert!(!record.is_dead);
        assert!(!record.is_failed);
    }
}
