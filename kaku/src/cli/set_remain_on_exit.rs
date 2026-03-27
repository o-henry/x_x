use clap::Parser;
use codec::{SetRemainOnExit, SetRemainOnExitResponse, TaskPaneState};
use mux::pane::PaneId;
use serde::Serialize;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SetRemainOnExitCommand {
    #[arg(long = "pane-id")]
    pane_id: PaneId,

    #[arg(long = "remain-on-exit", action = clap::ArgAction::Set)]
    remain_on_exit: bool,
}

impl SetRemainOnExitCommand {
    fn to_request(&self) -> SetRemainOnExit {
        SetRemainOnExit {
            pane_id: self.pane_id,
            remain_on_exit: self.remain_on_exit,
        }
    }

    fn render_json(pane: &TaskPaneState) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct Output<'a> {
            pane_id: PaneId,
            remain_on_exit: bool,
            is_dead: bool,
            is_failed: bool,
            updated_at: &'a str,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&Output {
                pane_id: pane.pane_id,
                remain_on_exit: pane.remain_on_exit,
                is_dead: pane.is_dead,
                is_failed: pane.is_failed,
                updated_at: &pane.updated_at,
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let SetRemainOnExitResponse { pane } = client.set_remain_on_exit(self.to_request()).await?;
        std::io::stdout()
            .lock()
            .write_all(Self::render_json(&pane)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SetRemainOnExitCommand;
    use clap::Parser;
    use codec::TaskPaneState;
    use mux::pane::PaneId;
    use mux::tab::TabId;
    use serde_json::Value;

    fn sample_task_pane() -> TaskPaneState {
        TaskPaneState {
            pane_id: PaneId::new(4),
            workspace: Some("default".to_string()),
            window_id: Some(1),
            tab_id: Some(TabId::new(2)),
            remain_on_exit: true,
            silenced: false,
            is_dead: false,
            is_failed: false,
            rerun_available: false,
            tee_path: None,
            current_working_dir: None,
            updated_at: "2026-03-27T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn lifecycle_contracts_set_remain_on_exit_request_shape() {
        let cmd = SetRemainOnExitCommand::parse_from([
            "kaku",
            "--pane-id",
            "4",
            "--remain-on-exit",
            "true",
        ]);
        let request = cmd.to_request();
        assert_eq!(request.pane_id, PaneId::new(4));
        assert!(request.remain_on_exit);
    }

    #[test]
    fn lifecycle_contracts_set_remain_on_exit_json_shape() {
        let json = SetRemainOnExitCommand::render_json(&sample_task_pane()).expect("json");
        let payload: Value = serde_json::from_str(&json).expect("json payload");
        let keys = payload
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
                "is_dead",
                "is_failed",
                "pane_id",
                "remain_on_exit",
                "updated_at",
            ]
        );
        assert_eq!(payload["pane_id"], 4);
        assert_eq!(payload["remain_on_exit"], true);
        assert!(payload.get("pane").is_none());
    }
}
