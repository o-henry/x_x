# Phase 2: Workspace Metadata Plane - Research

**Researched:** 2026-03-27
**Domain:** mux-owned workspace metadata, stable CLI contracts, and minimal Kaku-native UI surfacing
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
- Searchable or interactive metadata browsing overlay — Phase 3 Task Center
- Task-pane-aware status synthesis — Phase 4 lifecycle work
- Rich metadata history, log search, or timeline UX beyond simple append/list/clear semantics
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| META-01 | Developer can set a workspace-scoped status value through the CLI. | Reuse the Phase 1 typed `codec` -> `wezterm-client` -> `sessionhandler` -> `Mux` path with a dedicated status request/response and stable JSON output. |
| META-02 | Developer can clear and list workspace-scoped status values through the CLI. | Model status as one structured record per workspace, keyed by workspace name, with `list-status` returning stable records and `clear-status` returning a mutation summary. |
| META-03 | Developer can set and clear workspace-scoped progress values through the CLI. | Keep workspace progress in a separate mux-owned store from pane progress, and expose it to GUI composition at render time rather than mutating `PaneInformation.progress`. |
| META-04 | Developer can append workspace-scoped log entries and list or clear them through the CLI. | Use an append-only bounded per-workspace log collection in mux state, with deterministic insertion order and explicit truncation behavior. |
| META-05 | Developer can see workspace status and progress on existing Kaku UI surfaces without adding a new app shell. | Extend existing tabbar/status-title refresh paths and `MuxNotification`-driven invalidation instead of adding a new shell or overlay-first workflow. |
</phase_requirements>

## Summary

Phase 2 should be planned as a direct extension of the Phase 1 pattern that already exists in this repo: mux-owned in-memory state, typed codec PDUs, thin CLI commands under `kaku/src/cli/`, server-side mutation/query handling in `crates/wezterm-mux-server-impl/src/sessionhandler.rs`, and lightweight GUI refresh via existing `MuxNotification` fan-out. The current codebase already contains the important precedent: `mux/src/notification_store.rs`, `MuxNotification::NotificationsChanged`, JSON contract tests in the CLI modules, codec round-trip tests, and tabbar refresh hooks in `kaku-gui/src/termwindow/mod.rs`.

The planning-critical decision is to keep workspace metadata as a focused mux module of its own rather than smearing it across `Mux`, pane user vars, or GUI-local caches. The cleanest Phase 2 shape is a new `mux/src/workspace_state.rs` module owned by `Mux`, with three explicit record types: a single status record per workspace, a single progress record per workspace, and a bounded append-only log list per workspace. That keeps rename-safe workspace keys, avoids contaminating pane runtime progress, and gives Phase 3 Task Center a stable aggregate source later.

UI scope should stay intentionally small. Existing surfaces already support this: tab titles are composed in `kaku-gui/src/tabbar.rs`, window status text is already refreshed through `update-right-status` / `update-status` hooks in `kaku-gui/src/termwindow/mod.rs`, and tabbar repaint is already event-driven. The phase plan should therefore focus on adding metadata-aware refresh triggers and compact render composition, not on introducing any new overlay workflow.

**Primary recommendation:** Add a mux-owned `workspace_state` store plus a dedicated `WorkspaceMetadataChanged` mux event, then mirror the Phase 1 CLI/RPC contract style for `set-status`, `clear-status`, `list-status`, `set-progress`, `clear-progress`, `log`, `clear-log`, and `list-log`.

## Project Constraints (from CLAUDE.md)

- Read and follow `AGENTS.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, and `PLANS.md`.
- Product scope is defined by `docs/KAKU_CONTROL_PLANE_SPEC.md` and `PLANS.md`.
- Hard constraints: Rust-only; no Swift/Xcode; no React/Electron/Tauri/WebView; no copied code from cmux or tmux; preserve Kaku feel and architecture.
- Preferred extension points: `kaku/src/cli`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay`, `kaku-gui/src/termwindow`, `mux/src`.
- Do not start with a sidebar, browser, PR UI, socket daemon, or full tmux detached semantics.

## Standard Stack

### Core
| Library / Module | Version | Purpose | Why Standard |
|------------------|---------|---------|--------------|
| Rust toolchain | 1.93.0 | Project toolchain pin | The repo pins `rust-toolchain.toml` to 1.93.0, so planning should target that compiler and stdlib behavior. |
| `mux/src/workspace_state.rs` (new) | repo-local | Authoritative workspace metadata store | Matches the Phase 1 `notification_store` pattern and keeps metadata mux-owned, rename-safe, and reusable by later phases. |
| `crates/codec` + `serde` | serde 1.0.228 | Stable request/response types | This is already how Phase 1 machine-readable contracts are encoded and tested. |
| `kaku/src/cli` + `clap` | clap 4.5.51 locked, 4.5.60 latest docs.rs | Stable subcommand parsing | The repo already uses one-subcommand-per-file clap-derived commands; Phase 2 should extend the same pattern. |
| `parking_lot` | 0.12.5 | Shared mux locking | `Mux` already uses `parking_lot::{RwLock, Mutex}` heavily; Phase 2 should fit that model rather than introducing a second synchronization style. |
| `chrono` | 0.4.42 locked, 0.4.44 latest docs.rs | Metadata timestamps | Already used by Phase 1 records and suitable for status/progress/log timestamps if serialized carefully. |

### Supporting
| Library / Module | Version | Purpose | When to Use |
|------------------|---------|---------|-------------|
| `tabout` | repo-local crate | Human-readable CLI table output | Use for `list-status` / `list-log` table mode to match existing CLI UX. |
| `serde_json` | 1.0.145 | Stable JSON output | Use for all machine-oriented output paths and contract tests. |
| `kaku-gui/src/tabbar.rs` | repo-local | Minimal metadata visualization | Use for compact workspace markers and composed status/progress rendering. |
| `kaku-gui/src/termwindow/mod.rs` | repo-local | Event-driven repaint and status hooks | Use for refresh scheduling and passing workspace metadata into existing status surfaces. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| mux-owned `workspace_state` | pane `user_vars` or GUI-local caches | Worse: pane user vars are runtime-scoped and GUI caches would break the locked Phase 2 ownership model. |
| dedicated metadata event | reusing `NotificationsChanged` | Worse: semantically noisy, makes refresh filtering less precise, and couples unrelated stores. |
| separate workspace progress store | mutating `PaneInformation.progress` | Worse: violates locked decision D-09 and confuses workspace metadata with pane runtime progress. |
| existing tabbar/status surfaces | new overlay surface | Worse for Phase 2: widens scope and collides with the deferred Task Center work. |

**Installation:**
```bash
# No new package manager is required for planning.
# Existing repo workflow:
cargo build --locked -p kaku -p kaku-gui -p wezterm-mux-server-impl
```

**Version verification:** Verified from local repo state and current docs.rs pages on 2026-03-27.
- Rust toolchain pin: `1.93.0` from `rust-toolchain.toml`
- Locked workspace crates from `Cargo.lock`: `clap 4.5.51`, `chrono 0.4.42`, `parking_lot 0.12.5`, `serde 1.0.228`, `serde_json 1.0.145`, `anyhow 1.0.100`
- Current docs.rs pages: `clap 4.5.60` (2026-02-19), `chrono 0.4.44` (2026-02-23), `parking_lot 0.12.5` (2025-10-03)

## Architecture Patterns

### Recommended Project Structure
```text
mux/
├── src/lib.rs                 # Mux ownership and notification dispatch
└── src/workspace_state.rs     # new focused workspace metadata store

kaku/
└── src/cli/
    ├── set_status.rs
    ├── clear_status.rs
    ├── list_status.rs
    ├── set_progress.rs
    ├── clear_progress.rs
    ├── log.rs
    ├── clear_log.rs
    └── list_log.rs

crates/
├── codec/src/lib.rs           # typed request/response structs
├── wezterm-client/src/client.rs
└── wezterm-mux-server-impl/src/sessionhandler.rs

kaku-gui/
├── src/tabbar.rs              # minimal workspace marker composition
└── src/termwindow/mod.rs      # repaint/status refresh and metadata snapshot flow
```

### Pattern 1: Focused Mux-Owned Store Per Concern
**What:** Keep workspace metadata in its own mux module, similar to `notification_store`, instead of bolting more unrelated fields directly onto `Mux`.
**When to use:** Immediately for Phase 2; this is the cleanest way to keep state structured, rename-safe, and future-aggregatable.
**Example:**
```rust
// Source: existing repo pattern in mux/src/notification_store.rs
pub struct WorkspaceStateStore {
    status_by_workspace: HashMap<String, WorkspaceStatusRecord>,
    progress_by_workspace: HashMap<String, WorkspaceProgressRecord>,
    logs_by_workspace: HashMap<String, Vec<WorkspaceLogRecord>>,
    next_log_seq: u64,
}
```

### Pattern 2: Typed CLI -> Codec -> Client -> Sessionhandler -> Mux
**What:** Every new CLI command gets its own clap module, typed PDU structs, client wrapper, and server-side handler.
**When to use:** For every Phase 2 command. Do not shortcut by reading mux state directly from the CLI.
**Example:**
```rust
// Source: existing repo pattern in crates/wezterm-client/src/client.rs
rpc!(set_workspace_status, SetWorkspaceStatus, SetWorkspaceStatusResponse);
rpc!(list_workspace_status, ListWorkspaceStatus, ListWorkspaceStatusResponse);
```

### Pattern 3: Event-Driven GUI Refresh
**What:** GUI state repaints when mux notifications say a relevant domain changed.
**When to use:** For tabbar/status metadata updates. Avoid polling loops or GUI-owned recomputation.
**Example:**
```rust
// Source: existing repo pattern in kaku-gui/src/termwindow/mod.rs
fn mux_notification_requires_tabbar_refresh(notification: &MuxNotification) -> bool {
    matches!(
        notification,
        MuxNotification::NotificationsChanged
            | MuxNotification::WorkspaceMetadataChanged
    )
}
```

### Pattern 4: Compose Workspace and Pane Progress at Render Time
**What:** Keep workspace progress and pane progress as separate sources of truth; combine only in the title/status rendering layer.
**When to use:** When a tab needs to show both runtime pane activity and long-lived workspace task progress.
**Example:**
```rust
// Source: recommendation based on D-09/D-10 and existing tabbar progress rendering
let pane_progress = active_pane.progress;
let workspace_progress = mux.workspace_progress_for_tab(tab.tab_id());
let title = compose_progress_badges(pane_progress, workspace_progress);
```

### Recommended Data Shapes

Use these stable JSON-oriented records:

```rust
pub struct WorkspaceStatusState {
    pub workspace: String,
    pub status: Option<String>,
    pub updated_at: String,
}

pub struct WorkspaceProgressState {
    pub workspace: String,
    pub value: Option<u8>,
    pub updated_at: String,
}

pub struct WorkspaceLogState {
    pub workspace: String,
    pub seq: u64,
    pub message: String,
    pub created_at: String,
}
```

Prescriptive contract guidance:
- `status` should be a single optional string per workspace, not a list.
- `progress` should be an integer `0..=100`; reject out-of-range input in the CLI/parser layer.
- `list-status` should return records, not bare strings, so timestamps stay available.
- `list-log` should return logs in append order and support filtering by workspace.
- `clear-*` commands should return mutation summaries, not silent success.

### Anti-Patterns to Avoid
- **GUI-owned metadata caches:** Breaks D-01 and creates sync problems with CLI mutations.
- **Reusing pane user vars as the workspace metadata database:** User vars are pane runtime state, not a stable workspace metadata contract.
- **Stuffing workspace progress into `PaneInformation.progress`:** Violates D-09 and makes later lifecycle work harder.
- **One giant generic `workspace-metadata` command with modes:** Goes against the repo’s one-subcommand-per-file CLI pattern.
- **Formatting timestamps with `DateTime::to_string()` for the stable contract:** It is display-oriented. Prefer explicit RFC 3339 serialization for a stable machine contract.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parser | custom argv parser | `clap` derive subcommands | Already standard in this repo and gives stable, typed command surfaces. |
| JSON serialization | hand-built JSON strings | `serde` + `serde_json` | Reduces contract drift and matches Phase 1 patterns. |
| Shared runtime ownership | ad hoc globals or GUI caches | `Mux` + focused store module | Keeps a single source of truth and existing fan-out semantics. |
| Table rendering | manual padding logic | `tabout` | Already used in Kaku CLI list commands. |
| UI refresh | polling or timer-based redraws | `MuxNotification`-driven invalidation | Matches current GUI architecture and avoids unnecessary work. |
| Timestamp formatting | display formatting as contract | explicit RFC 3339 (`to_rfc3339` / `to_rfc3339_opts`) | Stable machine-readable timestamps are easier to test and parse. |

**Key insight:** The repo already solved the hard architectural part in Phase 1. Phase 2 planning should deliberately clone the shape of that solution, not invent a second control-plane pattern.

## Common Pitfalls

### Pitfall 1: Letting workspace metadata leak into pane runtime state
**What goes wrong:** Status/progress ends up attached to panes or user vars, so switching panes or replacing panes changes workspace-level truth.
**Why it happens:** Pane APIs and `PaneInformation.progress` are already visible and tempting to reuse.
**How to avoid:** Keep authoritative workspace metadata in `Mux` keyed by workspace name and derive display snapshots from there.
**Warning signs:** The plan mentions `copy_user_vars`, `set_user_var`, or editing `PaneInformation.progress` as the source of truth.

### Pitfall 2: Reusing `NotificationsChanged` instead of adding a metadata-specific event
**What goes wrong:** GUI refresh logic becomes ambiguous, event filtering grows noisy, and future Task Center work can’t tell which store changed.
**Why it happens:** `NotificationsChanged` already exists and refreshes the tabbar.
**How to avoid:** Add a distinct `MuxNotification::WorkspaceMetadataChanged` or similarly precise event.
**Warning signs:** Phase 2 UI updates are described as “just trigger `NotificationsChanged` too.”

### Pitfall 3: Designing unstable timestamps
**What goes wrong:** JSON contracts become display-dependent and harder to parse or compare in tests.
**Why it happens:** The existing Phase 1 server helper currently uses `record.created_at.to_string()` / `updated_at.to_string()`.
**How to avoid:** Standardize on RFC 3339 output in the new Phase 2 contract and ideally align Phase 1 later.
**Warning signs:** The plan says “serialize timestamps with `.to_string()`.”

### Pitfall 4: Allowing unbounded workspace logs
**What goes wrong:** A long-running local Kaku session accumulates large in-memory log vectors, and `list-log` becomes noisy and slow.
**Why it happens:** Append-only logs look simple until they live for days.
**How to avoid:** Pick an explicit per-workspace cap now, such as last `200` entries, and document truncation semantics.
**Warning signs:** No cap, no truncation rule, and no sequence number in the log record model.

### Pitfall 5: Trying to solve Task Center in Phase 2
**What goes wrong:** The phase expands into overlay browsing, filtering, and interaction work that belongs to Phase 3.
**Why it happens:** Workspace logs naturally invite a browsing UI.
**How to avoid:** Keep Phase 2 UI to tabbar and right/left status surfaces only; overlay reuse is optional and should not be required.
**Warning signs:** The plan introduces search, filters, or interactive metadata history.

## Code Examples

Verified patterns from repo and official sources:

### Workspace Store API Shape
```rust
// Source: recommended from repo pattern in mux/src/notification_store.rs
impl WorkspaceStateStore {
    pub fn set_status(&mut self, workspace: &str, status: String) -> WorkspaceStatusRecord;
    pub fn clear_status(&mut self, workspace: &str) -> bool;
    pub fn list_statuses(&self, workspace: Option<&str>) -> Vec<WorkspaceStatusRecord>;

    pub fn set_progress(&mut self, workspace: &str, value: u8) -> WorkspaceProgressRecord;
    pub fn clear_progress(&mut self, workspace: &str) -> bool;

    pub fn append_log(&mut self, workspace: &str, message: String) -> WorkspaceLogRecord;
    pub fn list_logs(&self, workspace: Option<&str>) -> Vec<WorkspaceLogRecord>;
    pub fn clear_logs(&mut self, workspace: &str) -> usize;
}
```

### CLI Request/Response Pattern
```rust
// Source: existing repo pattern in kaku/src/cli/notify.rs and list_notifications.rs
#[derive(Debug, Parser, Clone)]
pub struct SetStatusCommand {
    #[arg(long = "workspace")]
    workspace: String,

    #[arg(long = "status")]
    status: String,
}

impl SetStatusCommand {
    fn to_request(&self) -> SetWorkspaceStatus {
        SetWorkspaceStatus {
            workspace: self.workspace.clone(),
            status: self.status.clone(),
        }
    }
}
```

### Explicit RFC 3339 Timestamp Formatting
```rust
// Source: chrono docs.rs DateTime::to_rfc3339 / to_rfc3339_opts
use chrono::{SecondsFormat, Utc};

let now = Utc::now();
let created_at = now.to_rfc3339_opts(SecondsFormat::Secs, true);
```

### Tabbar Refresh Trigger
```rust
// Source: existing repo pattern in kaku-gui/src/termwindow/mod.rs
fn mux_notification_requires_tabbar_refresh(notification: &MuxNotification) -> bool {
    matches!(
        notification,
        MuxNotification::PaneFocused(_)
            | MuxNotification::TabResized(_)
            | MuxNotification::TabTitleChanged { .. }
            | MuxNotification::WindowInvalidated(_)
            | MuxNotification::NotificationsChanged
            | MuxNotification::WorkspaceMetadataChanged
    )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| GUI-local unread or status hints | mux-owned shared state with typed fan-out | Already established by Phase 1 in this fork | Phase 2 should extend the same ownership model. |
| Human-oriented CLI only | stable JSON-oriented CLI contracts with dedicated codec PDUs | Already established by Phase 1 in this fork | Phase 2 should keep machine-readable mutation/list outputs from day one. |
| Display-oriented timestamp strings | explicit RFC 3339 contract fields | Current best practice in Chrono API | Makes future tooling and tests deterministic. |
| Polling for UI changes | event-driven mux notifications | Existing Kaku architecture | Phase 2 UI should refresh on targeted notifications only. |

**Deprecated/outdated:**
- Using pane-local state as a substitute for workspace-level metadata: outdated for this fork because it conflicts with locked mux ownership.
- Adding a sidebar or new shell surface first: outdated for this phase because the spec explicitly orders tabbar/status surfaces ahead of overlay and Task Center.

## Open Questions

1. **Should `list-status` default to all workspaces or only the active workspace?**
   What we know: The phase requires listability and stable CLI output, but the exact filtering defaults are discretionary.
   What's unclear: Whether operator ergonomics favor “current workspace by default” or “all workspaces by default.”
   Recommendation: Plan for `--workspace` filtering, but default `list-status` and `list-log` to all workspaces so future automation can inventory the whole mux state.

2. **What per-workspace log cap is appropriate?**
   What we know: The context explicitly allows discretion on storage limits, as long as commands stay deterministic.
   What's unclear: The desired retention window for solo Unity workflows.
   Recommendation: Start with a simple in-memory cap of `200` entries per workspace and document FIFO truncation.

3. **Which existing surface should carry the primary progress signal: tab title or right status?**
   What we know: Both are allowed, and overlay-first UX is deferred.
   What's unclear: Which one is visually quieter in real Kaku use.
   Recommendation: Plan for compact tabbar marker plus richer right-status text, with tabbar as the always-on minimal signal.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | build, check, tests | ✓ | 1.93.0 | — |
| `rustc` | compile | ✓ | 1.93.0 | — |
| `rg` | code search during execution and debugging | ✓ | 15.1.0 | use `grep` if needed |
| `cargo-nextest` | repo `make test` path | ✗ | — | use targeted `cargo test` / `cargo check` for this phase |

**Missing dependencies with no fallback:**
- None for planning. Phase 2 can still be implemented and checked without `cargo-nextest`.

**Missing dependencies with fallback:**
- `cargo-nextest` is not installed locally, but targeted `cargo test` and `cargo check --locked` remain viable phase validation fallbacks.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust inline unit tests + Cargo test/nextest |
| Config file | `Makefile` defines standard commands; no separate test runner config found for this phase |
| Quick run command | `cargo test -p mux workspace_state -- --nocapture` |
| Full suite command | `make test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| META-01 | set workspace status | unit | `cargo test -p mux workspace_status -- --nocapture` | ❌ Wave 0 |
| META-02 | clear and list status | unit | `cargo test -p kaku set_status -- --nocapture` | ❌ Wave 0 |
| META-03 | set and clear progress | unit | `cargo test -p mux workspace_progress -- --nocapture` | ❌ Wave 0 |
| META-04 | append, list, clear logs | unit | `cargo test -p mux workspace_logs -- --nocapture` | ❌ Wave 0 |
| META-05 | tabbar/status refresh on workspace metadata | unit | `cargo test -p kaku-gui workspace_metadata -- --nocapture` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check --locked -p mux -p kaku -p kaku-gui -p wezterm-mux-server-impl`
- **Per wave merge:** targeted `cargo test` for touched crates
- **Phase gate:** `make test` if `cargo-nextest` is installed; otherwise run targeted `cargo test` suites plus the locked `cargo check` command above

### Wave 0 Gaps
- [ ] `mux/src/workspace_state.rs` tests for status/progress/log mutation, ordering, truncation, and workspace rename behavior
- [ ] `kaku/src/cli/{set_status,clear_status,list_status,set_progress,clear_progress,log,clear_log,list_log}.rs` request-shape and JSON contract tests
- [ ] `crates/codec/src/lib.rs` PDU round-trip tests for all new metadata request/response structs
- [ ] `kaku-gui/src/tabbar.rs` and/or `kaku-gui/src/termwindow/mod.rs` tests for metadata-driven title/status refresh triggers
- [ ] Optional local tool install: `cargo install cargo-nextest --locked`

## Sources

### Primary (HIGH confidence)
- Repo sources inspected directly:
  - `mux/src/lib.rs` - existing mux ownership, `notification_store`, and `MuxNotification` fan-out
  - `mux/src/notification_store.rs` - Phase 1 focused store pattern and tests
  - `kaku/src/cli/mod.rs` - one-subcommand-per-file clap command registry
  - `kaku/src/cli/notify.rs`, `kaku/src/cli/list_notifications.rs`, `kaku/src/cli/capabilities.rs` - stable CLI request/JSON contract patterns
  - `crates/codec/src/lib.rs` - typed request/response structs and codec round-trip tests
  - `crates/wezterm-client/src/client.rs` - RPC macro pattern
  - `crates/wezterm-mux-server-impl/src/sessionhandler.rs` - server-side PDU handling on the main thread
  - `kaku-gui/src/tabbar.rs` - existing title/progress composition and unread marker pattern
  - `kaku-gui/src/termwindow/mod.rs` - mux subscription filtering, tabbar refresh triggers, right/left status hooks
- Project docs:
  - `docs/KAKU_CONTROL_PLANE_SPEC.md`
  - `PLANS.md`
  - `.planning/phases/02-workspace-metadata-plane/02-CONTEXT.md`
  - `.planning/REQUIREMENTS.md`
  - `.planning/codebase/ARCHITECTURE.md`
  - `.planning/codebase/STRUCTURE.md`
  - `.planning/codebase/CONVENTIONS.md`
- Docs.rs current crate pages:
  - https://docs.rs/crate/clap/latest - current clap release/version metadata
  - https://docs.rs/crate/chrono/latest - current chrono release/version metadata
  - https://docs.rs/crate/parking_lot/latest - current parking_lot release/version metadata
- Chrono API docs:
  - https://docs.rs/chrono/latest/chrono/struct.DateTime.html - `to_rfc3339` / `to_rfc3339_opts`

### Secondary (MEDIUM confidence)
- Rust std docs:
  - https://doc.rust-lang.org/std/sync/struct.RwLock.html - used as contrast for standard lock behavior while the repo standardizes on `parking_lot`
- Clap derive example:
  - https://docs.rs/clap/latest/src/03_04_subcommands_derive/03_04_subcommands.rs.html - confirms current clap derive subcommand pattern

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Most recommendations are verified directly from the repo plus current docs.rs crate pages.
- Architecture: HIGH - The exact extension seams are already visible in current Phase 1 code and project planning docs.
- Pitfalls: HIGH - They derive from concrete existing code patterns, locked decisions, and one current contract weakness around timestamp formatting.

**Research date:** 2026-03-27
**Valid until:** 2026-04-26
