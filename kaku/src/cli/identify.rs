use clap::Parser;
use codec::{IdentifyNotificationTarget, IdentifyNotificationTargetResponse};
use mux::pane::PaneId;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct IdentifyCommand {
    /// `--pane-id <id>`
    #[arg(long = "pane-id")]
    pane_id: Option<PaneId>,
}

impl IdentifyCommand {
    fn make_request(pane_id: PaneId) -> IdentifyNotificationTarget {
        IdentifyNotificationTarget { pane_id }
    }

    fn render_response_json(
        response: &IdentifyNotificationTargetResponse,
    ) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let pane_id = client.resolve_pane_id(self.pane_id).await?;
        let response = client
            .identify_notification_target(Self::make_request(pane_id))
            .await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::IdentifyCommand;
    use codec::IdentifyNotificationTargetResponse;
    use mux::{pane::PaneId, tab::TabId, window::WindowId};

    #[test]
    fn notification_discovery_contracts_identify_request_shape() {
        let request = IdentifyCommand::make_request(PaneId::from(5usize));
        assert_eq!(request.pane_id, PaneId::from(5usize));
    }

    #[test]
    fn notification_discovery_contracts_identify_json_shape() {
        let json = IdentifyCommand::render_response_json(&IdentifyNotificationTargetResponse {
            workspace: "unity".to_string(),
            window_id: WindowId::from(11usize),
            tab_id: TabId::from(22usize),
            pane_id: PaneId::from(33usize),
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"workspace\": \"unity\",\n",
                "  \"window_id\": 11,\n",
                "  \"tab_id\": 22,\n",
                "  \"pane_id\": 33\n",
                "}\n"
            )
        );
    }
}
