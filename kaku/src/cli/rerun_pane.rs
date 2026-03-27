use clap::Parser;
use codec::{RerunPane, RerunPaneResponse};
use mux::pane::PaneId;
use serde::Serialize;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct RerunPaneCommand {
    #[arg(long = "pane-id")]
    pane_id: PaneId,
}

impl RerunPaneCommand {
    fn to_request(&self) -> RerunPane {
        RerunPane {
            pane_id: self.pane_id,
        }
    }

    fn render_json(response: &RerunPaneResponse) -> anyhow::Result<String> {
        anyhow::ensure!(
            response.status == "rerun",
            "unexpected rerun-pane status `{}`",
            response.status
        );

        #[derive(Serialize)]
        struct Output<'a> {
            pane_id: PaneId,
            spawned_pane_id: PaneId,
            status: &'a str,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&Output {
                pane_id: response.pane_id,
                spawned_pane_id: response.spawned_pane_id,
                status: &response.status,
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.rerun_pane(self.to_request()).await?;
        std::io::stdout()
            .lock()
            .write_all(Self::render_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RerunPaneCommand;
    use clap::Parser;
    use codec::RerunPaneResponse;
    use mux::pane::PaneId;
    use serde_json::Value;

    #[test]
    fn lifecycle_contracts_rerun_pane_request_shape() {
        let cmd = RerunPaneCommand::parse_from(["kaku", "--pane-id", "4"]);
        let request = cmd.to_request();
        assert_eq!(request.pane_id, PaneId::new(4));
    }

    #[test]
    fn lifecycle_contracts_rerun_pane_json_shape() {
        let json = RerunPaneCommand::render_json(&RerunPaneResponse {
            pane_id: PaneId::new(4),
            spawned_pane_id: PaneId::new(8),
            status: "rerun".to_string(),
        })
        .expect("json");
        let payload: Value = serde_json::from_str(&json).expect("json payload");
        let keys = payload
            .as_object()
            .expect("object payload")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut keys = keys;
        keys.sort();

        assert_eq!(keys, vec!["pane_id", "spawned_pane_id", "status"]);
        assert_eq!(payload["spawned_pane_id"], 8);
        assert_eq!(payload["status"], "rerun");
        assert!(payload.get("response").is_none());
    }

    #[test]
    fn lifecycle_contracts_rerun_pane_rejects_unexpected_status() {
        let err = RerunPaneCommand::render_json(&RerunPaneResponse {
            pane_id: PaneId::new(4),
            spawned_pane_id: PaneId::new(8),
            status: "respawn".to_string(),
        })
        .expect_err("unexpected status should fail");

        assert!(err.to_string().contains("unexpected rerun-pane status"));
    }
}
