use clap::Parser;
use codec::{CreateNotification, CreateNotificationResponse, NotificationRecordState};
use mux::{pane::PaneId, tab::TabId, window::WindowId};
use std::io::Write;
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct NotifyCommand {
    /// `--workspace <string>`
    #[arg(long = "workspace")]
    workspace: String,

    /// `--window-id <id>`
    #[arg(long = "window-id")]
    window_id: Option<WindowId>,

    /// `--tab-id <id>`
    #[arg(long = "tab-id")]
    tab_id: Option<TabId>,

    /// `--pane-id <id>`
    #[arg(long = "pane-id")]
    pane_id: Option<PaneId>,

    /// `--kind <string>`
    #[arg(long = "kind")]
    kind: String,

    /// `--title <string>`
    #[arg(long = "title")]
    title: String,

    /// `--body <string>`
    #[arg(long = "body")]
    body: Option<String>,

    /// `--unread-mode <clear-on-focus|sticky>`
    #[arg(long = "unread-mode", default_value = "clear-on-focus")]
    unread_mode: String,
}

impl NotifyCommand {
    fn to_request(&self) -> CreateNotification {
        CreateNotification {
            workspace: self.workspace.clone(),
            window_id: self.window_id,
            tab_id: self.tab_id,
            pane_id: self.pane_id,
            kind: self.kind.clone(),
            title: self.title.clone(),
            body: self.body.clone(),
            unread_mode: self.unread_mode.clone(),
        }
    }

    fn render_notification_json(notification: &NotificationRecordState) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(notification)?))
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let CreateNotificationResponse { notification } =
            client.create_notification(self.to_request()).await?;
        let mut out = std::io::stdout().lock();
        out.write_all(Self::render_notification_json(&notification)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::NotifyCommand;
    use clap::Parser;
    use codec::NotificationRecordState;
    use mux::{pane::PaneId, tab::TabId, window::WindowId};

    fn sample_notification(
        window_id: Option<WindowId>,
        tab_id: Option<TabId>,
        pane_id: Option<PaneId>,
    ) -> NotificationRecordState {
        NotificationRecordState {
            notification_id: "notif-1".to_string(),
            workspace: "unity".to_string(),
            window_id,
            tab_id,
            pane_id,
            kind: "build.failed".to_string(),
            title: "Build failed".to_string(),
            body: Some("see logs".to_string()),
            unread: true,
            unread_mode: "clear-on-focus".to_string(),
            created_at: "2026-03-27T00:00:00Z".to_string(),
            updated_at: "2026-03-27T00:00:01Z".to_string(),
        }
    }

    #[test]
    fn notification_contracts_notify_workspace_only_request_shape() {
        let cmd = NotifyCommand::parse_from([
            "kaku",
            "--workspace",
            "unity",
            "--kind",
            "build.failed",
            "--title",
            "Build failed",
        ]);

        let request = cmd.to_request();
        assert_eq!(request.workspace, "unity");
        assert_eq!(request.window_id, None);
        assert_eq!(request.tab_id, None);
        assert_eq!(request.pane_id, None);
        assert_eq!(request.kind, "build.failed");
        assert_eq!(request.title, "Build failed");
        assert_eq!(request.body, None);
        assert_eq!(request.unread_mode, "clear-on-focus");
    }

    #[test]
    fn notification_contracts_notify_tab_scoped_json_shape() {
        let json = NotifyCommand::render_notification_json(&sample_notification(
            Some(WindowId::from(11usize)),
            Some(TabId::from(22usize)),
            None,
        ))
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "{\n",
                "  \"notification_id\": \"notif-1\",\n",
                "  \"workspace\": \"unity\",\n",
                "  \"window_id\": 11,\n",
                "  \"tab_id\": 22,\n",
                "  \"pane_id\": null,\n",
                "  \"kind\": \"build.failed\",\n",
                "  \"title\": \"Build failed\",\n",
                "  \"body\": \"see logs\",\n",
                "  \"unread\": true,\n",
                "  \"unread_mode\": \"clear-on-focus\",\n",
                "  \"created_at\": \"2026-03-27T00:00:00Z\",\n",
                "  \"updated_at\": \"2026-03-27T00:00:01Z\"\n",
                "}\n"
            )
        );
    }

    #[test]
    fn notification_contracts_notify_pane_scoped_json_shape() {
        let json = NotifyCommand::render_notification_json(&sample_notification(
            Some(WindowId::from(11usize)),
            Some(TabId::from(22usize)),
            Some(PaneId::from(33usize)),
        ))
        .expect("json");

        assert!(json.contains("\"window_id\": 11"));
        assert!(json.contains("\"tab_id\": 22"));
        assert!(json.contains("\"pane_id\": 33"));
        assert!(json.contains("\"unread_mode\": \"clear-on-focus\""));
    }
}
