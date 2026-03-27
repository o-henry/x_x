use clap::Parser;
use codec::{SilenceWatchdog, SilenceWatchdogResponse};
use mux::pane::PaneId;
use serde::Serialize;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SilenceWatchdogCommand {
    #[arg(long = "pane-id")]
    pane_id: PaneId,

    #[arg(long = "silenced", action = clap::ArgAction::Set)]
    silenced: bool,
}

impl SilenceWatchdogCommand {
    fn to_request(&self) -> SilenceWatchdog {
        SilenceWatchdog {
            pane_id: self.pane_id,
            silenced: self.silenced,
        }
    }

    fn render_json(response: &SilenceWatchdogResponse) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct Output {
            pane_id: PaneId,
            silenced: bool,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&Output {
                pane_id: response.pane_id,
                silenced: response.silenced,
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.silence_watchdog(self.to_request()).await?;
        std::io::stdout()
            .lock()
            .write_all(Self::render_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SilenceWatchdogCommand;
    use clap::Parser;
    use serde_json::Value;

    #[test]
    fn lifecycle_contracts_silence_watchdog_request_shape() {
        let cmd =
            SilenceWatchdogCommand::parse_from(["kaku", "--pane-id", "4", "--silenced", "true"]);
        let request = cmd.to_request();
        assert_eq!(request.pane_id.to_string(), "4");
        assert!(request.silenced);
    }

    #[test]
    fn lifecycle_contracts_silence_watchdog_json_shape() {
        let json = SilenceWatchdogCommand::render_json(&codec::SilenceWatchdogResponse {
            pane_id: mux::pane::PaneId::new(4),
            silenced: true,
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

        assert_eq!(keys, vec!["pane_id", "silenced"]);
        assert_eq!(payload["pane_id"], 4);
        assert_eq!(payload["silenced"], true);
    }
}
