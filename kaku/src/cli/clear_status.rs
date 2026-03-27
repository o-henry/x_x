use clap::Parser;
use codec::{ClearWorkspaceStatus, ClearWorkspaceStatusResponse};
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ClearWorkspaceStatusCommand {
    #[arg(long = "workspace")]
    workspace: Option<String>,
}

impl ClearWorkspaceStatusCommand {
    fn to_request(&self) -> ClearWorkspaceStatus {
        ClearWorkspaceStatus {
            workspace: self.workspace.clone(),
        }
    }

    fn render_response_json(response: &ClearWorkspaceStatusResponse) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.clear_workspace_status(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ClearWorkspaceStatusCommand;
    use clap::Parser;
    use codec::ClearWorkspaceStatusResponse;

    #[test]
    fn workspace_metadata_contracts_clear_status_request_shape() {
        let cmd = ClearWorkspaceStatusCommand::parse_from(["kaku", "--workspace", "unity-main"]);

        let request = cmd.to_request();
        assert_eq!(request.workspace.as_deref(), Some("unity-main"));
    }

    #[test]
    fn workspace_metadata_contracts_clear_status_json_shape() {
        let json =
            ClearWorkspaceStatusCommand::render_response_json(&ClearWorkspaceStatusResponse {
                cleared_count: 1,
                workspaces: vec!["unity-main".to_string()],
            })
            .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"cleared_count\": 1,\n",
                "  \"workspaces\": [\n",
                "    \"unity-main\"\n",
                "  ]\n",
                "}\n"
            )
        );
    }
}
