use clap::Parser;
use mux::pane::PaneId;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ActivatePane {
    /// Specify the target pane.
    /// The default is to use the current pane based on the
    /// environment variable WEZTERM_PANE.
    #[arg(long)]
    pane_id: Option<PaneId>,
}

impl ActivatePane {
    fn request_for(pane_id: PaneId) -> codec::SetFocusedPane {
        codec::SetFocusedPane { pane_id }
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let pane_id = client.resolve_pane_id(self.pane_id).await?;
        client
            .set_focused_pane_id(Self::request_for(pane_id))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn existing_cli_regressions_activate_pane_accepts_explicit_pane_id() {
        let cmd = ActivatePane::parse_from(["kaku", "--pane-id", "42"]);
        assert_eq!(cmd.pane_id, Some(PaneId::from(42usize)));
    }

    #[test]
    fn existing_cli_regressions_activate_pane_builds_focus_request() {
        let request = ActivatePane::request_for(PaneId::from(7usize));
        assert_eq!(request.pane_id, PaneId::from(7usize));
    }
}
