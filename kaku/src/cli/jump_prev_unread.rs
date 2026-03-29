use clap::Parser;
use codec::{JumpUnread, JumpUnreadResponse};
use mux::pane::PaneId;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct JumpPrevUnreadCommand {
    /// `--pane-id <id>`
    #[arg(long = "pane-id")]
    pane_id: Option<PaneId>,
}

impl JumpPrevUnreadCommand {
    fn make_request(pane_id: PaneId) -> JumpUnread {
        JumpUnread {
            pane_id,
            direction: "prev".to_string(),
        }
    }

    fn render_response_json(response: &JumpUnreadResponse) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let pane_id = client.resolve_pane_id(self.pane_id).await?;
        let response = client.jump_unread(Self::make_request(pane_id)).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::JumpPrevUnreadCommand;
    use codec::JumpUnreadResponse;
    use mux::pane::PaneId;

    #[test]
    fn notification_discovery_contracts_jump_prev_request_shape() {
        let request = JumpPrevUnreadCommand::make_request(PaneId::from(7usize));
        assert_eq!(request.pane_id, PaneId::from(7usize));
        assert_eq!(request.direction, "prev");
    }

    #[test]
    fn notification_discovery_contracts_jump_prev_json_shape() {
        let json = JumpPrevUnreadCommand::render_response_json(&JumpUnreadResponse {
            pane_id: Some(PaneId::from(7usize)),
        })
        .expect("json");
        assert_eq!(json, "{\n  \"pane_id\": 7\n}\n");
    }
}
