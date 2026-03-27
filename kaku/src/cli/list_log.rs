use crate::cli::CliOutputFormatKind;
use clap::Parser;
use codec::{ListWorkspaceLog, ListWorkspaceLogResponse, WorkspaceLogState};
use std::io::Write;
use tabout::{tabulate_output, Alignment, Column};
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ListWorkspaceLogCommand {
    #[arg(long = "workspace")]
    workspace: Option<String>,

    #[arg(long = "limit")]
    limit: Option<usize>,

    #[arg(long = "format", default_value = "table")]
    format: CliOutputFormatKind,
}

impl ListWorkspaceLogCommand {
    fn to_request(&self) -> ListWorkspaceLog {
        ListWorkspaceLog {
            workspace: self.workspace.clone(),
            limit: self.limit,
        }
    }

    fn render_entries_json(entries: &[WorkspaceLogState]) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(entries)?))
    }

    fn render_table<W: std::io::Write>(
        entries: &[WorkspaceLogState],
        out: &mut W,
    ) -> anyhow::Result<()> {
        let cols = vec![
            Column {
                name: "WORKSPACE".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "SEQ".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "MESSAGE".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "CREATED_AT".to_string(),
                alignment: Alignment::Left,
            },
        ];
        let data = entries
            .iter()
            .map(|entry| {
                vec![
                    entry.workspace.clone(),
                    entry.seq.to_string(),
                    entry.message.clone(),
                    entry.created_at.clone(),
                ]
            })
            .collect::<Vec<_>>();
        tabulate_output(&cols, &data, out)?;
        Ok(())
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let ListWorkspaceLogResponse { entries } =
            client.list_workspace_log(self.to_request()).await?;
        match self.format {
            CliOutputFormatKind::Json => {
                let mut out = std::io::stdout().lock();
                out.write_all(Self::render_entries_json(&entries)?.as_bytes())?;
            }
            CliOutputFormatKind::Table => {
                Self::render_table(&entries, &mut std::io::stdout().lock())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ListWorkspaceLogCommand;
    use clap::Parser;
    use codec::WorkspaceLogState;

    fn sample_entry(
        workspace: &str,
        seq: u64,
        message: &str,
        created_at: &str,
    ) -> WorkspaceLogState {
        WorkspaceLogState {
            workspace: workspace.to_string(),
            seq,
            message: message.to_string(),
            created_at: created_at.to_string(),
        }
    }

    #[test]
    fn workspace_metadata_contracts_list_log_request_shape() {
        let cmd = ListWorkspaceLogCommand::parse_from([
            "kaku",
            "--workspace",
            "unity-main",
            "--limit",
            "20",
            "--format",
            "json",
        ]);

        let request = cmd.to_request();
        assert_eq!(request.workspace.as_deref(), Some("unity-main"));
        assert_eq!(request.limit, Some(20));
    }

    #[test]
    fn workspace_metadata_contracts_list_log_json_shape() {
        let json = ListWorkspaceLogCommand::render_entries_json(&[
            sample_entry("unity-main", 1, "build failed", "2026-03-27T00:00:00.000Z"),
            sample_entry("unity-main", 2, "build fixed", "2026-03-27T00:00:01.000Z"),
        ])
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "[\n",
                "  {\n",
                "    \"workspace\": \"unity-main\",\n",
                "    \"seq\": 1,\n",
                "    \"message\": \"build failed\",\n",
                "    \"created_at\": \"2026-03-27T00:00:00.000Z\"\n",
                "  },\n",
                "  {\n",
                "    \"workspace\": \"unity-main\",\n",
                "    \"seq\": 2,\n",
                "    \"message\": \"build fixed\",\n",
                "    \"created_at\": \"2026-03-27T00:00:01.000Z\"\n",
                "  }\n",
                "]\n"
            )
        );
    }

    #[test]
    fn workspace_metadata_contracts_list_log_table_columns() {
        let mut out = Vec::new();
        ListWorkspaceLogCommand::render_table(
            &[sample_entry(
                "unity-main",
                1,
                "build failed",
                "2026-03-27T00:00:00.000Z",
            )],
            &mut out,
        )
        .expect("table");
        let rendered = String::from_utf8(out).expect("utf8");
        assert!(rendered.contains("WORKSPACE"));
        assert!(rendered.contains("SEQ"));
        assert!(rendered.contains("MESSAGE"));
        assert!(rendered.contains("CREATED_AT"));
    }
}
