use crate::cli::CliOutputFormatKind;
use clap::Parser;
use codec::{ListWorkspaceStatus, ListWorkspaceStatusResponse, WorkspaceStatusState};
use std::io::Write;
use tabout::{tabulate_output, Alignment, Column};
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ListWorkspaceStatusCommand {
    #[arg(long = "workspace")]
    workspace: Option<String>,

    #[arg(long = "format", default_value = "table")]
    format: CliOutputFormatKind,
}

impl ListWorkspaceStatusCommand {
    fn to_request(&self) -> ListWorkspaceStatus {
        ListWorkspaceStatus {
            workspace: self.workspace.clone(),
        }
    }

    fn render_statuses_json(statuses: &[WorkspaceStatusState]) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(statuses)?))
    }

    fn render_table<W: std::io::Write>(
        statuses: &[WorkspaceStatusState],
        out: &mut W,
    ) -> anyhow::Result<()> {
        let cols = vec![
            Column {
                name: "WORKSPACE".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "STATUS".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "UPDATED_AT".to_string(),
                alignment: Alignment::Left,
            },
        ];
        let data = statuses
            .iter()
            .map(|status| {
                vec![
                    status.workspace.clone(),
                    status.status.clone(),
                    status.updated_at.clone(),
                ]
            })
            .collect::<Vec<_>>();
        tabulate_output(&cols, &data, out)?;
        Ok(())
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let ListWorkspaceStatusResponse { statuses } =
            client.list_workspace_status(self.to_request()).await?;
        match self.format {
            CliOutputFormatKind::Json => {
                let mut out = std::io::stdout().lock();
                out.write_all(Self::render_statuses_json(&statuses)?.as_bytes())?;
            }
            CliOutputFormatKind::Table => {
                Self::render_table(&statuses, &mut std::io::stdout().lock())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ListWorkspaceStatusCommand;
    use clap::Parser;
    use codec::WorkspaceStatusState;

    fn sample_status(workspace: &str, status: &str, updated_at: &str) -> WorkspaceStatusState {
        WorkspaceStatusState {
            workspace: workspace.to_string(),
            status: status.to_string(),
            updated_at: updated_at.to_string(),
        }
    }

    #[test]
    fn workspace_metadata_contracts_list_status_request_shape() {
        let cmd = ListWorkspaceStatusCommand::parse_from([
            "kaku",
            "--workspace",
            "unity-main",
            "--format",
            "json",
        ]);

        let request = cmd.to_request();
        assert_eq!(request.workspace.as_deref(), Some("unity-main"));
    }

    #[test]
    fn workspace_metadata_contracts_list_status_json_shape() {
        let json = ListWorkspaceStatusCommand::render_statuses_json(&[
            sample_status("unity-main", "blocked", "2026-03-27T00:00:00Z"),
            sample_status("unity-sidecar", "running", "2026-03-27T00:00:01Z"),
        ])
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "[\n",
                "  {\n",
                "    \"workspace\": \"unity-main\",\n",
                "    \"status\": \"blocked\",\n",
                "    \"updated_at\": \"2026-03-27T00:00:00Z\"\n",
                "  },\n",
                "  {\n",
                "    \"workspace\": \"unity-sidecar\",\n",
                "    \"status\": \"running\",\n",
                "    \"updated_at\": \"2026-03-27T00:00:01Z\"\n",
                "  }\n",
                "]\n"
            )
        );
    }

    #[test]
    fn workspace_metadata_contracts_list_status_table_columns() {
        let mut out = Vec::new();
        ListWorkspaceStatusCommand::render_table(
            &[sample_status(
                "unity-main",
                "blocked",
                "2026-03-27T00:00:00Z",
            )],
            &mut out,
        )
        .expect("table");
        let rendered = String::from_utf8(out).expect("utf8");
        assert!(rendered.contains("WORKSPACE"));
        assert!(rendered.contains("STATUS"));
        assert!(rendered.contains("UPDATED_AT"));
    }
}
