use crate::PKI;
use anyhow::{anyhow, Context};
use chrono::{Datelike, Timelike};
use codec::*;
use config::TermConfig;
use mux::client::ClientId;
use mux::domain::SplitSource;
use mux::notification_store::{
    NotificationCreateRequest, NotificationRecord, NotificationUnreadMode,
};
use mux::pane::{CachePolicy, Pane, PaneId};
use mux::renderable::{RenderableDimensions, StableCursorPosition};
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::{WorkspaceLogRecord, WorkspaceProgressRecord, WorkspaceStatusRecord};
use mux::{Mux, MuxNotification};
use promise::spawn::spawn_into_main_thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use termwiz::surface::SequenceNo;
use url::Url;
use wezterm_term::terminal::Alert;
use wezterm_term::StableRowIndex;

fn make_notification_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("notification-{}-{}", now.as_secs(), now.subsec_nanos())
}

fn parse_unread_mode(mode: &str) -> anyhow::Result<NotificationUnreadMode> {
    match mode {
        "clear-on-focus" => Ok(NotificationUnreadMode::ClearOnFocus),
        "sticky" => Ok(NotificationUnreadMode::Sticky),
        _ => Err(anyhow!("unsupported unread mode `{mode}`")),
    }
}

fn notification_record_to_state(record: &NotificationRecord) -> NotificationRecordState {
    NotificationRecordState {
        notification_id: record.notification_id.clone(),
        workspace: record.workspace.clone(),
        window_id: record.window_id,
        tab_id: record.tab_id,
        pane_id: record.pane_id,
        kind: record.kind.clone(),
        title: record.title.clone(),
        body: record.body.clone(),
        unread: record.unread,
        unread_mode: match record.unread_mode {
            NotificationUnreadMode::ClearOnFocus => "clear-on-focus".to_string(),
            NotificationUnreadMode::Sticky => "sticky".to_string(),
        },
        created_at: record.created_at.to_string(),
        updated_at: record.updated_at.to_string(),
    }
}

trait TimestampRfc3339Ext {
    fn to_rfc3339(&self) -> String;
}

impl TimestampRfc3339Ext for chrono::DateTime<chrono::Utc> {
    fn to_rfc3339(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second(),
            self.timestamp_subsec_millis()
        )
    }
}

fn timestamp_to_rfc3339(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    timestamp.to_rfc3339()
}

fn workspace_status_to_state(record: WorkspaceStatusRecord) -> WorkspaceStatusState {
    WorkspaceStatusState {
        workspace: record.workspace,
        status: record.status,
        updated_at: timestamp_to_rfc3339(record.updated_at),
    }
}

fn workspace_progress_to_state(record: WorkspaceProgressRecord) -> WorkspaceProgressState {
    WorkspaceProgressState {
        workspace: record.workspace,
        value: record.value,
        updated_at: timestamp_to_rfc3339(record.updated_at),
    }
}

fn workspace_log_to_state(record: WorkspaceLogRecord) -> WorkspaceLogState {
    WorkspaceLogState {
        workspace: record.workspace,
        seq: record.seq,
        message: record.message,
        created_at: timestamp_to_rfc3339(record.created_at),
    }
}

fn task_pane_to_state(record: TaskPaneRecord) -> TaskPaneState {
    TaskPaneState {
        pane_id: record.pane_id,
        workspace: record.workspace,
        window_id: record.window_id,
        tab_id: record.tab_id,
        remain_on_exit: record.remain_on_exit,
        silenced: record.silenced,
        is_dead: record.is_dead,
        is_failed: record.is_failed,
        rerun_available: !record.rerun.is_empty(),
        tee_path: record.tee_path,
        current_working_dir: record.current_working_dir,
        updated_at: timestamp_to_rfc3339(record.updated_at),
    }
}

fn clear_selected_workspaces<F>(workspaces: Vec<String>, mut clear: F) -> (usize, Vec<String>)
where
    F: FnMut(&str) -> bool,
{
    let mut cleared = Vec::new();
    for workspace in workspaces {
        if clear(&workspace) {
            cleared.push(workspace);
        }
    }
    let cleared_count = cleared.len();
    (cleared_count, cleared)
}

fn notification_matches_list_filter(
    record: &NotificationRecord,
    request: &ListNotifications,
) -> bool {
    if request
        .workspace
        .as_ref()
        .map(|workspace| &record.workspace != workspace)
        .unwrap_or(false)
    {
        return false;
    }
    if request
        .window_id
        .map(|window_id| record.window_id != Some(window_id))
        .unwrap_or(false)
    {
        return false;
    }
    if request
        .tab_id
        .map(|tab_id| record.tab_id != Some(tab_id))
        .unwrap_or(false)
    {
        return false;
    }
    if request
        .pane_id
        .map(|pane_id| record.pane_id != Some(pane_id))
        .unwrap_or(false)
    {
        return false;
    }
    if request.unread_only && !record.unread {
        return false;
    }
    true
}

fn notification_matches_clear_filter(
    record: &NotificationRecord,
    request: &ClearNotifications,
) -> bool {
    if !request.notification_ids.is_empty()
        && !request
            .notification_ids
            .iter()
            .any(|notification_id| notification_id == &record.notification_id)
    {
        return false;
    }
    if request
        .workspace
        .as_ref()
        .map(|workspace| &record.workspace != workspace)
        .unwrap_or(false)
    {
        return false;
    }
    if request
        .tab_id
        .map(|tab_id| record.tab_id != Some(tab_id))
        .unwrap_or(false)
    {
        return false;
    }
    if request
        .pane_id
        .map(|pane_id| record.pane_id != Some(pane_id))
        .unwrap_or(false)
    {
        return false;
    }
    true
}

fn set_focused_pane(
    mux: &Mux,
    client_id: Option<Arc<ClientId>>,
    pane_id: PaneId,
) -> anyhow::Result<()> {
    let _identity = mux.with_identity(client_id);

    let pane = mux
        .get_pane(pane_id)
        .ok_or_else(|| anyhow::anyhow!("pane {pane_id} not found"))?;

    let (_domain_id, window_id, tab_id) = mux
        .resolve_pane_id(pane_id)
        .ok_or_else(|| anyhow::anyhow!("pane {pane_id} not found"))?;
    {
        let mut window = mux
            .get_window_mut(window_id)
            .ok_or_else(|| anyhow::anyhow!("window {window_id} not found"))?;
        let tab_idx = window
            .idx_by_id(tab_id)
            .ok_or_else(|| anyhow::anyhow!("tab {tab_id} isn't really in window {window_id}!?"))?;
        window.save_and_then_set_active(tab_idx);
    }
    let tab = mux
        .get_tab(tab_id)
        .ok_or_else(|| anyhow::anyhow!("tab {tab_id} not found"))?;
    tab.set_active_pane(&pane);

    mux.record_focus_for_current_identity(pane_id);
    mux.notify(MuxNotification::PaneFocused(pane_id));
    Ok(())
}

#[derive(Clone)]
pub struct PduSender {
    func: Arc<dyn Fn(DecodedPdu) -> anyhow::Result<()> + Send + Sync>,
}

impl PduSender {
    pub fn send(&self, pdu: DecodedPdu) -> anyhow::Result<()> {
        (self.func)(pdu)
    }

    pub fn new<T>(f: T) -> Self
    where
        T: Fn(DecodedPdu) -> anyhow::Result<()> + Send + Sync + 'static,
    {
        Self { func: Arc::new(f) }
    }
}

#[derive(Default, Debug)]
pub(crate) struct PerPane {
    cursor_position: StableCursorPosition,
    title: String,
    working_dir: Option<Url>,
    dimensions: RenderableDimensions,
    mouse_grabbed: bool,
    sent_initial_palette: bool,
    seqno: SequenceNo,
    config_generation: usize,
    pub(crate) notifications: Vec<Alert>,
}

impl PerPane {
    fn compute_changes(
        &mut self,
        pane: &Arc<dyn Pane>,
        force_with_input_serial: Option<InputSerial>,
    ) -> Option<GetPaneRenderChangesResponse> {
        let mut changed = false;
        let mouse_grabbed = pane.is_mouse_grabbed();
        if mouse_grabbed != self.mouse_grabbed {
            changed = true;
        }

        let dims = pane.get_dimensions();
        if dims != self.dimensions {
            changed = true;
        }

        let cursor_position = pane.get_cursor_position();
        if cursor_position != self.cursor_position {
            changed = true;
        }

        let title = pane.get_title();
        if title != self.title {
            changed = true;
        }

        let working_dir = pane.get_current_working_dir(CachePolicy::AllowStale);
        if working_dir != self.working_dir {
            changed = true;
        }

        let old_seqno = self.seqno;
        self.seqno = pane.get_current_seqno();
        let mut all_dirty_lines = pane.get_changed_since(
            0..dims.physical_top + dims.viewport_rows as StableRowIndex,
            old_seqno,
        );
        if !all_dirty_lines.is_empty() {
            changed = true;
        }

        if !changed && !force_with_input_serial.is_some() {
            return None;
        }

        // Figure out what we're going to send as dirty lines vs bonus lines
        let viewport_range =
            dims.physical_top..dims.physical_top + dims.viewport_rows as StableRowIndex;

        let (first_line, lines) = pane.get_lines(viewport_range);
        let mut bonus_lines = lines
            .into_iter()
            .enumerate()
            .filter_map(|(idx, mut line)| {
                let stable_row = first_line + idx as StableRowIndex;
                if all_dirty_lines.contains(stable_row) {
                    all_dirty_lines.remove(stable_row);
                    line.compress_for_scrollback();
                    Some((stable_row, line))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Always send the cursor's row, as that tends to the busiest and we don't
        // have a sequencing concept for our idea of the remote state.
        let (cursor_line_idx, mut lines) = pane.get_lines(cursor_position.y..cursor_position.y + 1);
        let mut cursor_line = lines.remove(0);
        cursor_line.compress_for_scrollback();
        bonus_lines.push((cursor_line_idx, cursor_line));

        self.cursor_position = cursor_position;
        self.title = title.clone();
        self.working_dir = working_dir.clone();
        self.dimensions = dims;
        self.mouse_grabbed = mouse_grabbed;

        let bonus_lines = bonus_lines.into();
        Some(GetPaneRenderChangesResponse {
            pane_id: pane.pane_id(),
            mouse_grabbed,
            dirty_lines: all_dirty_lines.iter().cloned().collect(),
            dimensions: dims,
            cursor_position,
            title,
            bonus_lines,
            working_dir: working_dir.map(Into::into),
            input_serial: force_with_input_serial,
            seqno: self.seqno,
        })
    }
}

fn maybe_push_pane_changes(
    pane: &Arc<dyn Pane>,
    sender: PduSender,
    per_pane: Arc<Mutex<PerPane>>,
) -> anyhow::Result<()> {
    let mut per_pane = per_pane.lock().unwrap();
    if let Some(resp) = per_pane.compute_changes(pane, None) {
        sender.send(DecodedPdu {
            pdu: Pdu::GetPaneRenderChangesResponse(resp),
            serial: 0,
        })?;
    }

    let config = config::configuration();
    if per_pane.config_generation != config.generation() {
        per_pane.config_generation = config.generation();
        // If the config changed, it may have changed colors
        // in the palette that we need to push down, so we
        // synthesize a palette change notification to let
        // the client know
        per_pane.notifications.push(Alert::PaletteChanged);
        per_pane.sent_initial_palette = true;
    }

    if !per_pane.sent_initial_palette {
        per_pane.notifications.push(Alert::PaletteChanged);
        per_pane.sent_initial_palette = true;
    }
    for alert in per_pane.notifications.drain(..) {
        match alert {
            Alert::PaletteChanged => {
                sender.send(DecodedPdu {
                    pdu: Pdu::SetPalette(SetPalette {
                        pane_id: pane.pane_id(),
                        palette: pane.palette(),
                    }),
                    serial: 0,
                })?;
            }
            alert => {
                sender.send(DecodedPdu {
                    pdu: Pdu::NotifyAlert(NotifyAlert {
                        pane_id: pane.pane_id(),
                        alert,
                    }),
                    serial: 0,
                })?;
            }
        }
    }
    Ok(())
}

pub struct SessionHandler {
    to_write_tx: PduSender,
    per_pane: HashMap<PaneId, Arc<Mutex<PerPane>>>,
    client_id: Option<Arc<ClientId>>,
    proxy_client_id: Option<ClientId>,
}

impl Drop for SessionHandler {
    fn drop(&mut self) {
        if let Some(client_id) = self.client_id.take() {
            let mux = Mux::get();
            mux.unregister_client(&client_id);
        }
    }
}

impl SessionHandler {
    pub fn new(to_write_tx: PduSender) -> Self {
        Self {
            to_write_tx,
            per_pane: HashMap::new(),
            client_id: None,
            proxy_client_id: None,
        }
    }

    pub(crate) fn per_pane(&mut self, pane_id: PaneId) -> Arc<Mutex<PerPane>> {
        Arc::clone(
            self.per_pane
                .entry(pane_id)
                .or_insert_with(|| Arc::new(Mutex::new(PerPane::default()))),
        )
    }

    pub fn schedule_pane_push(&mut self, pane_id: PaneId) {
        let sender = self.to_write_tx.clone();
        let per_pane = self.per_pane(pane_id);
        spawn_into_main_thread(async move {
            let mux = Mux::get();
            let pane = mux
                .get_pane(pane_id)
                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
            maybe_push_pane_changes(&pane, sender, per_pane)?;
            Ok::<(), anyhow::Error>(())
        })
        .detach();
    }

    pub fn process_one(&mut self, decoded: DecodedPdu) {
        let start = Instant::now();
        let sender = self.to_write_tx.clone();
        let serial = decoded.serial;

        if let Some(client_id) = &self.client_id {
            if decoded.pdu.is_user_input() {
                Mux::get().client_had_input(client_id);
            }
        }

        let send_response = move |result: anyhow::Result<Pdu>| {
            let pdu = match result {
                Ok(pdu) => pdu,
                Err(err) => Pdu::ErrorResponse(ErrorResponse {
                    reason: format!("Error: {err:#}"),
                }),
            };
            log::trace!("{} processing time {:?}", serial, start.elapsed());
            sender.send(DecodedPdu { pdu, serial }).ok();
        };

        fn catch<F, SND>(f: F, send_response: SND)
        where
            F: FnOnce() -> anyhow::Result<Pdu>,
            SND: Fn(anyhow::Result<Pdu>),
        {
            send_response(f());
        }

        match decoded.pdu {
            Pdu::Ping(Ping {}) => send_response(Ok(Pdu::Pong(Pong {}))),
            Pdu::SetWindowWorkspace(SetWindowWorkspace {
                window_id,
                workspace,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let mut window = mux
                                .get_window_mut(window_id)
                                .ok_or_else(|| anyhow!("window {} is invalid", window_id))?;
                            window.set_workspace(&workspace);
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::SetClientId(SetClientId {
                mut client_id,
                is_proxy,
            }) => {
                if is_proxy {
                    if self.proxy_client_id.is_none() {
                        // Copy proxy identity, but don't assign it to the mux;
                        // we'll use it to annotate the actual clients own
                        // identity when they send it
                        self.proxy_client_id.replace(client_id);
                    }
                } else {
                    // If this session is a proxy, override the incoming id with
                    // the proxy information so that it is clear what is going
                    // on from the `wezterm cli list-clients` information
                    if let Some(proxy_id) = &self.proxy_client_id {
                        client_id.ssh_auth_sock = proxy_id.ssh_auth_sock.clone();
                        // Note that this `via proxy pid` string is coupled
                        // with the logic in mux/src/ssh_agent
                        client_id.hostname =
                            format!("{} (via proxy pid {})", client_id.hostname, proxy_id.pid);
                    }

                    let client_id = Arc::new(client_id);
                    self.client_id.replace(client_id.clone());
                    spawn_into_main_thread(async move {
                        let mux = Mux::get();
                        mux.register_client(client_id);
                    })
                    .detach();
                }
                send_response(Ok(Pdu::UnitResponse(UnitResponse {})))
            }
            Pdu::SetFocusedPane(SetFocusedPane { pane_id }) => {
                let client_id = self.client_id.clone();
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            set_focused_pane(&mux, client_id, pane_id)?;
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::CreateNotification(request) => {
                let client_id = self.client_id.clone();
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let _identity = mux.with_identity(client_id);
                            let record = NotificationCreateRequest {
                                notification_id: make_notification_id(),
                                workspace: request.workspace,
                                window_id: request.window_id,
                                tab_id: request.tab_id,
                                pane_id: request.pane_id,
                                kind: request.kind,
                                title: request.title,
                                body: request.body,
                                unread_mode: parse_unread_mode(&request.unread_mode)?,
                            }
                            .into_record();
                            let notification = mux.create_notification(record)?;
                            Ok(Pdu::CreateNotificationResponse(
                                CreateNotificationResponse {
                                    notification: notification_record_to_state(&notification),
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ListNotifications(request) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let notifications = mux
                                .list_notifications()
                                .into_iter()
                                .filter(|record| notification_matches_list_filter(record, &request))
                                .map(|record| notification_record_to_state(&record))
                                .collect();
                            Ok(Pdu::ListNotificationsResponse(ListNotificationsResponse {
                                notifications,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ClearNotifications(request) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let notification_ids: Vec<String> = mux
                                .list_notifications()
                                .into_iter()
                                .filter(|record| {
                                    notification_matches_clear_filter(record, &request)
                                })
                                .map(|record| record.notification_id)
                                .collect();
                            let cleared_count = mux.clear_notifications(&notification_ids);
                            Ok(Pdu::ClearNotificationsResponse(
                                ClearNotificationsResponse {
                                    cleared_count,
                                    notification_ids,
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::MarkNotificationsRead(request) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let updated_count =
                                mux.mark_notifications_read(&request.notification_ids);
                            Ok(Pdu::MarkNotificationsResponse(MarkNotificationsResponse {
                                updated_count,
                                notification_ids: request.notification_ids,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::MarkNotificationsUnread(request) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let updated_count =
                                mux.mark_notifications_unread(&request.notification_ids);
                            Ok(Pdu::MarkNotificationsResponse(MarkNotificationsResponse {
                                updated_count,
                                notification_ids: request.notification_ids,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::JumpUnread(JumpUnread { pane_id, direction }) => {
                let client_id = self.client_id.clone();
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            if direction != "next" && direction != "prev" {
                                return Err(anyhow!(
                                    "direction must be either \"next\" or \"prev\""
                                ));
                            }
                            let mux = Mux::get();
                            let target = if direction == "next" {
                                mux.jump_next_unread_pane(Some(pane_id))
                            } else {
                                mux.jump_prev_unread_pane(Some(pane_id))
                            };
                            if let Some(target_pane_id) = target {
                                set_focused_pane(&mux, client_id, target_pane_id)?;
                            }
                            Ok(Pdu::JumpUnreadResponse(JumpUnreadResponse {
                                pane_id: target,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::IdentifyNotificationTarget(IdentifyNotificationTarget { pane_id }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let (_domain_id, window_id, tab_id) = mux
                                .resolve_pane_id(pane_id)
                                .ok_or_else(|| anyhow!("pane {pane_id} not found"))?;
                            let workspace = mux
                                .get_window(window_id)
                                .map(|window| window.get_workspace().to_string())
                                .ok_or_else(|| anyhow!("window {window_id} not found"))?;
                            Ok(Pdu::IdentifyNotificationTargetResponse(
                                IdentifyNotificationTargetResponse {
                                    workspace,
                                    window_id,
                                    tab_id,
                                    pane_id,
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::GetNotificationCapabilities(_) => send_response(Ok(
                Pdu::GetNotificationCapabilitiesResponse(GetNotificationCapabilitiesResponse {
                    notification_commands: vec![
                        "notify".to_string(),
                        "list-notifications".to_string(),
                        "clear-notifications".to_string(),
                        "mark-read".to_string(),
                        "mark-unread".to_string(),
                        "jump-next-unread".to_string(),
                        "jump-prev-unread".to_string(),
                        "identify".to_string(),
                        "capabilities".to_string(),
                    ],
                    unread_modes: vec!["clear-on-focus".to_string(), "sticky".to_string()],
                    supports_tabbar_markers: true,
                }),
            )),
            Pdu::SetWorkspaceStatus(SetWorkspaceStatus { workspace, status }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            mux.set_workspace_status(&workspace, &status);
                            let status =
                                mux.workspace_status_for_workspace(&workspace).ok_or_else(
                                    || anyhow!("workspace status missing for {workspace}"),
                                )?;
                            Ok(Pdu::SetWorkspaceStatusResponse(
                                SetWorkspaceStatusResponse {
                                    status: workspace_status_to_state(status),
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ClearWorkspaceStatus(ClearWorkspaceStatus { workspace }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let workspaces = match workspace {
                                Some(workspace) => vec![workspace],
                                None => mux
                                    .list_workspace_status()
                                    .into_iter()
                                    .map(|record| record.workspace)
                                    .collect(),
                            };
                            let (cleared_count, workspaces) =
                                clear_selected_workspaces(workspaces, |workspace| {
                                    mux.clear_workspace_status(workspace)
                                });
                            Ok(Pdu::ClearWorkspaceStatusResponse(
                                ClearWorkspaceStatusResponse {
                                    cleared_count,
                                    workspaces,
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ListWorkspaceStatus(ListWorkspaceStatus { workspace }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let statuses = Mux::get()
                                .list_workspace_status()
                                .into_iter()
                                .filter(|record| {
                                    workspace
                                        .as_ref()
                                        .map(|name| &record.workspace == name)
                                        .unwrap_or(true)
                                })
                                .map(workspace_status_to_state)
                                .collect();
                            Ok(Pdu::ListWorkspaceStatusResponse(
                                ListWorkspaceStatusResponse { statuses },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::SetWorkspaceProgress(SetWorkspaceProgress { workspace, value }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            mux.set_workspace_progress(&workspace, value).map_err(
                                |err| match err {
                                    mux::workspace_state::WorkspaceProgressError::OutOfRange(
                                        value,
                                    ) => {
                                        anyhow!("workspace progress value {value} is out of range")
                                    }
                                },
                            )?;
                            let progress = mux
                                .workspace_progress_for_workspace(&workspace)
                                .ok_or_else(|| {
                                    anyhow!("workspace progress missing for {workspace}")
                                })?;
                            Ok(Pdu::SetWorkspaceProgressResponse(
                                SetWorkspaceProgressResponse {
                                    progress: workspace_progress_to_state(progress),
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ClearWorkspaceProgress(ClearWorkspaceProgress { workspace }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let workspaces = match workspace {
                                Some(workspace) => vec![workspace],
                                None => mux
                                    .list_workspace_progress()
                                    .into_iter()
                                    .map(|record| record.workspace)
                                    .collect(),
                            };
                            let (cleared_count, workspaces) =
                                clear_selected_workspaces(workspaces, |workspace| {
                                    mux.clear_workspace_progress(workspace)
                                });
                            Ok(Pdu::ClearWorkspaceProgressResponse(
                                ClearWorkspaceProgressResponse {
                                    cleared_count,
                                    workspaces,
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::AppendWorkspaceLog(AppendWorkspaceLog { workspace, message }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let entry = Mux::get().append_workspace_log(&workspace, &message);
                            Ok(Pdu::AppendWorkspaceLogResponse(
                                AppendWorkspaceLogResponse {
                                    entry: workspace_log_to_state(entry),
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ClearWorkspaceLog(ClearWorkspaceLog { workspace }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let workspaces = match workspace {
                                Some(workspace) => vec![workspace],
                                None => mux.list_workspace_log_workspaces(),
                            };
                            let (cleared_count, workspaces) =
                                clear_selected_workspaces(workspaces, |workspace| {
                                    mux.clear_workspace_log(workspace)
                                });
                            Ok(Pdu::ClearWorkspaceLogResponse(ClearWorkspaceLogResponse {
                                cleared_count,
                                workspaces,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ListWorkspaceLog(ListWorkspaceLog { workspace, limit }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let mut entries = if let Some(workspace) = workspace {
                                mux.list_workspace_log(&workspace)
                            } else {
                                let mut entries = mux
                                    .list_workspace_log_workspaces()
                                    .into_iter()
                                    .flat_map(|workspace| mux.list_workspace_log(&workspace))
                                    .collect::<Vec<_>>();
                                entries.sort_by_key(|record| record.seq);
                                entries
                            };

                            if let Some(limit) = limit {
                                if entries.len() > limit {
                                    entries = entries.split_off(entries.len() - limit);
                                }
                            }

                            let entries = entries.into_iter().map(workspace_log_to_state).collect();
                            Ok(Pdu::ListWorkspaceLogResponse(ListWorkspaceLogResponse {
                                entries,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ListTaskPanes(ListTaskPanes { pane_id, workspace }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let task_panes = Mux::get()
                                .list_task_panes()
                                .into_iter()
                                .filter(|record| {
                                    pane_id
                                        .map(|target| record.pane_id == target)
                                        .unwrap_or(true)
                                })
                                .filter(|record| {
                                    workspace
                                        .as_ref()
                                        .map(|target| record.workspace.as_deref() == Some(target))
                                        .unwrap_or(true)
                                })
                                .map(task_pane_to_state)
                                .collect();
                            Ok(Pdu::ListTaskPanesResponse(ListTaskPanesResponse {
                                task_panes,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::SetRemainOnExit(SetRemainOnExit {
                pane_id,
                remain_on_exit,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let pane = Mux::get()
                                .set_task_pane_remain_on_exit(pane_id, remain_on_exit)
                                .ok_or_else(|| anyhow!("pane {} not found", pane_id))?;
                            Ok(Pdu::SetRemainOnExitResponse(SetRemainOnExitResponse {
                                pane: task_pane_to_state(pane),
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::SilenceWatchdog(SilenceWatchdog { pane_id, silenced }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let pane = Mux::get()
                                .set_task_pane_silenced(pane_id, silenced)
                                .ok_or_else(|| anyhow!("pane {} not found", pane_id))?;
                            Ok(Pdu::SilenceWatchdogResponse(SilenceWatchdogResponse {
                                pane_id: pane.pane_id,
                                silenced: pane.silenced,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::RerunPane(request) => {
                spawn_into_main_thread(async move {
                    schedule_rerun_task_pane(request, send_response);
                })
                .detach();
            }
            Pdu::RespawnPane(request) => {
                spawn_into_main_thread(async move {
                    schedule_respawn_task_pane(request, send_response);
                })
                .detach();
            }
            Pdu::PipePane(PipePane { pane_id, file_path }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let pane = Mux::get()
                                .set_task_pane_tee_path(pane_id, file_path)
                                .ok_or_else(|| anyhow!("pane {} not found", pane_id))?;
                            Ok(Pdu::PipePaneResponse(PipePaneResponse {
                                pane_id: pane.pane_id,
                                tee_path: pane.tee_path,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::GetClientList(GetClientList) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let clients = mux.iter_clients();
                            Ok(Pdu::GetClientListResponse(GetClientListResponse {
                                clients,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::ListPanes(ListPanes {}) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let mut tabs = vec![];
                            let mut tab_titles = vec![];
                            let mut window_titles = HashMap::new();
                            for window_id in mux.iter_windows().into_iter() {
                                let window = mux.get_window(window_id).unwrap();
                                window_titles.insert(window_id, window.get_title().to_string());
                                for tab in window.iter() {
                                    tabs.push(tab.codec_pane_tree());
                                    tab_titles.push(tab.get_title());
                                }
                            }
                            log::trace!("ListPanes {tabs:#?} {tab_titles:?}");
                            Ok(Pdu::ListPanesResponse(ListPanesResponse {
                                tabs,
                                tab_titles,
                                window_titles,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::RenameWorkspace(RenameWorkspace {
                old_workspace,
                new_workspace,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            mux.rename_workspace(&old_workspace, &new_workspace);
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    );
                })
                .detach();
            }

            Pdu::WriteToPane(WriteToPane { pane_id, data }) => {
                let sender = self.to_write_tx.clone();
                let per_pane = self.per_pane(pane_id);
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.writer().write_all(&data)?;
                            maybe_push_pane_changes(&pane, sender, per_pane)?;
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    );
                })
                .detach();
            }
            Pdu::EraseScrollbackRequest(EraseScrollbackRequest {
                pane_id,
                erase_mode,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.erase_scrollback(erase_mode);
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    );
                })
                .detach();
            }
            Pdu::KillPane(KillPane { pane_id }) => {
                let sender = self.to_write_tx.clone();
                let per_pane = self.per_pane(pane_id);
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.kill();
                            mux.remove_pane(pane_id);
                            maybe_push_pane_changes(&pane, sender, per_pane)?;
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    );
                })
                .detach();
            }
            Pdu::SendPaste(SendPaste { pane_id, data }) => {
                let sender = self.to_write_tx.clone();
                let per_pane = self.per_pane(pane_id);
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.send_paste(&data)?;
                            maybe_push_pane_changes(&pane, sender, per_pane)?;
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::SearchScrollbackRequest(SearchScrollbackRequest {
                pane_id,
                pattern,
                range,
                limit,
            }) => {
                use mux::pane::Pattern;

                async fn do_search(
                    pane_id: PaneId,
                    pattern: Pattern,
                    range: std::ops::Range<StableRowIndex>,
                    limit: Option<u32>,
                ) -> anyhow::Result<Pdu> {
                    let mux = Mux::get();
                    let pane = mux
                        .get_pane(pane_id)
                        .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;

                    pane.search(pattern, range, limit).await.map(|results| {
                        Pdu::SearchScrollbackResponse(SearchScrollbackResponse { results })
                    })
                }

                spawn_into_main_thread(async move {
                    promise::spawn::spawn(async move {
                        let result = do_search(pane_id, pattern, range, limit).await;
                        send_response(result);
                    })
                    .detach();
                })
                .detach();
            }

            Pdu::SetPaneZoomed(SetPaneZoomed {
                containing_tab_id,
                pane_id,
                zoomed,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            let tab = mux
                                .get_tab(containing_tab_id)
                                .ok_or_else(|| anyhow!("no such tab {}", containing_tab_id))?;
                            match tab.get_zoomed_pane() {
                                Some(p) => {
                                    let is_zoomed = p.pane_id() == pane_id;
                                    if is_zoomed != zoomed {
                                        tab.set_zoomed(false);
                                        if zoomed {
                                            tab.set_active_pane(&pane);
                                            tab.set_zoomed(zoomed);
                                        }
                                    }
                                }
                                None => {
                                    if zoomed {
                                        tab.set_active_pane(&pane);
                                        tab.set_zoomed(zoomed);
                                    }
                                }
                            }
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::GetPaneDirection(GetPaneDirection { pane_id, direction }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let (_domain_id, _window_id, tab_id) = mux
                                .resolve_pane_id(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            let tab = mux
                                .get_tab(tab_id)
                                .ok_or_else(|| anyhow!("no such tab {}", tab_id))?;
                            let panes = tab.iter_panes_ignoring_zoom();
                            let pane_id = tab
                                .get_pane_direction(direction, true)
                                .map(|pane_index| panes[pane_index].pane.pane_id());

                            Ok(Pdu::GetPaneDirectionResponse(GetPaneDirectionResponse {
                                pane_id,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::ActivatePaneDirection(ActivatePaneDirection { pane_id, direction }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let (_domain_id, _window_id, tab_id) = mux
                                .resolve_pane_id(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            let tab = mux
                                .get_tab(tab_id)
                                .ok_or_else(|| anyhow!("no such tab {}", tab_id))?;
                            tab.activate_pane_direction(direction);
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::Resize(Resize {
                containing_tab_id,
                pane_id,
                size,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.resize(size)?;
                            let tab = mux
                                .get_tab(containing_tab_id)
                                .ok_or_else(|| anyhow!("no such tab {}", containing_tab_id))?;
                            tab.rebuild_splits_sizes_from_contained_panes();
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::SendKeyDown(SendKeyDown {
                pane_id,
                event,
                input_serial,
            }) => {
                let sender = self.to_write_tx.clone();
                let per_pane = self.per_pane(pane_id);
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.key_down(event.key, event.modifiers)?;

                            // For a key press, we want to always send back the
                            // cursor position so that the predictive echo doesn't
                            // leave the cursor in the wrong place
                            let mut per_pane = per_pane.lock().unwrap();
                            if let Some(resp) = per_pane.compute_changes(&pane, Some(input_serial))
                            {
                                sender.send(DecodedPdu {
                                    pdu: Pdu::GetPaneRenderChangesResponse(resp),
                                    serial: 0,
                                })?;
                            }
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::SendMouseEvent(SendMouseEvent { pane_id, event }) => {
                let sender = self.to_write_tx.clone();
                let per_pane = self.per_pane(pane_id);
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            pane.mouse_event(event)?;
                            maybe_push_pane_changes(&pane, sender, per_pane)?;
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::SpawnV2(spawn) => {
                let client_id = self.client_id.clone();
                spawn_into_main_thread(async move {
                    schedule_domain_spawn_v2(spawn, send_response, client_id);
                })
                .detach();
            }

            Pdu::SplitPane(split) => {
                let client_id = self.client_id.clone();
                spawn_into_main_thread(async move {
                    schedule_split_pane(split, send_response, client_id);
                })
                .detach();
            }

            Pdu::MovePaneToNewTab(request) => {
                let client_id = self.client_id.clone();
                spawn_into_main_thread(async move {
                    schedule_move_pane(request, send_response, client_id);
                })
                .detach();
            }

            Pdu::GetPaneRenderableDimensions(GetPaneRenderableDimensions { pane_id }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            let cursor_position = pane.get_cursor_position();
                            let dimensions = pane.get_dimensions();
                            Ok(Pdu::GetPaneRenderableDimensionsResponse(
                                GetPaneRenderableDimensionsResponse {
                                    pane_id,
                                    cursor_position,
                                    dimensions,
                                },
                            ))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::GetPaneRenderChanges(GetPaneRenderChanges { pane_id, .. }) => {
                let sender = self.to_write_tx.clone();
                let per_pane = self.per_pane(pane_id);
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let is_alive = match mux.get_pane(pane_id) {
                                Some(pane) => {
                                    maybe_push_pane_changes(&pane, sender, per_pane)?;
                                    true
                                }
                                None => false,
                            };
                            Ok(Pdu::LivenessResponse(LivenessResponse {
                                pane_id,
                                is_alive,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::GetLines(GetLines { pane_id, lines }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;
                            let mut lines_and_indices = vec![];

                            for range in lines {
                                let (first_row, lines) = pane.get_lines(range);
                                for (idx, mut line) in lines.into_iter().enumerate() {
                                    let stable_row = first_row + idx as StableRowIndex;
                                    line.compress_for_scrollback();
                                    lines_and_indices.push((stable_row, line));
                                }
                            }
                            Ok(Pdu::GetLinesResponse(GetLinesResponse {
                                pane_id,
                                lines: lines_and_indices.into(),
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::GetImageCell(GetImageCell {
                pane_id,
                line_idx,
                cell_idx,
                data_hash,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let mut data = None;

                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;

                            let (_, lines) = pane.get_lines(line_idx..line_idx + 1);
                            'found_data: for line in lines {
                                if let Some(cell) = line.get_cell(cell_idx) {
                                    if let Some(images) = cell.attrs().images() {
                                        for im in images {
                                            if im.image_data().hash() == data_hash {
                                                data.replace(im.image_data().clone());
                                                break 'found_data;
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Pdu::GetImageCellResponse(GetImageCellResponse {
                                pane_id,
                                data,
                            }))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::GetCodecVersion(_) => {
                match std::env::current_exe().context("resolving current_exe") {
                    Err(err) => send_response(Err(err)),
                    Ok(executable_path) => {
                        send_response(Ok(Pdu::GetCodecVersionResponse(GetCodecVersionResponse {
                            codec_vers: CODEC_VERSION,
                            version_string: config::wezterm_version().to_owned(),
                            executable_path,
                            config_file_path: std::env::var_os("WEZTERM_CONFIG_FILE")
                                .map(Into::into),
                        })))
                    }
                }
            }

            Pdu::GetTlsCreds(_) => {
                catch(
                    move || {
                        let client_cert_pem = PKI.generate_client_cert()?;
                        let ca_cert_pem = PKI.ca_pem_string()?;
                        Ok(Pdu::GetTlsCredsResponse(GetTlsCredsResponse {
                            client_cert_pem,
                            ca_cert_pem,
                        }))
                    },
                    send_response,
                );
            }
            Pdu::WindowTitleChanged(WindowTitleChanged { window_id, title }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let mut window = mux
                                .get_window_mut(window_id)
                                .ok_or_else(|| anyhow!("no such window {window_id}"))?;

                            window.set_title(&title);

                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::TabTitleChanged(TabTitleChanged { tab_id, title }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let tab = mux
                                .get_tab(tab_id)
                                .ok_or_else(|| anyhow!("no such tab {tab_id}"))?;

                            tab.set_title(&title);

                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }
            Pdu::SetPalette(SetPalette { pane_id, palette }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let pane = mux
                                .get_pane(pane_id)
                                .ok_or_else(|| anyhow!("no such pane {}", pane_id))?;

                            match pane.get_config() {
                                Some(config) => match config.downcast_ref::<TermConfig>() {
                                    Some(tc) => tc.set_client_palette(palette),
                                    None => {
                                        log::error!(
                                            "pane {pane_id} doesn't \
                                            have TermConfig as its config! \
                                            Ignoring client palette update"
                                        );
                                    }
                                },
                                None => {
                                    let config = TermConfig::new();
                                    config.set_client_palette(palette);
                                    pane.set_config(Arc::new(config));
                                }
                            }

                            mux.notify(MuxNotification::Alert {
                                pane_id,
                                alert: Alert::PaletteChanged,
                            });

                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::AdjustPaneSize(AdjustPaneSize {
                pane_id,
                direction,
                amount,
            }) => {
                spawn_into_main_thread(async move {
                    catch(
                        move || {
                            let mux = Mux::get();
                            let (_pane_domain_id, _window_id, tab_id) = mux
                                .resolve_pane_id(pane_id)
                                .ok_or_else(|| anyhow!("pane_id {} invalid", pane_id))?;

                            let tab = match mux.get_tab(tab_id) {
                                Some(tab) => tab,
                                None => {
                                    return Err(anyhow!(
                                        "Failed to retrieve tab with ID {}",
                                        tab_id
                                    ))
                                }
                            };

                            tab.adjust_pane_size(direction, amount);
                            Ok(Pdu::UnitResponse(UnitResponse {}))
                        },
                        send_response,
                    )
                })
                .detach();
            }

            Pdu::Invalid { .. } => send_response(Err(anyhow!("invalid PDU {:?}", decoded.pdu))),
            Pdu::Pong { .. }
            | Pdu::ListPanesResponse { .. }
            | Pdu::SetClipboard { .. }
            | Pdu::NotifyAlert { .. }
            | Pdu::SpawnResponse { .. }
            | Pdu::GetPaneRenderChangesResponse { .. }
            | Pdu::UnitResponse { .. }
            | Pdu::LivenessResponse { .. }
            | Pdu::GetPaneDirectionResponse { .. }
            | Pdu::SearchScrollbackResponse { .. }
            | Pdu::GetLinesResponse { .. }
            | Pdu::GetCodecVersionResponse { .. }
            | Pdu::WindowWorkspaceChanged { .. }
            | Pdu::GetTlsCredsResponse { .. }
            | Pdu::GetClientListResponse { .. }
            | Pdu::PaneRemoved { .. }
            | Pdu::PaneFocused { .. }
            | Pdu::TabResized { .. }
            | Pdu::GetImageCellResponse { .. }
            | Pdu::MovePaneToNewTabResponse { .. }
            | Pdu::TabAddedToWindow { .. }
            | Pdu::GetPaneRenderableDimensionsResponse { .. }
            | Pdu::CreateNotificationResponse { .. }
            | Pdu::ListNotificationsResponse { .. }
            | Pdu::ClearNotificationsResponse { .. }
            | Pdu::MarkNotificationsResponse { .. }
            | Pdu::JumpUnreadResponse { .. }
            | Pdu::IdentifyNotificationTargetResponse { .. }
            | Pdu::GetNotificationCapabilitiesResponse { .. }
            | Pdu::SetWorkspaceStatusResponse { .. }
            | Pdu::ClearWorkspaceStatusResponse { .. }
            | Pdu::ListWorkspaceStatusResponse { .. }
            | Pdu::SetWorkspaceProgressResponse { .. }
            | Pdu::ClearWorkspaceProgressResponse { .. }
            | Pdu::AppendWorkspaceLogResponse { .. }
            | Pdu::ClearWorkspaceLogResponse { .. }
            | Pdu::ListWorkspaceLogResponse { .. }
            | Pdu::ListTaskPanesResponse { .. }
            | Pdu::SetRemainOnExitResponse { .. }
            | Pdu::RerunPaneResponse { .. }
            | Pdu::RespawnPaneResponse { .. }
            | Pdu::SilenceWatchdogResponse { .. }
            | Pdu::PipePaneResponse { .. }
            | Pdu::TaskPaneChanged { .. }
            | Pdu::ErrorResponse { .. } => {
                send_response(Err(anyhow!("expected a request, got {:?}", decoded.pdu)))
            }
        }
    }
}

fn schedule_rerun_task_pane<SND>(request: RerunPane, send_response: SND)
where
    SND: FnOnce(anyhow::Result<Pdu>) + 'static + Send,
{
    promise::spawn::spawn(async move { send_response(rerun_task_pane(request).await) }).detach();
}

async fn rerun_task_pane(request: RerunPane) -> anyhow::Result<Pdu> {
    let mux = Mux::get();
    let (spawned_pane_id, status) = mux.rerun_task_pane(request.pane_id).await?;
    Ok(Pdu::RerunPaneResponse(RerunPaneResponse {
        pane_id: request.pane_id,
        spawned_pane_id,
        status,
    }))
}

fn schedule_respawn_task_pane<SND>(request: RespawnPane, send_response: SND)
where
    SND: FnOnce(anyhow::Result<Pdu>) + 'static + Send,
{
    promise::spawn::spawn(async move { send_response(respawn_task_pane(request).await) }).detach();
}

async fn respawn_task_pane(request: RespawnPane) -> anyhow::Result<Pdu> {
    let mux = Mux::get();
    let (spawned_pane_id, status) = mux.respawn_task_pane(request.pane_id).await?;
    Ok(Pdu::RespawnPaneResponse(RespawnPaneResponse {
        pane_id: request.pane_id,
        spawned_pane_id,
        status,
    }))
}

// Dancing around a little bit here; we can't directly spawn_into_main_thread the domain_spawn
// function below because the compiler thinks that all of its locals then need to be Send.
// We need to shimmy through this helper to break that aspect of the compiler flow
// analysis and allow things to compile.
fn schedule_domain_spawn_v2<SND>(
    spawn: SpawnV2,
    send_response: SND,
    client_id: Option<Arc<ClientId>>,
) where
    SND: Fn(anyhow::Result<Pdu>) + 'static,
{
    promise::spawn::spawn(async move { send_response(domain_spawn_v2(spawn, client_id).await) })
        .detach();
}

fn schedule_split_pane<SND>(split: SplitPane, send_response: SND, client_id: Option<Arc<ClientId>>)
where
    SND: Fn(anyhow::Result<Pdu>) + 'static,
{
    promise::spawn::spawn(async move { send_response(split_pane(split, client_id).await) })
        .detach();
}

async fn split_pane(split: SplitPane, client_id: Option<Arc<ClientId>>) -> anyhow::Result<Pdu> {
    let mux = Mux::get();
    let _identity = mux.with_identity(client_id);

    let (_pane_domain_id, window_id, tab_id) = mux
        .resolve_pane_id(split.pane_id)
        .ok_or_else(|| anyhow!("pane_id {} invalid", split.pane_id))?;

    let source = if let Some(move_pane_id) = split.move_pane_id {
        SplitSource::MovePane(move_pane_id)
    } else {
        SplitSource::Spawn {
            command: split.command,
            command_dir: split.command_dir,
        }
    };

    let (pane, size) = mux
        .split_pane(split.pane_id, split.split_request, source, split.domain)
        .await?;

    Ok::<Pdu, anyhow::Error>(Pdu::SpawnResponse(SpawnResponse {
        pane_id: pane.pane_id(),
        tab_id,
        window_id,
        size,
    }))
}

async fn domain_spawn_v2(spawn: SpawnV2, client_id: Option<Arc<ClientId>>) -> anyhow::Result<Pdu> {
    let mux = Mux::get();
    let _identity = mux.with_identity(client_id);

    let (tab, pane, window_id) = mux
        .spawn_tab_or_window(
            spawn.window_id,
            spawn.domain,
            spawn.command,
            spawn.command_dir,
            None,
            spawn.size,
            None, // optional current pane_id
            spawn.workspace,
            None, // optional gui window position
        )
        .await?;

    Ok::<Pdu, anyhow::Error>(Pdu::SpawnResponse(SpawnResponse {
        pane_id: pane.pane_id(),
        tab_id: tab.tab_id(),
        window_id,
        size: tab.get_size(),
    }))
}

fn schedule_move_pane<SND>(
    request: MovePaneToNewTab,
    send_response: SND,
    client_id: Option<Arc<ClientId>>,
) where
    SND: Fn(anyhow::Result<Pdu>) + 'static,
{
    promise::spawn::spawn(async move { send_response(move_pane(request, client_id).await) })
        .detach();
}

async fn move_pane(
    request: MovePaneToNewTab,
    client_id: Option<Arc<ClientId>>,
) -> anyhow::Result<Pdu> {
    let mux = Mux::get();
    let _identity = mux.with_identity(client_id);

    let (tab, window_id) = mux
        .move_pane_to_new_tab(
            request.pane_id,
            request.window_id,
            request.workspace_for_new_window,
        )
        .await?;

    Ok::<Pdu, anyhow::Error>(Pdu::MovePaneToNewTabResponse(MovePaneToNewTabResponse {
        tab_id: tab.tab_id(),
        window_id,
    }))
}
