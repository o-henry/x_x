# Phase 1: Notification Routing - Research

**Researched:** 2026-03-26
**Domain:** Kaku-native mux-owned notification routing, CLI contracts, and minimal unread UI surfacing
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
None — analysis stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| NOTF-01 | Developer can create notification records scoped to an existing workspace, tab, or pane. | Add mux-owned notification store plus dedicated `notify` command and typed request/response PDUs. |
| NOTF-02 | Developer can list notifications through stable machine-readable CLI output. | Mirror `kaku/src/cli/list.rs` JSON contract pattern with explicit serialized record types. |
| NOTF-03 | Developer can clear, mark read, and mark unread notification records through stable CLI commands. | Add typed mutation APIs in mux plus dedicated CLI modules and transport shims. |
| NOTF-04 | Developer can jump to the next unread target across background panes. | Reuse existing focus/navigation flow via `SetFocusedPane` and mux-owned unread ordering helpers. |
| NOTF-05 | Developer can jump to the previous unread target across background panes. | Same routing helpers as NOTF-04, but reverse traversal over existing tab/window order. |
| NOTF-06 | Developer can see unread visual markers on existing Kaku surfaces without changing the overall look and feel. | Extend tabbar/termwindow data snapshots with mux-derived unread aggregates and render small additive markers. |
</phase_requirements>

## Summary

Phase 1 should be implemented as a mux-owned notification subsystem, not as another layer on top of the existing GUI bell flags. The current tree already has the right architectural seams: `mux` owns cross-window runtime state and fan-out notifications, `kaku/src/cli/` already exposes stable machine-readable commands, `crates/codec` and `wezterm-client` already use typed PDUs, and `kaku-gui` already renders tabbar state from snapshot structs. The clean plan is to attach notification records and unread indexes to `mux`, expose them via dedicated CLI subcommands, and have GUI surfaces consume derived unread summaries.

The biggest planning trap is assuming the current unread behavior is close enough. It is not. Today unread is mostly GUI-local bell state in [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs) and a global dock badge count in [`kaku-gui/src/frontend.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/frontend.rs). That is useful as a migration seam, but it cannot be the source of truth for Phase 1 because the user explicitly locked mux ownership and stable CLI access. Planning should therefore treat bell state as compatibility behavior to bridge or replace, not as the notification model itself.

The safest execution order is: add a focused mux store and mutation/query API, add codec/client/server transport, add dedicated CLI subcommands with stable JSON, then wire routing and minimal UI markers. Defer any extra pane-adjacent visuals unless tabbar markers alone fail the requirement.

**Primary recommendation:** Build Phase 1 around a small mux-owned notification store plus derived unread-routing helpers, and treat GUI bells as an input/compatibility seam rather than the authoritative state model.

## Project Constraints (from CLAUDE.md)

- Read and align with `AGENTS.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, and `PLANS.md`.
- Product scope is defined by `docs/KAKU_CONTROL_PLANE_SPEC.md` and `PLANS.md`.
- Rust-only.
- No Swift, Xcode, React, Electron, Tauri, or WebView.
- No copied code from cmux or tmux.
- Preserve Kaku feel and architecture.
- Prefer extending `kaku/src/cli`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay`, `kaku-gui/src/termwindow`, and `mux/src`.
- Do not start with sidebar, browser, PR UI, socket daemon, or full tmux detached semantics.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `mux` | workspace local | Authoritative notification store, unread indexes, routing helpers, and internal events | Existing Kaku runtime state already lives here; Phase 1 locked mux ownership. |
| `codec` | workspace local | Typed request/response and unilateral transport payloads | Existing CLI/server RPC pattern already flows through typed PDUs. |
| `wezterm-client` / `wezterm-mux-server-impl` | workspace local | Client/server transport shims for new notification commands | This is the established bridge between CLI and mux state. |
| `clap` | `4.5.51` | Dedicated `kaku cli` subcommands and argument parsing | Existing command registry is clap-driven and already organized one subcommand per file. |
| `serde` / `serde_json` | `1.0.228` / workspace-resolved | Stable machine-readable output contracts | Existing JSON-facing CLI surfaces already serialize explicit structs. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `anyhow` | `1.0.100` | Boundary error handling for CLI and transport code | Use at command and RPC boundaries; keep record payloads typed. |
| `tabout` | `0.3.0` | Optional table fallback for human-readable list output | Use only for list-style commands; JSON remains the planner-critical contract. |
| `kaku-gui` tabbar / termwindow snapshot structs | workspace local | Minimal unread marker surfacing and Lua-visible metadata | Use after mux state exists; keep visuals additive. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| mux-owned notification truth | GUI-local bell bookkeeping | Faster to hack, but violates D-01/D-02 and breaks stable CLI access. |
| dedicated CLI subcommands | overloading `list` or pane commands | Smaller diff, but weakens machine-readable contracts and feature discoverability. |
| typed PDUs | raw proxy/text parsing | Less code up front, but brittle and inconsistent with the current transport architecture. |
| tabbar-first markers | new overlay/sidebar | More visible, but violates scope and risks changing Kaku’s feel. |

**Version verification:** Versions were verified from [`Cargo.lock`](/Users/henry/Documents/code/vibe/hybrid/x_x/Cargo.lock) and workspace manifests on 2026-03-26: `clap 4.5.51`, `serde 1.0.228`, `anyhow 1.0.100`, `tabout 0.3.0`.

## Architecture Patterns

### Recommended Project Structure
```text
mux/src/
├── lib.rs                  # new notification variants + store entry points
├── notification_store.rs   # add in Phase 1 if lib.rs starts to bloat
└── tab.rs                  # unread traversal helpers in existing tab/window order

kaku/src/cli/
├── mod.rs                  # register Phase 1 subcommands
├── notify.rs
├── list_notifications.rs
├── clear_notifications.rs
├── mark_read.rs
├── mark_unread.rs
├── jump_next_unread.rs
├── jump_prev_unread.rs
├── identify.rs
└── capabilities.rs

crates/
├── codec/src/lib.rs
├── wezterm-client/src/client.rs
└── wezterm-mux-server-impl/src/sessionhandler.rs

kaku-gui/src/
├── tabbar.rs               # tab-level marker composition
├── frontend.rs             # bridge dock badge from mux unread totals
└── termwindow/mod.rs       # focus/bell compatibility wiring and snapshot enrichment
```

### Pattern 1: Mux-Owned Notification Store With Derived Aggregates
**What:** Store notification records and unread truth in `mux`, keyed by existing workspace/tab/pane identities; derive tab/workspace unread counts from pane-targeted truth.
**When to use:** For all notification creation, listing, mutation, and routing logic.
**Recommended shape:**
```rust
struct NotificationRecord {
    notification_id: String,
    workspace: String,
    window_id: Option<WindowId>,
    tab_id: Option<TabId>,
    pane_id: Option<PaneId>,
    unread: bool,
    unread_mode: NotificationUnreadMode, // clear-on-focus | sticky
    created_at: SystemTime,
    updated_at: SystemTime,
    kind: String,
    title: String,
    body: Option<String>,
}
```
**Why this pattern fits:** The repo already centralizes shared runtime state in `Mux`, and the user explicitly locked mux ownership. Keep IDs tied to real mux entities instead of inventing parallel notification targets.

### Pattern 2: Focus-Driven Auto-Read
**What:** Consume existing `MuxNotification::PaneFocused` events to auto-mark matching `clear-on-focus` notifications read.
**When to use:** For `clear-on-focus` semantics only.
**Example:**
```rust
match notification {
    MuxNotification::PaneFocused(pane_id) => {
        notification_store.mark_clear_on_focus_read(pane_id);
    }
    _ => {}
}
```
**Source:** [`mux/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs), [`mux/src/tab.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/tab.rs)

### Pattern 3: Stable Typed CLI Contracts
**What:** Follow the existing `kaku cli list` model: explicit structs, `serde` output, optional table fallback for list surfaces, and dedicated subcommand modules.
**When to use:** For `notify`, `list-notifications`, `clear-notifications`, `mark-read`, `mark-unread`, `identify`, and `capabilities`.
**Example:**
```rust
#[derive(serde::Serialize)]
struct NotificationListItem {
    notification_id: String,
    workspace: String,
    tab_id: Option<TabId>,
    pane_id: Option<PaneId>,
    unread: bool,
    unread_mode: String,
    title: String,
}
```
**Source:** [`kaku/src/cli/list.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/list.rs)

### Pattern 4: Reuse Existing Focus Navigation
**What:** Compute the target unread pane in mux order, then route through existing focus APIs by setting the focused pane.
**When to use:** For `jump-next-unread` and `jump-prev-unread`.
**Recommended approach:** Resolve the target pane in mux, then call the same focus path used by existing CLI and GUI activation flows instead of inventing a new navigation action.
**Source:** [`kaku/src/cli/activate_pane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/activate_pane.rs), [`kaku/src/cli/activate_tab.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/activate_tab.rs)

### Pattern 5: Snapshot-Enriched Minimal UI Surfacing
**What:** Extend `TabInformation` and `PaneInformation` with mux-derived unread booleans/counts so tabbar rendering and Lua formatting callbacks can consume them naturally.
**When to use:** After mux state and routing helpers exist.
**Why this pattern fits:** `kaku-gui` already snapshots tab/pane data for synchronous tabbar formatting; extending those structs is lower risk than adding a separate GUI cache.

### Anti-Patterns to Avoid
- **GUI-owned unread truth:** The current `has_unread_bell` state in [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs) is only a migration seam, not the Phase 1 architecture.
- **New navigation model:** Do not create a custom unread navigator that bypasses normal focus/tab activation behavior.
- **Raw text contracts:** Do not make automation parse human text for mutation/list responses.
- **Sidebar creep:** Tabbar markers first; pane-adjacent hints only if necessary; no Task Center or redesign in this phase.
- **Main-thread ownership resolution in notification callbacks:** Existing comments already warn this can deadlock around `PaneFocused`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Global notification truth | GUI-only unread cache | `mux` store + derived indexes | One source of truth for CLI, routing, and GUI. |
| Navigation action | custom pane-switching logic | existing focused-pane activation path | Preserves Kaku behavior and compatibility requirements. |
| CLI transport | ad hoc proxy or stdout parsing | typed `codec` PDUs and `wezterm-client` wrappers | Matches current architecture and keeps contracts stable. |
| UI state fan-out | polling/reconciliation loop | `MuxNotification` plus existing subscription fan-out | Event-driven updates are already the repo preference. |
| Marker data flow | separate GUI cache | extend `TabInformation` / `PaneInformation` snapshots | Reuses existing synchronous tabbar formatting seam. |

**Key insight:** Phase 1 looks small at the UI layer, but the expensive bugs come from split ownership and contract drift. Reusing mux, typed PDUs, and existing focus paths is the simplest approach once end-to-end behavior matters.

## Common Pitfalls

### Pitfall 1: Double-Counting Bell State
**What goes wrong:** Dock badge or tab markers count both old GUI bell flags and new mux notifications.
**Why it happens:** The current code already increments global unread bell counts in [`kaku-gui/src/frontend.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/frontend.rs) and per-pane flags in [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs).
**How to avoid:** Define one migration rule early: either bell alerts create notification records, or bell flags become a derived compatibility view from mux unread totals.
**Warning signs:** Badge count differs from `list-notifications --format json`, or counts drop when changing focus in only one window.

### Pitfall 2: Focus Callback Deadlocks
**What goes wrong:** Window subscriptions resolve pane ownership on the main thread while handling `PaneFocused`, causing re-entrant locking trouble.
**Why it happens:** The existing subscription code explicitly avoids some ownership resolution on the main thread.
**How to avoid:** Keep unread auto-read in mux/store code and keep GUI notification filters cheap.
**Warning signs:** Focusing panes intermittently freezes or logs ownership-resolution warnings.

### Pitfall 3: Routing Order Drift
**What goes wrong:** CLI `jump-next-unread` and GUI behavior disagree on what “next” means.
**Why it happens:** Order is recomputed in multiple places or uses different traversal APIs.
**How to avoid:** Implement one mux helper that defines unread traversal order and reuse it everywhere.
**Warning signs:** Same unread set yields different next/previous targets depending on caller.

### Pitfall 4: Scope Drift Into Task Center
**What goes wrong:** Planner adds overlay/search work before notification truth and routing are stable.
**Why it happens:** Unread navigation and markers naturally suggest richer UI.
**How to avoid:** Hold the line on tabbar markers first; Phase 3 owns aggregated overlay UX.
**Warning signs:** Plans mention launcher overlays, sidebars, or complex filtering in Phase 1.

### Pitfall 5: Parallel Identity Model
**What goes wrong:** Notifications store custom target IDs instead of existing workspace/tab/pane identities.
**Why it happens:** It feels simpler than threading through real mux IDs.
**How to avoid:** Keep records scoped to existing mux identities from day one; only add notification IDs as record IDs, not routing IDs.
**Warning signs:** Code needs translation tables between notification targets and panes/tabs/windows.

## Code Examples

Verified patterns from repository sources:

### Existing Mux Notification Hook
```rust
if let Some(pane_id) = pane_id {
    let mux = Mux::get();
    mux.notify(MuxNotification::PaneFocused(pane_id));
}
```
Source: [`mux/src/tab.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/tab.rs)

### Existing Stable JSON CLI Pattern
```rust
match self.format {
    CliOutputFormatKind::Json => {
        let mut writer = serde_json::Serializer::pretty(out.lock());
        writer.collect_seq(output_items.iter())?;
    }
    CliOutputFormatKind::Table => { /* tabulate_output */ }
}
```
Source: [`kaku/src/cli/list.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/list.rs)

### Existing GUI Snapshot Pattern
```rust
PaneInformation {
    pane_id: pos.pane.pane_id(),
    pane_index: pos.index,
    is_active: pos.is_active,
    is_zoomed: pos.is_zoomed,
    has_unseen_output: pos.pane.has_unseen_output(),
    progress: pos.pane.get_progress(),
}
```
Source: [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| GUI-local unread bell flags | Mux-owned notification records with derived GUI surfacing | Phase 1 target | Enables stable CLI, routing, and cross-window consistency. |
| Bell-only unread semantics | Explicit `clear-on-focus` and `sticky` notification modes | Phase 1 target | Makes unread behavior predictable and scriptable. |
| Human-centric pane commands | Dedicated notification commands with typed transport | Phase 1 target | Supports tooling and future Task Center work without contract drift. |

**Deprecated/outdated for this phase:**
- GUI-only unread bookkeeping as the source of truth: acceptable today, but incompatible with locked Phase 1 decisions.
- Overloading existing pane list commands for notifications: not aligned with the dedicated-command requirement.

## Open Questions

1. **Should bell alerts automatically create notification records, or should Phase 1 require explicit `notify` calls for durable records?**
   - What we know: Existing bell state is already tracked in GUI code, and the spec says background panes can raise unread attention state.
   - What's unclear: Whether bell is merely one producer of notifications or the default producer.
   - Recommendation: Plan explicit `notify` support first, then decide whether to bridge bell alerts into the same store as a small follow-on inside Phase 1 if needed.

2. **How much pane-adjacent surfacing is actually needed beyond tabbar markers?**
   - What we know: User locked tabbar markers first and left pane-adjacent hints discretionary.
   - What's unclear: Whether tabbar markers alone satisfy NOTF-06 for long-lived background panes.
   - Recommendation: Make pane-adjacent hints optional in the plan, after tabbar markers are validated.

3. **What is the final machine-readable response shape for mutation commands?**
   - What we know: Output must be stable and machine-readable.
   - What's unclear: Whether mutation commands should return full records, counts, or compact acknowledgements.
   - Recommendation: Standardize on JSON objects with `ok`, target identifiers, and affected counts; keep full-record output for list/query surfaces.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build/check/test validation | ✓ | `1.93.0` | — |
| `rustfmt` | Phase quality gate formatting | ✓ | `1.8.0-stable` | — |
| `rustup` | Toolchain management for pinned workspace toolchain | ✓ | `1.28.2` | — |
| `cargo-nextest` | `make test` fast test runner | ✗ | — | use targeted `cargo test` / `cargo check` until installed |
| `rg` | Codebase/search workflow | ✓ | `15.1.0` | `grep` if needed |

**Missing dependencies with no fallback:**
- None identified for planning.

**Missing dependencies with fallback:**
- `cargo-nextest` is missing, so plans should use `cargo test` or `cargo check` as the immediate verification path unless the phase also includes a tool-install step.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust inline/unit tests plus workspace `cargo`/`nextest` commands |
| Config file | [`Makefile`](/Users/henry/Documents/code/vibe/hybrid/x_x/Makefile) and [`.github/workflows/ci.yml`](/Users/henry/Documents/code/vibe/hybrid/x_x/.github/workflows/ci.yml) |
| Quick run command | `cargo check --locked -p kaku -p kaku-gui` |
| Full suite command | `make test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| NOTF-01 | create notification records scoped to workspace/tab/pane | unit | `cargo test --locked -p mux notification_store::tests::create_scoped_notifications -x` | ❌ Wave 0 |
| NOTF-02 | list notifications in stable JSON | unit | `cargo test --locked -p kaku list_notifications::tests::json_shape_is_stable -x` | ❌ Wave 0 |
| NOTF-03 | clear / mark-read / mark-unread mutations | unit | `cargo test --locked -p mux notification_store::tests::mark_and_clear_transitions -x` | ❌ Wave 0 |
| NOTF-04 | jump next unread in existing order | unit | `cargo test --locked -p mux tab::tests::jump_next_unread_respects_order -x` | ❌ Wave 0 |
| NOTF-05 | jump previous unread in existing order | unit | `cargo test --locked -p mux tab::tests::jump_prev_unread_respects_order -x` | ❌ Wave 0 |
| NOTF-06 | tabbar marker reflects unread state without redesign | unit | `cargo test --locked -p kaku-gui tabbar::tests::unread_marker_composition -x` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check --locked -p kaku -p kaku-gui`
- **Per wave merge:** `cargo test --locked -p mux` and `cargo test --locked -p kaku-gui tabbar`
- **Phase gate:** `make check` plus targeted tests for new mux/CLI/UI behavior

### Wave 0 Gaps
- [ ] Add mux unit tests for notification store creation, mutation, and unread traversal.
- [ ] Add CLI serialization tests for notification list/mutation JSON contracts.
- [ ] Add GUI tabbar tests for unread marker composition.
- [ ] Add targeted compatibility check for focus-driven auto-read behavior.
- [ ] Decide whether to install `cargo-nextest` or rely on `cargo test` during this phase.

## Sources

### Primary (HIGH confidence)
- [`docs/KAKU_CONTROL_PLANE_SPEC.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/docs/KAKU_CONTROL_PLANE_SPEC.md) - canonical product definition, Phase 1 commands, UI strategy, constraints
- [`PLANS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/PLANS.md) - extension points, target files, and integration risk notes
- [`AGENTS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/AGENTS.md) - hard constraints, priorities, working rules
- [`CLAUDE.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/CLAUDE.md) - project constraints and preferred extension points
- [`.planning/phases/01-notification-routing/01-CONTEXT.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/01-notification-routing/01-CONTEXT.md) - locked user decisions for this phase
- [`mux/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs) - `Mux`, `MuxNotification`, subscriber/event model
- [`mux/src/tab.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/tab.rs) - focus notifications and pane traversal order
- [`kaku/src/cli/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/mod.rs) - CLI registry pattern
- [`kaku/src/cli/list.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/list.rs) - stable machine-readable output example
- [`crates/codec/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/crates/codec/src/lib.rs) - typed transport PDU schema
- [`crates/wezterm-client/src/client.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/crates/wezterm-client/src/client.rs) - typed client RPC wrapper pattern
- [`crates/wezterm-mux-server-impl/src/sessionhandler.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/crates/wezterm-mux-server-impl/src/sessionhandler.rs) - server dispatch and unilateral transport fan-out
- [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs) - current unread bell state, tab/pane snapshot structs, mux subscription filtering
- [`kaku-gui/src/tabbar.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/tabbar.rs) - minimal tab marker rendering seam
- [`kaku-gui/src/frontend.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/frontend.rs) - current global unread bell badge seam
- [`Cargo.toml`](/Users/henry/Documents/code/vibe/hybrid/x_x/Cargo.toml), [`Cargo.lock`](/Users/henry/Documents/code/vibe/hybrid/x_x/Cargo.lock), [`Makefile`](/Users/henry/Documents/code/vibe/hybrid/x_x/Makefile) - workspace stack and validation commands

### Secondary (MEDIUM confidence)
- [`.planning/codebase/ARCHITECTURE.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/codebase/ARCHITECTURE.md) - architecture summary verified against source reads
- [`.planning/codebase/STRUCTURE.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/codebase/STRUCTURE.md) - file placement guidance verified against source reads
- [`.planning/codebase/CONVENTIONS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/codebase/CONVENTIONS.md) - coding conventions cross-checked against touched files

### Tertiary (LOW confidence)
- None. This research did not rely on unverified web-search-only claims.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - verified directly from workspace manifests and lockfile.
- Architecture: HIGH - grounded in current source files for mux, CLI, transport, and GUI.
- Pitfalls: HIGH - derived from current code comments, existing unread implementation, and locked phase decisions.

**Research date:** 2026-03-26
**Valid until:** 2026-04-25
