# Phase 1: Notification Routing - Context

**Gathered:** 2026-03-26 (assumptions mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Add mux-owned notification records, unread indexes, stable CLI commands, and lightweight unread markers so background panes can raise and retain attention state that is visible and navigable through existing Kaku surfaces. This phase covers notification creation, listing, mutation, unread routing, and minimal visual surfacing only; broader workspace metadata, Task Center aggregation, and task-pane lifecycle behavior belong to later phases.

</domain>

<decisions>
## Implementation Decisions

### State ownership
- **D-01:** Notification records and unread indexes are mux-owned state keyed to existing workspace, tab, and pane identities rather than a parallel GUI-owned cache.
- **D-02:** Unread truth is pane-targeted, with derived aggregation by tab and workspace for routing and lightweight UI surfacing.

### CLI and transport contract
- **D-03:** Phase 1 adds dedicated `kaku cli` notification commands rather than overloading existing pane commands.
- **D-04:** Notification list/query surfaces use stable machine-readable output, and the implementation may add typed codec/client/server request paths as needed to preserve that contract.
- **D-05:** `identify` and `capabilities` are part of the phase-1 machine-oriented contract so local tooling can resolve targets and detect supported notification features safely.

### Unread semantics and routing
- **D-06:** Notifications support two explicit unread modes: `clear-on-focus`, which auto-marks read when the target pane is focused, and `sticky`, which remains unread until explicitly cleared or marked read.
- **D-07:** Next/previous unread navigation follows existing pane/tab/window ordering and must not change current Kaku navigation behavior.
- **D-08:** Unread routing should use existing focus/navigation surfaces and mux focus notifications instead of inventing a separate navigation model.

### UI surfacing
- **D-09:** Visual surfacing stays Kaku-native and minimal, with tabbar markers first and only small pane-adjacent hints if needed.
- **D-10:** Phase 1 must not introduce a sidebar, Task Center, or any broader visual redesign.

### the agent's Discretion
- Exact notification record field shape, provided it remains machine-readable and keyed to existing mux identities
- Exact marker glyph/color treatment, provided it feels additive to current Kaku tabbar and pane surfaces
- Whether pane-adjacent surfacing is needed in phase 1 beyond tabbar-level unread markers

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product definition
- `docs/KAKU_CONTROL_PLANE_SPEC.md` — Canonical product definition, phase ordering, phase-1 command list, UI strategy, and hard constraints
- `PLANS.md` — File-level extension-point audit, target files/modules, and phase-0 risk notes for control-plane work

### Planning and requirements
- `.planning/ROADMAP.md` — Phase 1 goal, success criteria, and requirement mapping
- `.planning/REQUIREMENTS.md` — `NOTF-01` through `NOTF-06` requirements and compatibility constraints
- `.planning/PROJECT.md` — Product principles, non-goals, preferred extension points, and additive-delivery constraints
- `.planning/STATE.md` — Current project focus and brownfield integration concerns

### Codebase maps
- `.planning/codebase/ARCHITECTURE.md` — Shared mux/CLI/GUI layering and event-driven update model
- `.planning/codebase/STRUCTURE.md` — Where control-plane code should land in the current workspace
- `.planning/codebase/CONVENTIONS.md` — Stable CLI output and additive-module conventions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `mux/src/lib.rs`: Existing `Mux` singleton, subscriber fan-out, and `MuxNotification` hub provide the natural ownership point for notification state and mutation events.
- `mux/src/tab.rs`: Existing `PaneFocused` emission and tab traversal logic provide the right seam for next/previous unread routing without changing navigation behavior.
- `kaku/src/cli/mod.rs`: Existing one-subcommand-per-file CLI registry is the right place to add phase-1 commands cleanly.
- `kaku/src/cli/list.rs`: Stable table-or-JSON formatting pattern is the reference for machine-readable notification listing.
- `crates/codec/src/lib.rs`: Existing typed PDU definitions show the normal path for new mux-backed CLI requests/responses.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`: Server-side PDU dispatch already performs mux mutations on the main thread, matching the likely needs of notification commands.
- `kaku-gui/src/tabbar.rs`: Existing tab title/progress composition is the preferred spot for small unread markers that preserve Kaku's current feel.
- `kaku-gui/src/termwindow/mod.rs`: Existing bell handling and pane-local unread state show where current GUI-only unread behavior can be replaced or bridged.
- `kaku-gui/src/frontend.rs`: Existing global unread bell badge logic is the current app-level aggregation seam that will need mux-backed unread totals later in the phase.

### Established Patterns
- Shared mutable runtime state is centralized in `Mux` and distributed through `MuxNotification` rather than polling loops.
- CLI contracts prefer dedicated typed commands with explicit JSON support over ad hoc text parsing.
- GUI affordances are additive through existing tabbar, overlay, and termwindow surfaces instead of introducing new shell structure.
- Focus changes already flow through mux-owned notifications, which fits auto-read semantics for `clear-on-focus` notifications.

### Integration Points
- New notification store and unread indexes should attach to `mux/src/lib.rs` or adjacent focused mux modules.
- New phase-1 commands should register in `kaku/src/cli/mod.rs` with dedicated modules under `kaku/src/cli/`.
- Any server/client transport needed for the new commands should extend `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, and `crates/wezterm-mux-server-impl/src/sessionhandler.rs`.
- Minimal unread markers should surface through `kaku-gui/src/tabbar.rs` and, if necessary, small termwindow updates in `kaku-gui/src/termwindow/mod.rs`.

</code_context>

<specifics>
## Specific Ideas

- Support two explicit notification modes from the start:
  - `clear-on-focus` notifications auto-read when their target pane is focused
  - `sticky` notifications stay unread until explicitly cleared or marked read
- Keep unread state pane-targeted, with tab/workspace indexes derived from pane-level truth
- Preserve existing pane/tab/window navigation ordering exactly for next/previous unread traversal

</specifics>

<deferred>
## Deferred Ideas

None — analysis stayed within phase scope.

</deferred>

---

*Phase: 01-notification-routing*
*Context gathered: 2026-03-26*
