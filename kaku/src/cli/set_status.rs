use clap::Parser;
use codec::{SetWorkspaceStatus, SetWorkspaceStatusResponse, WorkspaceStatusState};
use serde::Serialize;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SetWorkspaceStatusCommand {
    #[arg(long = "workspace")]
    workspace: String,

    #[arg(long = "status")]
    status: String,
}

impl SetWorkspaceStatusCommand {
    fn to_request(&self) -> SetWorkspaceStatus {
        SetWorkspaceStatus {
            workspace: self.workspace.clone(),
            status: self.status.clone(),
        }
    }

    fn render_status_json(status: &WorkspaceStatusState) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct SetStatusOutput<'a> {
            workspace: &'a str,
            status: &'a str,
            updated_at: &'a str,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&SetStatusOutput {
                workspace: &status.workspace,
                status: &status.status,
                updated_at: &status.updated_at,
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let SetWorkspaceStatusResponse { status } =
            client.set_workspace_status(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_status_json(&status)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SetWorkspaceStatusCommand;
    use clap::Parser;
    use codec::WorkspaceStatusState;

    #[test]
    fn workspace_metadata_contracts_set_status_request_shape() {
        let cmd = SetWorkspaceStatusCommand::parse_from([
            "kaku",
            "--workspace",
            "unity-main",
            "--status",
            "blocked",
        ]);

        let request = cmd.to_request();
        assert_eq!(request.workspace, "unity-main");
        assert_eq!(request.status, "blocked");
    }

    #[test]
    fn workspace_metadata_contracts_set_status_json_shape() {
        let json = SetWorkspaceStatusCommand::render_status_json(&WorkspaceStatusState {
            workspace: "unity-main".to_string(),
            status: "blocked".to_string(),
            updated_at: "2026-03-27T00:00:00Z".to_string(),
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"workspace\": \"unity-main\",\n",
                "  \"status\": \"blocked\",\n",
                "  \"updated_at\": \"2026-03-27T00:00:00Z\"\n",
                "}\n"
            )
        );
    }
}
