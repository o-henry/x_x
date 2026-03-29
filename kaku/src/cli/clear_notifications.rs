use clap::Parser;
use codec::{ClearNotifications, ClearNotificationsResponse};
use mux::{pane::PaneId, tab::TabId};
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ClearNotificationsCommand {
    #[arg(long = "notification-id")]
    notification_ids: Vec<String>,

    #[arg(long = "workspace")]
    workspace: Option<String>,

    #[arg(long = "tab-id")]
    tab_id: Option<TabId>,

    #[arg(long = "pane-id")]
    pane_id: Option<PaneId>,
}

impl ClearNotificationsCommand {
    fn to_request(&self) -> ClearNotifications {
        ClearNotifications {
            notification_ids: self.notification_ids.clone(),
            workspace: self.workspace.clone(),
            tab_id: self.tab_id,
            pane_id: self.pane_id,
        }
    }

    fn render_response_json(response: &ClearNotificationsResponse) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.clear_notifications(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ClearNotificationsCommand;
    use clap::Parser;
    use codec::ClearNotificationsResponse;

    #[test]
    fn notification_contracts_clear_response_shape() {
        let json = ClearNotificationsCommand::render_response_json(&ClearNotificationsResponse {
            cleared_count: 2,
            notification_ids: vec!["notif-1".to_string(), "notif-2".to_string()],
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"cleared_count\": 2,\n",
                "  \"notification_ids\": [\n",
                "    \"notif-1\",\n",
                "    \"notif-2\"\n",
                "  ]\n",
                "}\n"
            )
        );
    }

    #[test]
    fn notification_contracts_clear_request_shape() {
        let cmd = ClearNotificationsCommand::parse_from([
            "kaku",
            "--notification-id",
            "notif-1",
            "--notification-id",
            "notif-2",
            "--workspace",
            "unity",
        ]);
        let request = cmd.to_request();
        assert_eq!(request.notification_ids, vec!["notif-1", "notif-2"]);
        assert_eq!(request.workspace.as_deref(), Some("unity"));
        assert_eq!(request.tab_id, None);
        assert_eq!(request.pane_id, None);
    }
}
