use clap::Parser;
use codec::{AppendWorkspaceLog, AppendWorkspaceLogResponse, WorkspaceLogState};
use serde::Serialize;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct WorkspaceLogCommand {
    #[arg(long = "workspace")]
    workspace: String,

    #[arg(long = "message")]
    message: String,
}

impl WorkspaceLogCommand {
    fn to_request(&self) -> AppendWorkspaceLog {
        AppendWorkspaceLog {
            workspace: self.workspace.clone(),
            message: self.message.clone(),
        }
    }

    fn render_entry_json(entry: &WorkspaceLogState) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct LogOutput<'a> {
            workspace: &'a str,
            seq: u64,
            message: &'a str,
            created_at: &'a str,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&LogOutput {
                workspace: &entry.workspace,
                seq: entry.seq,
                message: &entry.message,
                created_at: &entry.created_at,
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let AppendWorkspaceLogResponse { entry } =
            client.append_workspace_log(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_entry_json(&entry)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceLogCommand;
    use clap::Parser;
    use codec::WorkspaceLogState;

    #[test]
    fn workspace_metadata_contracts_log_request_shape() {
        let cmd = WorkspaceLogCommand::parse_from([
            "kaku",
            "--workspace",
            "unity-main",
            "--message",
            "build failed",
        ]);

        let request = cmd.to_request();
        assert_eq!(request.workspace, "unity-main");
        assert_eq!(request.message, "build failed");
    }

    #[test]
    fn workspace_metadata_contracts_log_json_shape() {
        let json = WorkspaceLogCommand::render_entry_json(&WorkspaceLogState {
            workspace: "unity-main".to_string(),
            seq: 7,
            message: "build failed".to_string(),
            created_at: "2026-03-27T00:00:00.000Z".to_string(),
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"workspace\": \"unity-main\",\n",
                "  \"seq\": 7,\n",
                "  \"message\": \"build failed\",\n",
                "  \"created_at\": \"2026-03-27T00:00:00.000Z\"\n",
                "}\n"
            )
        );
    }
}
