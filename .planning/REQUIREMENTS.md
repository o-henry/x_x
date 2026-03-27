# Requirements: Kaku Agent Control Plane

**Defined:** 2026-03-26
**Core Value:** A solo Unity developer can see which pane needs attention, keep failed work visible, and recover or navigate quickly without leaving Kaku's existing UX model.

## v1 Requirements

### Notifications

- [ ] **NOTF-01**: Developer can create notification records scoped to an existing workspace, tab, or pane.
- [ ] **NOTF-02**: Developer can list notifications through stable machine-readable CLI output.
- [ ] **NOTF-03**: Developer can clear, mark read, and mark unread notification records through stable CLI commands.
- [ ] **NOTF-04**: Developer can jump to the next unread target across background panes.
- [ ] **NOTF-05**: Developer can jump to the previous unread target across background panes.
- [ ] **NOTF-06**: Developer can see unread visual markers on existing Kaku surfaces without changing the overall look and feel.

### Workspace Metadata

- [x] **META-01**: Developer can set a workspace-scoped status value through the CLI.
- [x] **META-02**: Developer can clear and list workspace-scoped status values through the CLI.
- [x] **META-03**: Developer can set and clear workspace-scoped progress values through the CLI.
- [x] **META-04**: Developer can append workspace-scoped log entries and list or clear them through the CLI.
- [ ] **META-05**: Developer can see workspace status and progress on existing Kaku UI surfaces without adding a new app shell.

### Task Center

- [ ] **TASK-01**: Developer can open a searchable Task Center overlay that aggregates workspaces, tabs, panes, unread items, failed tasks, and running tasks.
- [ ] **TASK-02**: Developer can filter Task Center results by unread, failed, running, workspace, source, and kind.
- [ ] **TASK-03**: Developer can focus a selected target directly from Task Center.
- [ ] **TASK-04**: Developer can clear unread state from Task Center.
- [ ] **TASK-05**: Developer can rerun a failed pane from Task Center when rerun data is available.

### Task Lifecycle

- [ ] **LIFE-01**: Developer can keep selected task panes visible after process exit with remain-on-exit semantics.
- [ ] **LIFE-02**: Developer can rerun or respawn a failed task pane without manually reconstructing the pane setup.
- [ ] **LIFE-03**: Developer can silence watchdog behavior for selected task panes.
- [ ] **LIFE-04**: Developer can pipe pane output to a file while preserving normal pane behavior.
- [ ] **LIFE-05**: Developer can retain enough pane lifecycle metadata to identify failed panes and surface rerun affordances.

### Compatibility and Hardening

- [x] **COMP-01**: Developer can continue using existing Kaku CLI pane-management commands without behavior regressions.
- [x] **COMP-02**: Developer can continue using existing pane/tab creation, splitting, navigation, focus, and workspace switching without behavior regressions.
- [x] **COMP-03**: Developer can rely on stable machine-readable contracts for each new CLI surface added by this control plane.
- [x] **COMP-04**: Developer can verify each phase with targeted tests or checks before the phase is considered done.
- [x] **COMP-05**: Developer can use the fork without it feeling like a different app shell or a cmux clone.

## v2 Requirements

### Operator UI

- [x] **UI-01**: Developer can discover core control-plane actions from native Kaku UI without remembering CLI subcommands.
- [x] **UI-02**: Developer can inspect notification, unread, failed-task, and workspace status context through mouse-friendly and keyboard-friendly UI surfaces.
- [x] **UI-03**: Developer can trigger common lifecycle and attention actions from the UI, including focus, clear unread, rerun/respawn, and remain-on-exit style toggles where applicable.
- [x] **UI-04**: Developer can update workspace-facing metadata through UI affordances for the common path, with CLI preserved as an advanced fallback rather than the primary path.
- [x] **UI-05**: New control-plane UI remains additive and Kaku-native, avoiding a dashboard-shell rewrite while still feeling more operable than a CLI-only workflow.

### Future Expansion

- **FUTR-01**: Developer can customize notification and workspace-metadata presentation more deeply after the core control plane is stable.
- **FUTR-02**: Developer can evaluate broader session-management semantics only after the Kaku-native v1 lifecycle model is proven.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Embedded browser | Explicitly excluded from v1 and not needed for the core control-plane workflow |
| PR or GitHub UI | Not part of the local Kaku control-plane value |
| Port scanner | Outside the solo Unity pane-attention problem |
| Full detached tmux server semantics | Too broad for v1 and would pull the product away from Kaku |
| Large visual redesign | The result must preserve Kaku's existing feel |
| cmux visual clone | Behavior inspiration is acceptable, but visual/product cloning is not |
| New socket daemon unless clearly required later | Avoid speculative architecture expansion |
| Swift, Xcode, React, Electron, Tauri, or WebView additions | Explicit hard constraint for this repo |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| NOTF-01 | Phase 1 | Pending |
| NOTF-02 | Phase 1 | Pending |
| NOTF-03 | Phase 1 | Pending |
| NOTF-04 | Phase 1 | Pending |
| NOTF-05 | Phase 1 | Pending |
| NOTF-06 | Phase 1 | Pending |
| META-01 | Phase 2 | Complete |
| META-02 | Phase 2 | Complete |
| META-03 | Phase 2 | Complete |
| META-04 | Phase 2 | Complete |
| META-05 | Phase 2 | Pending |
| TASK-01 | Phase 3 | Pending |
| TASK-02 | Phase 3 | Pending |
| TASK-03 | Phase 3 | Pending |
| TASK-04 | Phase 3 | Pending |
| TASK-05 | Phase 3 | Pending |
| LIFE-01 | Phase 4 | Pending |
| LIFE-02 | Phase 4 | Pending |
| LIFE-03 | Phase 4 | Pending |
| LIFE-04 | Phase 4 | Pending |
| LIFE-05 | Phase 4 | Pending |
| COMP-01 | Phase 5 | Complete |
| COMP-02 | Phase 5 | Complete |
| COMP-03 | Phase 5 | Complete |
| COMP-04 | Phase 5 | Complete |
| COMP-05 | Phase 5 | Complete |
| UI-01 | Phase 6 | Complete |
| UI-02 | Phase 6 | Complete |
| UI-03 | Phase 6 | Complete |
| UI-04 | Phase 6 | Complete |
| UI-05 | Phase 6 | Complete |

**Coverage:**
- v1/v2 requirements: 31 total
- Mapped to phases: 31
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-26*
*Last updated: 2026-03-26 after initial definition*
