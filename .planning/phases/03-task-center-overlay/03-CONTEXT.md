# Phase 3: Task Center Overlay - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 adds one Kaku-native searchable overlay that aggregates existing mux-backed work signals so a solo developer can quickly find and act on unread, failed, running, and workspace-scoped work without introducing a new app shell or sidebar.

</domain>

<decisions>
## Implementation Decisions

### Overlay shape
- **D-01:** Task Center must reuse Kaku's existing overlay model rather than introducing a sidebar, separate screen, or shell redesign.
- **D-02:** The natural entry path is the existing overlay/launcher command surface and lifecycle, adapted to Task Center semantics.

### Data ownership
- **D-03:** Task Center is a read/act overlay over mux-owned truth only; it must not create a parallel store for notifications, metadata, or pane/task state.
- **D-04:** The overlay aggregates existing sources: workspaces, tabs, panes, unread notifications, and whatever failed/running signals are already available from the current control plane.

### Phase boundary vs Phase 4
- **D-05:** Phase 3 provides the overlay UX skeleton for failed/running work, but complete rerun/remain-on-exit lifecycle reliability belongs to Phase 4.
- **D-06:** Failed/running items may be surfaced in a limited mode if only partial signals exist today, as long as the overlay contract stays honest and Kaku-native.

### Action surface
- **D-07:** Primary Task Center actions are limited to: focus target, clear unread, and rerun failed pane when rerun metadata exists.
- **D-08:** Filtering must cover unread, failed, running, workspace, source, and kind per the spec/roadmap requirements.

### the agent's Discretion
- Ranking and grouping strategy inside the overlay list
- Exact row copy, iconography, and field density
- How much launcher helper code to reuse directly versus lightly adapting the pattern in a dedicated Task Center module

</decisions>

<specifics>
## Specific Ideas

- Keep the visual feel aligned with existing Kaku overlays and command surfaces rather than inventing a new control dashboard.
- Favor filter-first interaction and direct keyboard navigation over mouse-heavy management UI.
- Existing Kaku surfaces should remain the primary shell; Task Center is a focused operational overlay.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product scope
- `docs/KAKU_CONTROL_PLANE_SPEC.md` — Canonical Phase 3 scope, actions, filters, and hard constraints for the local Kaku fork.
- `PLANS.md` — File-level extension-point guidance and additive-change constraints for this repo.

### Roadmap and requirements
- `.planning/ROADMAP.md` — Phase 3 goal, success criteria, and dependency ordering relative to Phases 1, 2, and 4.
- `.planning/REQUIREMENTS.md` — TASK-01 through TASK-05 requirement source of truth.

### Prior phase context
- `.planning/phases/01-notification-routing/01-CONTEXT.md` — Notification ownership and unread-routing decisions that feed Task Center unread sources.
- `.planning/phases/02-workspace-metadata-plane/02-CONTEXT.md` — Workspace metadata ownership and existing-surface constraints that Task Center must build on rather than replace.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `kaku-gui/src/overlay/launcher.rs`: closest existing searchable overlay pattern for filtering, row selection, and keyboard-driven actions.
- `kaku-gui/src/overlay/mod.rs`: existing overlay bootstrap/cancellation surface to reuse rather than inventing a new manager.
- `kaku-gui/src/commands.rs`: natural place to add a native Task Center entry command for palette/keybinding integration.

### Established Patterns
- `kaku-gui/src/termwindow/mod.rs`: current GUI state hub for mux notifications, overlay assignment, and per-window cached snapshots.
- `mux/src/notification_store.rs`: canonical unread notification source created in Phase 1.
- `mux/src/workspace_state.rs`: canonical workspace status/progress/log source created in Phase 2.

### Integration Points
- Task Center should consume mux-backed notification and workspace state through existing GUI client/snapshot paths instead of recomputing or polling.
- Any rerun affordance must tolerate partial lifecycle support in Phase 3 and defer full semantics to Phase 4.

</code_context>

<deferred>
## Deferred Ideas

- A full UI-SPEC/visual contract can be added later if Phase 3 execution exposes ambiguous overlay design choices.
- Robust failed-pane rerun/remain-on-exit/watchdog semantics stay in Phase 4.
- Any broader dashboard/task-management shell is out of scope for this phase.

</deferred>

---

*Phase: 03-task-center-overlay*
*Context gathered: 2026-03-27*
