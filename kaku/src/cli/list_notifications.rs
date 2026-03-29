use crate::cli::CliOutputFormatKind;
use clap::Parser;
use codec::{ListNotifications, ListNotificationsResponse, NotificationRecordState};
use mux::{pane::PaneId, tab::TabId, window::WindowId};
use std::io::Write;
use tabout::{tabulate_output, Alignment, Column};
use wezterm_client::client::Client;

#[derive(Debug, Parser, Clone)]
pub struct ListNotificationsCommand {
    #[arg(long = "workspace")]
    workspace: Option<String>,

    #[arg(long = "window-id")]
    window_id: Option<WindowId>,

    #[arg(long = "tab-id")]
    tab_id: Option<TabId>,

    #[arg(long = "pane-id")]
    pane_id: Option<PaneId>,

    #[arg(long = "unread-only")]
    unread_only: bool,

    #[arg(long = "format", default_value = "table")]
    format: CliOutputFormatKind,
}

impl ListNotificationsCommand {
    fn to_request(&self) -> ListNotifications {
        ListNotifications {
            workspace: self.workspace.clone(),
            window_id: self.window_id,
            tab_id: self.tab_id,
            pane_id: self.pane_id,
            unread_only: self.unread_only,
        }
    }

    fn render_notifications_json(
        notifications: &[NotificationRecordState],
    ) -> anyhow::Result<String> {
        Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(notifications)?
        ))
    }

    fn render_table<W: std::io::Write>(
        notifications: &[NotificationRecordState],
        out: &mut W,
    ) -> anyhow::Result<()> {
        let cols = vec![
            Column {
                name: "NOTIFICATION_ID".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "WORKSPACE".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "WINDOW_ID".to_string(),
                alignment: Alignment::Right,
            },
            Column {
                name: "TAB_ID".to_string(),
                alignment: Alignment::Right,
            },
            Column {
                name: "PANE_ID".to_string(),
                alignment: Alignment::Right,
            },
            Column {
                name: "UNREAD".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "UNREAD_MODE".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "KIND".to_string(),
                alignment: Alignment::Left,
            },
            Column {
                name: "TITLE".to_string(),
                alignment: Alignment::Left,
            },
        ];
        let data = notifications
            .iter()
            .map(|notification| {
                vec![
                    notification.notification_id.clone(),
                    notification.workspace.clone(),
                    notification
                        .window_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    notification
                        .tab_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    notification
                        .pane_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    notification.unread.to_string(),
                    notification.unread_mode.clone(),
                    notification.kind.clone(),
                    notification.title.clone(),
                ]
            })
            .collect::<Vec<_>>();
        tabulate_output(&cols, &data, out)?;
        Ok(())
    }

    pub async fn run(&self, client: Client) -> anyhow::Result<()> {
        let ListNotificationsResponse { notifications } =
            client.list_notifications(self.to_request()).await?;
        match self.format {
            CliOutputFormatKind::Json => {
                let mut out = std::io::stdout().lock();
                out.write_all(Self::render_notifications_json(&notifications)?.as_bytes())?;
            }
            CliOutputFormatKind::Table => {
                Self::render_table(&notifications, &mut std::io::stdout().lock())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ListNotificationsCommand;
    use clap::Parser;
    use codec::NotificationRecordState;
    use mux::{pane::PaneId, tab::TabId, window::WindowId};

    fn sample_notification(
        notification_id: &str,
        window_id: Option<WindowId>,
        tab_id: Option<TabId>,
        pane_id: Option<PaneId>,
    ) -> NotificationRecordState {
        NotificationRecordState {
            notification_id: notification_id.to_string(),
            workspace: "unity".to_string(),
            window_id,
            tab_id,
            pane_id,
            kind: "build.failed".to_string(),
            title: format!("title-{notification_id}"),
            body: None,
            unread: true,
            unread_mode: "sticky".to_string(),
            created_at: "2026-03-27T00:00:00Z".to_string(),
            updated_at: "2026-03-27T00:00:01Z".to_string(),
        }
    }

    #[test]
    fn notification_contracts_list_workspace_only_request_shape() {
        let cmd = ListNotificationsCommand::parse_from([
            "kaku",
            "--workspace",
            "unity",
            "--unread-only",
            "--format",
            "json",
        ]);
        let request = cmd.to_request();
        assert_eq!(request.workspace.as_deref(), Some("unity"));
        assert_eq!(request.window_id, None);
        assert_eq!(request.tab_id, None);
        assert_eq!(request.pane_id, None);
        assert!(request.unread_only);
    }

    #[test]
    fn notification_contracts_list_tab_scoped_json_shape() {
        let json = ListNotificationsCommand::render_notifications_json(&[sample_notification(
            "notif-tab",
            Some(WindowId::from(11usize)),
            Some(TabId::from(22usize)),
            None,
        )])
        .expect("json");

        assert_eq!(
            json,
            concat!(
                "[\n",
                "  {\n",
                "    \"notification_id\": \"notif-tab\",\n",
                "    \"workspace\": \"unity\",\n",
                "    \"window_id\": 11,\n",
                "    \"tab_id\": 22,\n",
                "    \"pane_id\": null,\n",
                "    \"kind\": \"build.failed\",\n",
                "    \"title\": \"title-notif-tab\",\n",
                "    \"body\": null,\n",
                "    \"unread\": true,\n",
                "    \"unread_mode\": \"sticky\",\n",
                "    \"created_at\": \"2026-03-27T00:00:00Z\",\n",
                "    \"updated_at\": \"2026-03-27T00:00:01Z\"\n",
                "  }\n",
                "]\n"
            )
        );
    }

    #[test]
    fn notification_contracts_list_pane_scoped_json_shape() {
        let json = ListNotificationsCommand::render_notifications_json(&[sample_notification(
            "notif-pane",
            Some(WindowId::from(11usize)),
            Some(TabId::from(22usize)),
            Some(PaneId::from(33usize)),
        )])
        .expect("json");

        assert!(json.contains("\"pane_id\": 33"));
        assert!(json.contains("\"unread_mode\": \"sticky\""));
    }
}
