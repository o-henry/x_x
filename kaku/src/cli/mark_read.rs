use clap::Parser;
use codec::{MarkNotificationsRead, MarkNotificationsResponse};
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct MarkReadCommand {
    #[arg(long = "notification-id")]
    notification_ids: Vec<String>,
}

impl MarkReadCommand {
    fn to_request(&self) -> MarkNotificationsRead {
        MarkNotificationsRead {
            notification_ids: self.notification_ids.clone(),
        }
    }

    fn render_response_json(response: &MarkNotificationsResponse) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.mark_notifications_read(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MarkReadCommand;
    use clap::Parser;
    use codec::MarkNotificationsResponse;

    #[test]
    fn notification_contracts_mark_read_response_shape() {
        let json = MarkReadCommand::render_response_json(&MarkNotificationsResponse {
            updated_count: 1,
            notification_ids: vec!["notif-1".to_string()],
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"updated_count\": 1,\n",
                "  \"notification_ids\": [\n",
                "    \"notif-1\"\n",
                "  ]\n",
                "}\n"
            )
        );
    }

    #[test]
    fn notification_contracts_mark_read_request_shape() {
        let cmd = MarkReadCommand::parse_from([
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
