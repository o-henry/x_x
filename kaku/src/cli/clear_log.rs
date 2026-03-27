use clap::Parser;
use codec::{ClearWorkspaceLog, ClearWorkspaceLogResponse};
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ClearWorkspaceLogCommand {
    #[arg(long = "workspace")]
    workspace: Option<String>,
}

impl ClearWorkspaceLogCommand {
    fn to_request(&self) -> ClearWorkspaceLog {
        ClearWorkspaceLog {
            workspace: self.workspace.clone(),
        }
    }

    fn render_response_json(response: &ClearWorkspaceLogResponse) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(response)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let response = client.clear_workspace_log(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_response_json(&response)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ClearWorkspaceLogCommand;
    use clap::Parser;
    use codec::ClearWorkspaceLogResponse;

    #[test]
    fn workspace_metadata_contracts_clear_log_request_shape() {
        let cmd = ClearWorkspaceLogCommand::parse_from(["kaku", "--workspace", "unity-main"]);

        let request = cmd.to_request();
        assert_eq!(request.workspace.as_deref(), Some("unity-main"));
    }

    #[test]
    fn workspace_metadata_contracts_clear_log_json_shape() {
        let json = ClearWorkspaceLogCommand::render_response_json(&ClearWorkspaceLogResponse {
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
