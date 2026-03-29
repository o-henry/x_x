use clap::Parser;
use codec::GetNotificationCapabilitiesResponse;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct CapabilitiesCommand {}

impl CapabilitiesCommand {
    fn render_response_json(
        response: &GetNotificationCapabilitiesResponse,
    ) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.get_notification_capabilities().await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CapabilitiesCommand;
    use codec::GetNotificationCapabilitiesResponse;

    #[test]
    fn notification_discovery_contracts_capabilities_json_shape() {
        let json =
            CapabilitiesCommand::render_response_json(&GetNotificationCapabilitiesResponse {
                notification_commands: vec![
                    "notify".to_string(),
                    "list-notifications".to_string(),
                    "jump-next-unread".to_string(),
                    "jump-prev-unread".to_string(),
                    "identify".to_string(),
                    "capabilities".to_string(),
                ],
                unread_modes: vec!["clear-on-focus".to_string(), "sticky".to_string()],
                supports_tabbar_markers: true,
            })
            .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"notification_commands\": [\n",
                "    \"notify\",\n",
                "    \"list-notifications\",\n",
                "    \"jump-next-unread\",\n",
                "    \"jump-prev-unread\",\n",
                "    \"identify\",\n",
                "    \"capabilities\"\n",
                "  ],\n",
                "  \"unread_modes\": [\n",
                "    \"clear-on-focus\",\n",
                "    \"sticky\"\n",
                "  ],\n",
                "  \"supports_tabbar_markers\": true\n",
                "}\n"
            )
        );
    }
}
