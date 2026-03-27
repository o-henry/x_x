# Roadmap: Kaku Agent Control Plane

## Overview

This roadmap turns the existing local Kaku fork into a Rust-only control plane for solo Unity development without changing Kaku's core feel. The sequence follows the canonical spec and current brownfield plan: first establish mux-owned unread routing and notification contracts, then add workspace metadata, then expose that state through a Task Center overlay, then add task-pane lifecycle controls, and finally harden the whole slice without regressing existing Kaku behavior.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: Notification Routing** - Add mux-owned notifications, unread indexes, stable CLI commands, and tab/pane markers.
- [ ] **Phase 2: Workspace Metadata Plane** - Add workspace-scoped status, progress, and log storage with existing-surface UI exposure.
- [ ] **Phase 3: Task Center Overlay** - Add a searchable overlay for unread, failed, running, and focusable work.
- [ ] **Phase 4: Task-Pane Lifecycle** - Add remain-on-exit, rerun/respawn, watchdog silence, and pipe-pane behavior.
- [x] **Phase 5: Hardening and Compatibility** - Lock down command contracts, tests, docs, and regression safety.
- [ ] **Phase 6: Operator UI Surfaces** - Make the control plane operable from native Kaku UI with mouse/keyboard-first flows instead of CLI recall.

## Phase Details

### Phase 1: Notification Routing
**Goal**: Background panes can raise and retain unread attention state that is visible and navigable through existing Kaku surfaces.
**Depends on**: Nothing (first phase)
**Requirements**: NOTF-01, NOTF-02, NOTF-03, NOTF-04, NOTF-05, NOTF-06
**Success Criteria** (what must be TRUE):
  1. Developer can create, list, clear, mark read, and mark unread notification records through stable machine-readable CLI commands.
  2. Developer can jump to the next or previous unread target across background panes without losing existing navigation behavior.
  3. Existing tabbar or pane-adjacent surfaces show unread markers in a way that still feels like Kaku rather than a visual redesign.
  4. Notification and unread state is owned by the mux and keyed to existing workspace, tab, and pane identities instead of a parallel state system.
**Plans**: 5 plans
Plans:
- [ ] 01-01-PLAN.md — Build the mux-owned notification store, unread indexes, and next/previous unread routing APIs.
- [ ] 01-02-PLAN.md — Add typed codec/client/server notification RPC contracts including identify and capabilities.
- [ ] 01-03-PLAN.md — Expose dedicated `kaku cli` create/list/mutation commands with targeted JSON-contract tests.
- [ ] 01-04-PLAN.md — Surface unread markers on the existing tabbar using mux-backed unread counts.
- [ ] 01-05-PLAN.md — Finish jump/discovery CLI commands, document the contract, and close the phase with formatting/tests/limitations coverage.
**UI hint**: yes

### Phase 2: Workspace Metadata Plane
**Goal**: Workspaces can hold status, progress, and log metadata that is writable from the CLI and visible on existing Kaku surfaces.
**Depends on**: Phase 1
**Requirements**: META-01, META-02, META-03, META-04, META-05
**Success Criteria** (what must be TRUE):
  1. Developer can set, clear, and list workspace status through stable CLI commands.
  2. Developer can set and clear workspace progress values through stable CLI commands.
  3. Developer can append, list, and clear workspace-scoped log entries through stable CLI commands.
  4. Existing Kaku UI surfaces such as the tabbar, right-status equivalent, or overlays can show workspace status/progress without introducing a new app shell.
**Plans**: 5 plans
Plans:
- [ ] 02-01-PLAN.md — Build the mux-owned workspace metadata store, rename-safe wrappers, and `WorkspaceMetadataChanged` event.
- [ ] 02-02-PLAN.md — Add typed codec/client/server workspace metadata contracts for status, progress, and logs.
- [ ] 02-03-PLAN.md — Expose dedicated `kaku cli` workspace metadata commands with targeted contract tests.
- [ ] 02-04-PLAN.md — Surface workspace status and progress on the existing Kaku tab/title path with mux-driven refreshes.
- [ ] 02-05-PLAN.md — Close Phase 2 with regression coverage, command-contract docs, changed-files, and limitations artifacts.
**UI hint**: yes

### Phase 3: Task Center Overlay
**Goal**: Developer can use one Kaku-native overlay to find and act on unread, failed, running, and workspace-scoped work quickly.
**Depends on**: Phase 2
**Requirements**: TASK-01, TASK-02, TASK-03, TASK-04, TASK-05
**Success Criteria** (what must be TRUE):
  1. Developer can open a searchable Task Center overlay that aggregates workspaces, tabs, panes, unread items, failed tasks, and running tasks.
  2. Developer can filter Task Center entries by unread, failed, running, workspace, source, and kind.
  3. Developer can focus a selected result or clear its unread state directly from the overlay.
  4. Developer can rerun a failed pane from the overlay when rerun metadata exists.
**Plans**: TBD
**UI hint**: yes

### Phase 4: Task-Pane Lifecycle
**Goal**: Failed or long-running task panes remain useful after exit and can be rerun, silenced, or tee'd without rebuilding context manually.
**Depends on**: Phase 3
**Requirements**: LIFE-01, LIFE-02, LIFE-03, LIFE-04, LIFE-05
**Success Criteria** (what must be TRUE):
  1. Developer can keep selected task panes visible after process exit with remain-on-exit semantics.
  2. Developer can rerun or respawn a failed task pane without manually reconstructing the original pane setup.
  3. Developer can silence watchdog behavior for selected task panes without changing unrelated pane behavior.
  4. Developer can pipe pane output to a file while preserving normal pane interaction and output flow.
  5. Failed-pane metadata persists long enough for downstream UI surfaces to identify failures and surface rerun affordances.
**Plans**: TBD

### Phase 5: Hardening and Compatibility
**Goal**: The control plane is regression-safe, documented, and still feels like Kaku while preserving the existing pane/tab/workspace baseline.
**Depends on**: Phase 4
**Requirements**: COMP-01, COMP-02, COMP-03, COMP-04, COMP-05
**Success Criteria** (what must be TRUE):
  1. Existing Kaku CLI pane-management commands and core pane/tab/workspace behavior still work without regressions.
  2. Every new CLI surface added by the control plane has a stable machine-readable contract and documented behavior.
  3. Each completed phase has targeted tests or checks plus documented limitations before it is considered done.
  4. The finished fork still feels like Kaku rather than a different shell, large redesign, or cmux clone.
**Plans**: TBD

### Phase 6: Operator UI Surfaces
**Goal**: Developer can operate the control plane from native Kaku UI surfaces without memorizing CLI commands, while preserving Kaku's additive feel.
**Depends on**: Phase 5
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05
**Success Criteria** (what must be TRUE):
  1. Developer can discover the most common control-plane actions from visible Kaku UI surfaces instead of remembering CLI verbs.
  2. Developer can inspect unread, failed, running, and workspace-scoped context through keyboard-friendly and mouse-friendly UI affordances.
  3. Developer can execute the common action path from UI, including focus, clear unread, rerun/respawn, and metadata editing flows where appropriate.
  4. The new surfaces still feel like Kaku overlays, inspectors, menus, or tabbar affordances rather than a separate dashboard shell.
**Plans**: 4 plans
Plans:
- [ ] 06-01-PLAN.md — Add the Phase 06 controller seam: scoped Task Center bootstrap plus typed workspace metadata read/write helpers in TermWindow and the client-domain transport.
- [ ] 06-02-PLAN.md — Upgrade Task Center into the primary operator surface with visible inline actions and keyboard+mouse parity.
- [ ] 06-03-PLAN.md — Add compact tabbar operator affordances and click-through discovery into scoped Task Center views.
- [ ] 06-04-PLAN.md — Close Phase 06 with targeted verification, changed-files/limitations/UAT artifacts, and shipped-shape spec updates.
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Notification Routing | 0/4 | Not started | - |
| 2. Workspace Metadata Plane | 0/5 | Not started | - |
| 3. Task Center Overlay | 0/TBD | Not started | - |
| 4. Task-Pane Lifecycle | 0/TBD | Not started | - |
| 5. Hardening and Compatibility | 5/5 | Complete | 2026-03-27 |
| 6. Operator UI Surfaces | 0/TBD | Not started | - |
