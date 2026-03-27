# Phase 2: Workspace Metadata Plane - Context

**Gathered:** 2026-03-27 (assumptions mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Add workspace-scoped status, progress, and log metadata that is writable through stable CLI commands and visible on existing Kaku surfaces. This phase covers mux-owned workspace metadata state, the typed CLI and transport needed to mutate/query it, and minimal surfacing on existing tabbar/status-title surfaces. It does not introduce a new app shell, a Task Center overlay, or task-pane lifecycle semantics.

</domain>

<decisions>
## Implementation Decisions

### State ownership
- **D-01:** Workspace status, progress, and log metadata are mux-owned workspace-scoped state, extending the Phase 1 ownership pattern instead of creating GUI-local or pane-local storage.
- **D-02:** Workspace metadata must remain structured and keyed to existing workspace identities so rename and future Task Center aggregation can reuse it safely.

### CLI and transport contract
- **D-03:** Phase 2 uses dedicated `kaku cli` workspace-metadata commands rather than overloading existing pane/tab commands.
- **D-04:** Phase 2 follows the Phase 1 pattern of typed codec/client/server request paths and stable machine-readable JSON-oriented contracts with targeted request/response shape tests.
- **D-05:** The Phase 2 CLI surface is expected to cover `set-status`, `clear-status`, `list-status`, `set-progress`, `clear-progress`, `log`, `clear-log`, and `list-log`.

### UI surfacing strategy
- **D-06:** Existing Kaku surfaces come first: tabbar and right/left status-title surfaces are the primary Phase 2 UI surfacing targets.
- **D-07:** Overlay exposure is optional and only acceptable if it falls out of minimal reuse of existing surfaces; a new overlay-centric experience is deferred to Phase 3 Task Center work.
- **D-08:** Phase 2 must stay visually additive and Kaku-native, continuing the Phase 1 rule of no app-shell change and no broad visual redesign.

### Progress semantics
- **D-09:** Workspace progress is stored separately from pane runtime progress and is not implemented by mutating existing `PaneInformation.progress` semantics.
- **D-10:** If both pane progress and workspace progress need to be visible, the GUI may compose them at render time while keeping their sources of truth separate.

### the agent's Discretion
- Exact JSON field shapes for status/progress/log records, provided they stay stable and machine-readable.
- Exact tabbar and right-status presentation details, provided they remain minimal and Kaku-native.
- Exact log storage limits or truncation rules, provided the commands stay deterministic and phase scope does not widen into history/search systems.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product definition
- `docs/KAKU_CONTROL_PLANE_SPEC.md` — canonical product definition, phase ordering, Phase 2 command list, UI strategy, and hard constraints
- `PLANS.md` — file-level extension-point audit and recommended placement for `workspace_state` and Phase 2 CLI/UI work

### Planning and requirements
- `.planning/ROADMAP.md` — Phase 2 goal, success criteria, and UI hint
- `.planning/REQUIREMENTS.md` — `META-01` through `META-05` requirements
- `.planning/PROJECT.md` — additive-delivery rules, preferred extension points, non-goals, and local-fork principles
- `.planning/STATE.md` — current project state and brownfield integration concerns
- `.planning/phases/01-notification-routing/01-CONTEXT.md` — prior locked decisions around mux-owned state, stable CLI contracts, and minimal Kaku-native surfacing

### Codebase maps
- `.planning/codebase/ARCHITECTURE.md` — mux/CLI/GUI layering and event-driven UI update model
- `.planning/codebase/STRUCTURE.md` — where new mux-owned state, CLI commands, and GUI surfacing should land
- `.planning/codebase/CONVENTIONS.md` — one-subcommand-per-file CLI pattern and stable JSON-facing contract conventions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `mux/src/lib.rs` — existing mux ownership root, workspace identity resolution, and notification/event fan-out
- `kaku/src/cli/mod.rs` — one-subcommand-per-file CLI registry pattern already used by Phase 1
- `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, `crates/wezterm-mux-server-impl/src/sessionhandler.rs` — typed transport path already extended in Phase 1 and ready for more metadata commands
- `kaku-gui/src/tabbar.rs` — existing tab title composition surface suitable for small metadata markers
- `kaku-gui/src/termwindow/mod.rs` — existing title/status refresh path and mux-driven GUI synchronization
- `kaku-gui/src/scripting/guiwin.rs` — reusable `set_right_status` / `set_left_status` hooks for existing status surfaces

### Established Patterns
- Shared runtime state should live in `Mux` and be distributed through existing mux notifications rather than polling or GUI-owned caches.
- CLI additions should be explicit subcommands with stable JSON-shaped output and colocated contract tests.
- GUI surfacing should remain additive, Kaku-native, and anchored in existing title/tab/status surfaces before introducing new overlay-first flows.

### Integration Points
- New workspace metadata storage should attach to `mux/src/lib.rs` or an adjacent focused mux module such as `mux/src/workspace_state.rs`.
- New Phase 2 commands should register in `kaku/src/cli/mod.rs` with dedicated modules under `kaku/src/cli/`.
- Typed transport for metadata commands should extend `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, and `crates/wezterm-mux-server-impl/src/sessionhandler.rs`.
- Minimal UI surfacing should integrate through `kaku-gui/src/tabbar.rs`, `kaku-gui/src/termwindow/mod.rs`, and existing left/right status-title hooks.

</code_context>

<specifics>
## Specific Ideas

- Keep workspace status as a small explicit value rather than a loose pile of pane-local strings.
- Treat workspace progress as its own metadata channel, separate from pane runtime progress.
- Use existing tabbar or status-title surfaces first; avoid pulling Task Center or large overlay work into Phase 2.

</specifics>

<deferred>
## Deferred Ideas

- Searchable or interactive metadata browsing overlay — Phase 3 Task Center
- Task-pane-aware status synthesis — Phase 4 lifecycle work
- Rich metadata history, log search, or timeline UX beyond simple append/list/clear semantics

</deferred>

---

*Phase: 02-workspace-metadata-plane*
*Context gathered: 2026-03-27*
