use clap::Parser;
use codec::{MarkNotificationsResponse, MarkNotificationsUnread};
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct MarkUnreadCommand {
    #[arg(long = "notification-id")]
    notification_ids: Vec<String>,
}

impl MarkUnreadCommand {
    fn to_request(&self) -> MarkNotificationsUnread {
        MarkNotificationsUnread {
            notification_ids: self.notification_ids.clone(),
        }
    }

    fn render_response_json(response: &MarkNotificationsResponse) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.mark_notifications_unread(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MarkUnreadCommand;
    use clap::Parser;
    use codec::MarkNotificationsResponse;

    #[test]
    fn notification_contracts_mark_unread_response_shape() {
        let json = MarkUnreadCommand::render_response_json(&MarkNotificationsResponse {
            updated_count: 2,
            notification_ids: vec!["notif-1".to_string(), "notif-2".to_string()],
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"updated_count\": 2,\n",
                "  \"notification_ids\": [\n",
                "    \"notif-1\",\n",
                "    \"notif-2\"\n",
                "  ]\n",
                "}\n"
            )
        );
    }

    #[test]
    fn notification_contracts_mark_unread_request_shape() {
        let cmd = MarkUnreadCommand::parse_from([
            "kaku",
            "--notification-id",
            "notif-1",
            "--notification-id",
            "notif-2",
        ]);
        let request = cmd.to_request();
        assert_eq!(request.notification_ids, vec!["notif-1", "notif-2"]);
    }
}
