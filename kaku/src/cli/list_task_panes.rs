use crate::cli::CliOutputFormatKind;
use clap::Parser;
use codec::{ListTaskPanes, ListTaskPanesResponse, TaskPaneState};
use mux::pane::PaneId;
use std::io::Write;
use tabout::{tabulate_output, Alignment, Column};
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ListTaskPanesCommand {
    #[arg(long = "pane-id")]
    pane_id: Option<PaneId>,

    #[arg(long = "workspace")]
    workspace: Option<String>,

    #[arg(long = "format", default_value = "table")]
    format: CliOutputFormatKind,
}

impl ListTaskPanesCommand {
    fn to_request(&self) -> ListTaskPanes {
        ListTaskPanes {
            pane_id: self.pane_id,
            workspace: self.workspace.clone(),
        }
    }

    fn render_json(task_panes: &[TaskPaneState]) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(task_panes)?))
    }

    fn render_table<W: std::io::Write>(
        task_panes: &[TaskPaneState],
        out: &mut W,
    ) -> anyhow::Result<()> {
        let cols = vec![
            Column {
                name: "PANE_ID".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "WORKSPACE".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "REMAIN_ON_EXIT".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "SILENCED".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "DEAD".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "FAILED".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "RERUN".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "TEE_PATH".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "UPDATED_AT".to_string(),
                alignment: Alignment::Left,
            },
        ];
        let data = task_panes
            .iter()
            .map(|pane| {
                vec![
                    pane.pane_id.to_string(),
                    pane.workspace.clone().unwrap_or_default(),
                    pane.remain_on_exit.to_string(),
                    pane.silenced.to_string(),
                    pane.is_dead.to_string(),
                    pane.is_failed.to_string(),
                    pane.rerun_available.to_string(),
                    pane.tee_path.clone().unwrap_or_default(),
                    pane.updated_at.clone(),
                ]
            })
            .collect::<Vec<_>>();
        tabulate_output(&cols, &data, out)?;
        Ok(())
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let ListTaskPanesResponse { task_panes } =
            client.list_task_panes(self.to_request()).await?;
        match self.format {
            CliOutputFormatKind::Json => {
                std::io::stdout()
                    .lock()
                    .write_all(Self::render_json(&task_panes)?.as_bytes())?;
            }
            CliOutputFormatKind::Table => {
                Self::render_table(&task_panes, &mut std::io::stdout().lock())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ListTaskPanesCommand;
    use clap::Parser;
    use codec::TaskPaneState;
    use mux::pane::PaneId;
    use mux::tab::TabId;
    use serde_json::Value;

    fn sample_task_pane() -> TaskPaneState {
        TaskPaneState {
            pane_id: PaneId::new(4),
            workspace: Some("unity-main".to_string()),
            window_id: Some(1),
            tab_id: Some(TabId::new(2)),
            remain_on_exit: true,
            silenced: false,
            is_dead: true,
            is_failed: true,
            rerun_available: true,
            tee_path: Some("/tmp/task.log".to_string()),
            current_working_dir: Some("/tmp/project".to_string()),
            updated_at: "2026-03-27T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn lifecycle_contracts_list_task_panes_request_shape() {
        let cmd = ListTaskPanesCommand::parse_from([
            "kaku",
            "--pane-id",
            "4",
            "--workspace",
            "unity-main",
            "--format",
            "json",
        ]);
        let request = cmd.to_request();
        assert_eq!(request.pane_id, Some(PaneId::new(4)));
        assert_eq!(request.workspace.as_deref(), Some("unity-main"));
    }

    #[test]
    fn lifecycle_contracts_list_task_panes_json_shape() {
        let json = ListTaskPanesCommand::render_json(&[sample_task_pane()]).expect("json");
        let payload: Value = serde_json::from_str(&json).expect("json payload");
        let item = &payload.as_array().expect("array payload")[0];
        let keys = item
            .as_object()
            .expect("object payload")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut keys = keys;
        keys.sort();

        assert_eq!(
            keys,
            vec![
                "current_working_dir",
                "is_dead",
                "is_failed",
                "pane_id",
                "remain_on_exit",
                "rerun_available",
                "silenced",
                "tab_id",
                "tee_path",
                "updated_at",
                "window_id",
                "workspace",
            ]
        );
        assert_eq!(item["pane_id"], 4);
        assert_eq!(item["remain_on_exit"], true);
        assert_eq!(item["rerun_available"], true);
        assert_eq!(item["tee_path"], "/tmp/task.log");
    }
}
