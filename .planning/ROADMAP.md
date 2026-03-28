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
- [x] **Phase 6: Operator UI Surfaces** - Task Center, compact tabbar discoverability, and prompt/confirm metadata editing now make the control plane operable from native Kaku UI without a dashboard rewrite.
- [ ] **Phase 7: Operator Visual Polish** - Bring the shipped native operator UI closer to the provided video reference with DM Mono defaults, icon-first affordances, and more intentional composition while staying inside native Kaku surfaces.

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
- [x] 06-01-PLAN.md — Add the Phase 06 controller seam: scoped Task Center bootstrap plus typed workspace metadata read/write helpers in TermWindow and the client-domain transport.
- [x] 06-02-PLAN.md — Upgrade Task Center into the primary operator surface with visible inline actions and keyboard+mouse parity.
- [x] 06-03-PLAN.md — Add compact tabbar operator affordances and click-through discovery into scoped Task Center views.
- [x] 06-04-PLAN.md — Close Phase 06 with targeted verification, changed-files/limitations/UAT artifacts, and shipped-shape spec updates.
**UI hint**: yes

### Phase 7: Operator Visual Polish
**Goal**: Bring the native operator UI materially closer to the provided video reference without breaking Kaku's additive Rust-native architecture.
**Depends on**: Phase 6
**Requirements**: POLISH-01, POLISH-02, POLISH-03, POLISH-04
**Success Criteria** (what must be TRUE):
  1. Developer sees DM Mono as the default operator-facing mono typography in the shipped UI surfaces.
  2. Developer sees compact icon-first state/action affordances instead of text-heavy markers where clarity benefits.
  3. Task Center and tabbar feel more intentional, composed, and reference-aligned while remaining native Kaku surfaces rather than a dashboard shell.
  4. Existing Phase 6 operator behavior still works after the visual refresh.
**Plans**: 5 plans
Plans:
- [ ] 07-01-PLAN.md — Rebuild the persistent left rail into a slim edge-attached native navigation surface without regressing the scoped Task Center workflow.
- [ ] 07-02-PLAN.md — Rewrite the top chrome and tab/title composition into quieter DM Mono-first app chrome with compact icon-first markers.
- [ ] 07-03-PLAN.md — Rebalance the main work-surface hierarchy and stop for a structural screenshot gate before any overlay-only polish.
- [ ] 07-04-PLAN.md — Refine Task Center only as a secondary operator surface after the persistent shell structure is approved.
- [ ] 07-05-PLAN.md — Close Phase 07 with regression evidence, screenshot-based UAT, and honest docs on whether the current renderer was sufficient.
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Notification Routing | 0/4 | Not started | - |
| 2. Workspace Metadata Plane | 0/5 | Not started | - |
| 3. Task Center Overlay | 0/TBD | Not started | - |
| 4. Task-Pane Lifecycle | 0/TBD | Not started | - |
| 5. Hardening and Compatibility | 5/5 | Complete | 2026-03-27 |
| 6. Operator UI Surfaces | 4/4 | Complete | 2026-03-27 |
| 7. Operator Visual Polish | 0/TBD | Not started | - |

### Phase 8: Native Shell Replatform

**Goal:** Replace the current primary UI shell path with a Rust-native native shell that presents the existing Kaku control plane through a persistent reference-matched workspace shell layout.
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05, POLISH-01, POLISH-02, POLISH-03, POLISH-04
**Depends on:** Phase 6 (Phase 8 supersedes the unfinished visual-polish remainder of Phase 7)
**Plans:** 5 plans
**Success Criteria** (what must be TRUE):
  1. Developer can launch a Rust-native shell that boots or owns the shared Kaku runtime path without requiring `kaku-gui` to be the primary shell authority.
  2. Developer sees a persistent shell layout with left workspace rail, thin top chrome, dominant main work surface, and persistent context panes that materially matches the provided references more closely than the old `kaku-gui` renderer path.
  3. Developer can inspect and mutate the existing Kaku control-plane state from the new shell, including workspace status/progress/log data plus notification/task context.
  4. Existing machine-readable contracts and mux-owned state remain the backend truth; the new shell does not introduce a parallel control-plane store.
  5. MVP may bridge terminal rendering through companion Kaku windows, but that bridge must be proven end-to-end on the shared runtime path before Phase 08 can close as complete.

Plans:
- [ ] 08-01-PLAN.md — Extract Wave 0 shell controller/runtime seams and move bootstrap authority into shared runtime for the native shell path.
- [ ] 08-02-PLAN.md — Replace timer polling with mux-driven event subscriptions and a reusable shell snapshot/action architecture.
- [ ] 08-03-PLAN.md — Build the persistent reference-matched shell layout with real runtime data, DM Mono-first styling, and a blocking screenshot gate.
- [ ] 08-04-PLAN.md — Replace the risky `--always-new-process` escape hatch with an honest companion-terminal bridge and common operator action parity.
- [ ] 08-05-PLAN.md — Close Phase 08 with full validation, manual UAT, honest stop conditions, and canonical docs/artifacts.
