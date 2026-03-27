use clap::{ArgAction, Parser, ValueHint};
use codec::{PipePane, PipePaneResponse};
use mux::pane::PaneId;
use serde::Serialize;
use std::ffi::OsString;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct PipePaneCommand {
    #[arg(long = "pane-id")]
    pane_id: PaneId,

    #[arg(long = "file", value_hint = ValueHint::FilePath, conflicts_with = "disable")]
    file: Option<OsString>,

    #[arg(long = "disable", action = ArgAction::SetTrue)]
    disable: bool,
}

impl PipePaneCommand {
    fn to_request(&self) -> anyhow::Result<PipePane> {
        let file_path = if self.disable {
            None
        } else {
            Some(
                self.file
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("--file is required unless --disable is used"))?
                    .to_string_lossy()
                    .into_owned(),
            )
        };
        Ok(PipePane {
            pane_id: self.pane_id,
            file_path,
        })
    }

    fn render_json(response: &PipePaneResponse) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct Output<'a> {
            pane_id: PaneId,
            tee_path: Option<&'a str>,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&Output {
                pane_id: response.pane_id,
                tee_path: response.tee_path.as_deref(),
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.pipe_pane(self.to_request()?).await?;
        std::io::stdout()
            .lock()
            .write_all(Self::render_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PipePaneCommand;
    use clap::Parser;
    use serde_json::Value;

    #[test]
    fn lifecycle_contracts_pipe_pane_request_shape() {
        let cmd =
            PipePaneCommand::parse_from(["kaku", "--pane-id", "4", "--file", "/tmp/task.log"]);
        let request = cmd.to_request().expect("request");
        assert_eq!(request.pane_id.to_string(), "4");
        assert_eq!(request.file_path.as_deref(), Some("/tmp/task.log"));
    }

    #[test]
    fn lifecycle_contracts_pipe_pane_json_shape() {
        let json = PipePaneCommand::render_json(&codec::PipePaneResponse {
            pane_id: mux::pane::PaneId::new(4),
            tee_path: Some("/tmp/task.log".to_string()),
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

        assert_eq!(keys, vec!["pane_id", "tee_path"]);
        assert_eq!(payload["pane_id"], 4);
        assert_eq!(payload["tee_path"], "/tmp/task.log");
    }
}
