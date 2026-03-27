# Phase 4: Task-Pane Lifecycle - Context

**Gathered:** 2026-03-27 (assumptions mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Add task-pane lifecycle semantics so failed or long-running panes remain useful after exit and can be rerun, silenced, or tee'd without rebuilding context manually. This phase covers remain-on-exit behavior, rerun/respawn metadata persistence, watchdog silence controls, pipe-pane style output teeing, and pane lifecycle metadata durable enough for downstream CLI and Task Center surfaces. It does not introduce a new shell model, replace Kaku's existing pane lifecycle, or widen into detached tmux-style session semantics.

</domain>

<decisions>
## Implementation Decisions

### Lifecycle ownership
- **D-01:** Task-pane lifecycle state is mux-owned pane-scoped state, not GUI-local state and not Task Center-owned cache.
- **D-02:** Phase 4 extends the existing pane lifecycle seams (`is_dead`, `exit_behavior`, user vars, pane snapshots) rather than inventing a second dead-pane model.

### Remain-on-exit semantics
- **D-03:** Remain-on-exit is implemented as an extension of existing `ExitBehavior` and `DeadPendingClose` behavior, preserving current Kaku close/hold semantics.
- **D-04:** Remain-on-exit must stay additive and opt-in for selected task panes; unrelated panes keep existing Kaku behavior.

### Rerun and respawn model
- **D-05:** Runtime rerun metadata seeded from pane user vars remains the source input, but Phase 4 adds mux-owned persistence so failed-pane rerun affordances survive long enough for later consumers.
- **D-06:** Lifecycle controls should distinguish rerun/respawn semantics explicitly enough for stable CLI contracts, but Task Center remains a consumer of those controls rather than the owner of their state machine.

### Control surface and contracts
- **D-07:** Phase 4 is CLI-first for lifecycle mutation/query actions; GUI and Task Center consume the resulting mux-backed lifecycle state instead of defining separate action semantics.
- **D-08:** New lifecycle commands must stay machine-readable and consistent with the existing `kaku/src/cli` one-command-per-file pattern.

### Watchdog silence and tee behavior
- **D-09:** Watchdog silence is per-task-pane lifecycle state, not a global process or workspace setting.
- **D-10:** Pipe-pane / tee output must preserve normal pane rendering and interactivity, acting as additive output duplication rather than output redirection.

### the agent's Discretion
- Exact CLI naming for remain-on-exit, rerun/respawn, silence, and tee commands, provided the contracts stay explicit and machine-readable.
- Exact lifecycle metadata field names and retention details, provided they remain pane-scoped and durable enough for downstream surfaces.
- Exact tee sink/file-handling policy, provided normal pane behavior remains intact and the implementation stays within existing mux output plumbing.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product definition
- `docs/KAKU_CONTROL_PLANE_SPEC.md` — Canonical Phase 4 lifecycle scope, required capabilities, and hard constraints.
- `PLANS.md` — File-level extension-point audit, lifecycle risk notes, and the requirement to extend existing `ExitBehavior`/pane seams rather than forking the model.

### Planning and requirements
- `.planning/ROADMAP.md` — Phase 4 goal, dependency ordering, and success criteria for remain-on-exit, rerun, silence, and tee behavior.
- `.planning/REQUIREMENTS.md` — `LIFE-01` through `LIFE-05` requirement source of truth.
- `.planning/PROJECT.md` — Rust-only, Kaku-native, additive-delivery constraints.
- `.planning/STATE.md` — Brownfield integration concerns and high-risk surfaces.

### Prior phase context
- `.planning/phases/01-notification-routing/01-CONTEXT.md` — Mux-owned state and stable CLI contract rules that Phase 4 must continue.
- `.planning/phases/02-workspace-metadata-plane/02-CONTEXT.md` — Existing-surface and mux-owned metadata decisions that lifecycle state should align with.
- `.planning/phases/03-task-center-overlay/03-CONTEXT.md` — Task Center is a consumer overlay, not the owner of lifecycle state; rerun UX boundary with Phase 4 is already locked.
- `.planning/phases/03-task-center-overlay/03-LIMITATIONS.md` — Current Phase 3 limitations around limited-mode failed/running heuristics and rerun metadata persistence that Phase 4 is expected to close.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `mux/src/localpane.rs` — Existing `ExitBehavior::{Close, CloseOnCleanExit, Hold}` handling and `DeadPendingClose` transitions are the natural foundation for remain-on-exit semantics.
- `mux/src/pane.rs` — Pane trait already exposes `is_dead`, `exit_behavior`, and `copy_user_vars`, which are the right seams for lifecycle persistence inputs.
- `mux/src/lib.rs` — Existing mux-owned snapshot assembly and output parsing path can host lifecycle state and tee hooks without inventing a second controller layer.
- `mux/src/task_center.rs` — Current failed/running/rerun snapshot model already consumes pane dead-state, progress, and rerun user vars; Phase 4 should upgrade that source rather than replace the consumer.
- `kaku-gui/src/termwindow/mod.rs` — Current Task Center rerun action is consumer-side and reads rerun user vars directly; this is the main GUI seam that should switch to durable lifecycle-backed behavior later in the phase.
- `kaku/src/cli/mod.rs` — Existing CLI registry pattern is the right home for explicit lifecycle commands.

### Established Patterns
- Mux remains the authoritative owner for control-plane state, with GUI overlays and status surfaces reading snapshots rather than owning lifecycle truth.
- Existing Kaku pane exit behavior already supports hold/close distinctions, so lifecycle work should extend that path instead of bypassing it.
- Task Center currently exposes failed/running/rerun in limited mode; Phase 4 should make those signals honest and durable without turning Task Center into a lifecycle controller.

### Integration Points
- Pane lifecycle state likely belongs in a focused mux module adjacent to other control-plane stores, with `mux/src/localpane.rs` and `mux/src/lib.rs` as the primary integration seams.
- New lifecycle CLI commands should live in `kaku/src/cli/` and likely require typed transport in `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, and `crates/wezterm-mux-server-impl/src/sessionhandler.rs`.
- Task Center and other GUI consumers should read upgraded failed/running/rerun state through existing snapshot refresh paths in `kaku-gui/src/termwindow/mod.rs`.
- Pipe-pane / tee should hook into existing mux output handling so pane rendering remains unchanged while output is duplicated to file.

</code_context>

<specifics>
## Specific Ideas

- Keep remain-on-exit as a Kaku-native extension of current hold behavior instead of a new detached task shell.
- Treat rerun metadata persistence as the bridge between Phase 3 Task Center affordances and Phase 4 lifecycle reliability.
- Prefer pane-scoped silence and tee controls that can be reasoned about locally, instead of workspace-global switches.

</specifics>

<deferred>
## Deferred Ideas

- Full detached session or tmux-style server semantics remain out of scope.
- Any broad task dashboard, browser-like log explorer, or non-Kaku app shell remains out of scope.
- Deeper policy systems for task orchestration beyond pane lifecycle belong to later phases or backlog items.

</deferred>

---

*Phase: 04-task-pane-lifecycle*
*Context gathered: 2026-03-27*
