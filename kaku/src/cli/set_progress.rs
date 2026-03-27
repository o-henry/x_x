use clap::Parser;
use codec::{SetWorkspaceProgress, SetWorkspaceProgressResponse, WorkspaceProgressState};
use serde::Serialize;
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct SetWorkspaceProgressCommand {
    #[arg(long = "workspace")]
    workspace: String,

    #[arg(long = "value")]
    value: u8,
}

impl SetWorkspaceProgressCommand {
    fn to_request(&self) -> SetWorkspaceProgress {
        SetWorkspaceProgress {
            workspace: self.workspace.clone(),
            value: self.value,
        }
    }

    fn render_progress_json(progress: &WorkspaceProgressState) -> anyhow::Result<String> {
        #[derive(Serialize)]
        struct SetProgressOutput<'a> {
            workspace: &'a str,
            value: u8,
            updated_at: &'a str,
        }

        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&SetProgressOutput {
                workspace: &progress.workspace,
                value: progress.value,
                updated_at: &progress.updated_at,
            })?
        ))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let SetWorkspaceProgressResponse { progress } =
            client.set_workspace_progress(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_progress_json(&progress)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SetWorkspaceProgressCommand;
    use clap::Parser;
    use codec::WorkspaceProgressState;

    #[test]
    fn workspace_metadata_contracts_set_progress_request_shape() {
        let cmd = SetWorkspaceProgressCommand::parse_from([
            "kaku",
            "--workspace",
            "unity-main",
            "--value",
            "37",
        ]);

        let request = cmd.to_request();
        assert_eq!(request.workspace, "unity-main");
        assert_eq!(request.value, 37);
    }

    #[test]
    fn workspace_metadata_contracts_set_progress_json_shape() {
        let json = SetWorkspaceProgressCommand::render_progress_json(&WorkspaceProgressState {
            workspace: "unity-main".to_string(),
            value: 37,
            updated_at: "2026-03-27T00:00:00Z".to_string(),
        })
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"workspace\": \"unity-main\",\n",
                "  \"value\": 37,\n",
                "  \"updated_at\": \"2026-03-27T00:00:00Z\"\n",
                "}\n"
            )
        );
    }
}
