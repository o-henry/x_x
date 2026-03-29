use crate::pane::PaneId;
use crate::tab::TabId;
use crate::window::WindowId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotificationUnreadMode {
    ClearOnFocus,
    Sticky,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub notification_id: String,
    pub workspace: String,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub unread: bool,
    pub unread_mode: NotificationUnreadMode,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NotificationCreateRequest {
    pub notification_id: String,
    pub workspace: String,
    pub window_id: Option<WindowId>,
    pub tab_id: Option<TabId>,
    pub pane_id: Option<PaneId>,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub unread_mode: NotificationUnreadMode,
}

impl NotificationCreateRequest {
    pub fn into_record(self) -> NotificationRecord {
        let now = utc_now();
        NotificationRecord {
            notification_id: self.notification_id,
            workspace: self.workspace,
            window_id: self.window_id,
            tab_id: self.tab_id,
            pane_id: self.pane_id,
            kind: self.kind,
            title: self.title,
            body: self.body,
            unread: true,
            unread_mode: self.unread_mode,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Clone, Debug)]
struct NotificationEntry {
    record: NotificationRecord,
    anchor_pane_id: PaneId,
}

#[derive(Default)]
pub struct NotificationStore {
    notifications: HashMap<String, NotificationEntry>,
    notification_order: Vec<String>,
    unread_by_anchor: HashMap<PaneId, BTreeSet<String>>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_notification(&mut self, record: NotificationRecord, anchor_pane_id: PaneId) {
        let notification_id = record.notification_id.clone();
        self.remove_notification(&notification_id);
        self.notification_order.push(notification_id.clone());
        let unread = record.unread;
        self.notifications.insert(
            notification_id.clone(),
            NotificationEntry {
                record,
                anchor_pane_id,
            },
        );
        if unread {
            self.add_to_unread_index(anchor_pane_id, &notification_id);
        }
    }

    pub fn list_notifications(&self) -> Vec<NotificationRecord> {
        self.notification_order
            .iter()
            .filter_map(|notification_id| {
                self.notifications
                    .get(notification_id)
                    .map(|entry| entry.record.clone())
            })
            .collect()
    }

    pub fn clear_notifications(&mut self, notification_ids: &[String]) -> usize {
        let notification_ids = self.normalize_notification_ids(notification_ids);
        let cleared = notification_ids
            .iter()
            .filter(|notification_id| self.notifications.contains_key(*notification_id))
            .count();
        for notification_id in notification_ids {
            self.remove_notification(&notification_id);
        }
        cleared
    }

    pub fn mark_notifications_read(&mut self, notification_ids: &[String]) -> usize {
        let now = utc_now();
        let mut changed = 0;
        for notification_id in notification_ids {
            let anchor_pane_id = if let Some(entry) = self.notifications.get_mut(notification_id) {
                if entry.record.unread {
                    entry.record.unread = false;
                    entry.record.updated_at = now;
                    Some(entry.anchor_pane_id)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(anchor_pane_id) = anchor_pane_id {
                self.remove_from_unread_index(anchor_pane_id, notification_id);
                changed += 1;
            }
        }
        changed
    }

    pub fn mark_notifications_unread(&mut self, notification_ids: &[String]) -> usize {
        let now = utc_now();
        let mut changed = 0;
        for notification_id in notification_ids {
            let anchor_pane_id = if let Some(entry) = self.notifications.get_mut(notification_id) {
                if !entry.record.unread {
                    entry.record.unread = true;
                    entry.record.updated_at = now;
                    Some(entry.anchor_pane_id)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(anchor_pane_id) = anchor_pane_id {
                self.add_to_unread_index(anchor_pane_id, notification_id);
                changed += 1;
            }
        }
        changed
    }

    pub fn apply_pane_focused(&mut self, pane_id: PaneId) -> usize {
        let now = utc_now();
        let notification_ids = self
            .unread_by_anchor
            .get(&pane_id)
            .map(|ids| ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut changed = 0;
        for notification_id in notification_ids {
            if let Some(entry) = self.notifications.get_mut(&notification_id) {
                if entry.record.unread
                    && matches!(
                        entry.record.unread_mode,
                        NotificationUnreadMode::ClearOnFocus
                    )
                {
                    entry.record.unread = false;
                    entry.record.updated_at = now;
                    self.remove_from_unread_index(pane_id, &notification_id);
                    changed += 1;
                }
            }
        }
        changed
    }

    pub fn unread_count_for_tab(
        &self,
        _tab_id: TabId,
        workspace: &str,
        pane_ids: &[PaneId],
    ) -> usize {
        let pane_ids: HashSet<PaneId> = pane_ids.iter().copied().collect();
        self.notifications
            .values()
            .filter(|entry| {
                entry.record.unread
                    && (pane_ids.contains(&entry.anchor_pane_id)
                        || (entry.record.workspace == workspace
                            && entry.record.window_id.is_none()
                            && entry.record.tab_id.is_none()
                            && entry.record.pane_id.is_none()))
            })
            .count()
    }

    pub fn unread_count_for_workspace(&self, workspace: &str) -> usize {
        self.notifications
            .values()
            .filter(|entry| entry.record.unread && entry.record.workspace == workspace)
            .count()
    }

    pub fn next_unread_pane(
        &self,
        pane_order: &[PaneId],
        current: Option<PaneId>,
    ) -> Option<PaneId> {
        self.navigate_unread_pane(pane_order, current, true)
    }

    pub fn prev_unread_pane(
        &self,
        pane_order: &[PaneId],
        current: Option<PaneId>,
    ) -> Option<PaneId> {
        self.navigate_unread_pane(pane_order, current, false)
    }

    pub fn anchor_pane_id_for_notification(&self, notification_id: &str) -> Option<PaneId> {
        self.notifications
            .get(notification_id)
            .map(|entry| entry.anchor_pane_id)
    }

    pub fn rename_workspace(&mut self, old_workspace: &str, new_workspace: &str) -> usize {
        let mut changed = 0;
        for entry in self.notifications.values_mut() {
            if entry.record.workspace == old_workspace {
                entry.record.workspace = new_workspace.to_string();
                entry.record.updated_at = utc_now();
                changed += 1;
            }
        }
        changed
    }

    fn navigate_unread_pane(
        &self,
        pane_order: &[PaneId],
        current: Option<PaneId>,
        forward: bool,
    ) -> Option<PaneId> {
        let unread_anchors = self.unread_anchor_panes_in_order(pane_order);
        if unread_anchors.is_empty() {
            return None;
        }

        if let Some(current) = current {
            if let Some(position) = pane_order.iter().position(|pane_id| *pane_id == current) {
                if forward {
                    for pane_id in pane_order.iter().skip(position + 1) {
                        if unread_anchors.contains(pane_id) {
                            return Some(*pane_id);
                        }
                    }
                    for pane_id in pane_order.iter().take(position + 1) {
                        if unread_anchors.contains(pane_id) {
                            return Some(*pane_id);
                        }
                    }
                } else {
                    for pane_id in pane_order.iter().take(position).rev() {
                        if unread_anchors.contains(pane_id) {
                            return Some(*pane_id);
                        }
                    }
                    for pane_id in pane_order.iter().skip(position).rev() {
                        if unread_anchors.contains(pane_id) {
                            return Some(*pane_id);
                        }
                    }
                }
                return None;
            }
        }

        if forward {
            unread_anchors.first().copied()
        } else {
            unread_anchors.last().copied()
        }
    }

    fn unread_anchor_panes_in_order(&self, pane_order: &[PaneId]) -> Vec<PaneId> {
        pane_order
            .iter()
            .copied()
            .filter(|pane_id| {
                self.unread_by_anchor
                    .get(pane_id)
                    .map(|notifications| !notifications.is_empty())
                    .unwrap_or(false)
            })
            .collect()
    }

    fn normalize_notification_ids(&self, notification_ids: &[String]) -> Vec<String> {
        if notification_ids.is_empty() {
            self.notification_order.clone()
        } else {
            notification_ids.to_vec()
        }
    }

    fn remove_notification(&mut self, notification_id: &str) -> Option<NotificationEntry> {
        let entry = self.notifications.remove(notification_id)?;
        self.notification_order
            .retain(|existing_id| existing_id != notification_id);
        self.remove_from_unread_index(entry.anchor_pane_id, notification_id);
        Some(entry)
    }

    fn add_to_unread_index(&mut self, anchor_pane_id: PaneId, notification_id: &str) {
        self.unread_by_anchor
            .entry(anchor_pane_id)
            .or_default()
            .insert(notification_id.to_string());
    }

    fn remove_from_unread_index(&mut self, anchor_pane_id: PaneId, notification_id: &str) {
        if let Some(unread_ids) = self.unread_by_anchor.get_mut(&anchor_pane_id) {
            unread_ids.remove(notification_id);
            if unread_ids.is_empty() {
                self.unread_by_anchor.remove(&anchor_pane_id);
            }
        }
    }
}

fn utc_now() -> DateTime<Utc> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    DateTime::from_timestamp(now.as_secs() as i64, now.subsec_nanos())
        .expect("system time out of range")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(
        notification_id: &str,
        workspace: &str,
        window_id: Option<WindowId>,
        tab_id: Option<TabId>,
        pane_id: Option<PaneId>,
        unread_mode: NotificationUnreadMode,
    ) -> NotificationRecord {
        let now = utc_now();
        NotificationRecord {
            notification_id: notification_id.to_string(),
            workspace: workspace.to_string(),
            window_id,
            tab_id,
            pane_id,
            kind: "task".to_string(),
            title: format!("Notification {notification_id}"),
            body: Some(format!("Body {notification_id}")),
            unread: true,
            unread_mode,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn creating_notifications_preserves_scope_fields_for_all_supported_scopes() {
        let mut store = NotificationStore::new();

        let workspace_only = sample_record(
            "workspace-only",
            "alpha",
            None,
            None,
            None,
            NotificationUnreadMode::Sticky,
        );
        let tab_only = sample_record(
            "tab-only",
            "alpha",
            Some(11),
            Some(TabId::new(21)),
            None,
            NotificationUnreadMode::ClearOnFocus,
        );
        let pane_scoped = sample_record(
            "pane-scoped",
            "alpha",
            Some(11),
            Some(TabId::new(21)),
            Some(PaneId::new(301)),
            NotificationUnreadMode::Sticky,
        );

        store.create_notification(workspace_only.clone(), PaneId::new(301));
        store.create_notification(tab_only.clone(), PaneId::new(302));
        store.create_notification(pane_scoped.clone(), PaneId::new(303));

        let notifications = store.list_notifications();
        assert_eq!(notifications.len(), 3);

        let stored_workspace = notifications
            .iter()
            .find(|record| record.notification_id == workspace_only.notification_id)
            .unwrap();
        assert_eq!(stored_workspace.workspace, workspace_only.workspace);
        assert_eq!(stored_workspace.window_id, None);
        assert_eq!(stored_workspace.tab_id, None);
        assert_eq!(stored_workspace.pane_id, None);

        let stored_tab = notifications
            .iter()
            .find(|record| record.notification_id == tab_only.notification_id)
            .unwrap();
        assert_eq!(stored_tab.window_id, tab_only.window_id);
        assert_eq!(stored_tab.tab_id, tab_only.tab_id);
        assert_eq!(stored_tab.pane_id, None);

        let stored_pane = notifications
            .iter()
            .find(|record| record.notification_id == pane_scoped.notification_id)
            .unwrap();
        assert_eq!(stored_pane.window_id, pane_scoped.window_id);
        assert_eq!(stored_pane.tab_id, pane_scoped.tab_id);
        assert_eq!(stored_pane.pane_id, pane_scoped.pane_id);
    }

    #[test]
    fn clear_on_focus_marks_only_clear_on_focus_records_as_read() {
        let mut store = NotificationStore::new();
        let clear = sample_record(
            "clear",
            "alpha",
            Some(11),
            Some(TabId::new(21)),
            Some(PaneId::new(301)),
            NotificationUnreadMode::ClearOnFocus,
        );
        let sticky = sample_record(
            "sticky",
            "alpha",
            Some(11),
            Some(TabId::new(21)),
            Some(PaneId::new(301)),
            NotificationUnreadMode::Sticky,
        );

        store.create_notification(clear.clone(), PaneId::new(301));
        store.create_notification(sticky.clone(), PaneId::new(301));

        assert_eq!(store.apply_pane_focused(PaneId::new(301)), 1);

        let notifications = store.list_notifications();
        let clear = notifications
            .iter()
            .find(|record| record.notification_id == "clear")
            .unwrap();
        let sticky = notifications
            .iter()
            .find(|record| record.notification_id == "sticky")
            .unwrap();

        assert!(!clear.unread);
        assert!(sticky.unread);
    }

    #[test]
    fn unread_navigation_follows_existing_pane_iteration_order_for_derived_scopes() {
        let mut store = NotificationStore::new();
        store.create_notification(
            sample_record(
                "workspace-derived",
                "alpha",
                None,
                None,
                None,
                NotificationUnreadMode::Sticky,
            ),
            PaneId::new(401),
        );
        store.create_notification(
            sample_record(
                "tab-derived",
                "alpha",
                Some(11),
                Some(TabId::new(22)),
                None,
                NotificationUnreadMode::Sticky,
            ),
            PaneId::new(402),
        );
        store.create_notification(
            sample_record(
                "pane-direct",
                "alpha",
                Some(11),
                Some(TabId::new(22)),
                Some(PaneId::new(403)),
                NotificationUnreadMode::Sticky,
            ),
            PaneId::new(403),
        );

        let pane_order = [
            PaneId::new(400),
            PaneId::new(401),
            PaneId::new(402),
            PaneId::new(403),
        ];

        assert_eq!(
            store.next_unread_pane(&pane_order, Some(PaneId::new(400))),
            Some(PaneId::new(401))
        );
        assert_eq!(
            store.next_unread_pane(&pane_order, Some(PaneId::new(401))),
            Some(PaneId::new(402))
        );
        assert_eq!(
            store.prev_unread_pane(&pane_order, Some(PaneId::new(403))),
            Some(PaneId::new(402))
        );
        assert_eq!(
            store.prev_unread_pane(&pane_order, Some(PaneId::new(402))),
            Some(PaneId::new(401))
        );
    }

    #[test]
    fn unread_count_for_tab_includes_workspace_only_notifications_for_same_workspace() {
        let mut store = NotificationStore::new();
        store.create_notification(
            sample_record(
                "workspace-only",
                "alpha",
                None,
                None,
                None,
                NotificationUnreadMode::Sticky,
            ),
            PaneId::new(401),
        );
        store.create_notification(
            sample_record(
                "other-tab-pane",
                "alpha",
                Some(11),
                Some(TabId::new(22)),
                Some(PaneId::new(402)),
                NotificationUnreadMode::Sticky,
            ),
            PaneId::new(402),
        );

        assert_eq!(
            store.unread_count_for_tab(TabId::new(21), "alpha", &[PaneId::new(999)]),
            1
        );
        assert_eq!(
            store.unread_count_for_tab(TabId::new(21), "beta", &[PaneId::new(999)]),
            0
        );
    }
}
