# Kaku Agent Control Plane Plan

## Big Picture

Turn this local Kaku fork into a Rust-only multi-agent terminal by combining:

- cmux-like notification and workspace metadata ergonomics
- tmux-like task-pane lifecycle and control semantics

while preserving vanilla Kaku's feel.

## Progress

- [x] Phase 0: audit extension points and finalize file-level design
  Shipped outcome:
  extension points were mapped, file-level ownership was documented, and the
  repo-local spec/agent/skill scaffolding needed for the implementation phases
  was established before Phase 1 work began
- [x] Phase 1: notification store + unread semantics + CLI + visual markers
  Phase 1 CLI contract:
  `notify`, `list-notifications`, `clear-notifications`, `mark-read`, `mark-unread`,
  `jump-next-unread`, `jump-prev-unread`, `identify`, `capabilities`
  Notification JSON fields:
  `notification_id`, `workspace`, `window_id`, `tab_id`, `pane_id`, `kind`, `title`,
  `body`, `unread`, `unread_mode`, `created_at`, `updated_at`
  Mutation JSON fields:
  `cleared_count`, `updated_count`, `notification_ids`
  Capabilities JSON fields:
  `notification_commands`, `unread_modes`, `supports_tabbar_markers`
- [x] Phase 2: workspace status/progress/log metadata
  Phase 2 CLI contract:
  `set-status`, `clear-status`, `list-status`, `set-progress`, `clear-progress`,
  `log`, `clear-log`, `list-log`
  Status JSON fields:
  `workspace`, `status`, `updated_at`
  Progress JSON fields:
  `workspace`, `value`, `updated_at`
  Log JSON fields:
  `workspace`, `seq`, `message`, `created_at`
  Clear summary JSON fields:
  `cleared_count`, `workspaces`
- [x] Phase 3: Task Center overlay
  Shipped shape:
  native `ShowTaskCenter` command, dedicated overlay, token filters, focus on `Enter`,
  clear-unread on `c`, rerun when durable metadata exists
- [x] Phase 4: task-pane lifecycle (remain-on-exit, rerun/respawn, silence watchdog, pipe-pane)
  Phase 4 CLI contract:
  `list-task-panes`, `set-remain-on-exit`, `rerun-pane`, `respawn-pane`,
  `silence-watchdog`, `pipe-pane`
  Task-pane list JSON fields:
  `pane_id`, `workspace`, `window_id`, `tab_id`, `remain_on_exit`, `silenced`,
  `is_dead`, `is_failed`, `rerun_available`, `tee_path`, `current_working_dir`, `updated_at`
  Mutation JSON fields:
  remain-on-exit -> `pane_id`, `remain_on_exit`, `is_dead`, `is_failed`, `updated_at`
  rerun/respawn -> `pane_id`, `spawned_pane_id`, `status`
  silence-watchdog -> `pane_id`, `silenced`
  pipe-pane -> `pane_id`, `tee_path`
- [x] Phase 5: hardening, tests, docs, regression pass
  Shipped outcome:
  green targeted hardening suite, explicit changed-files/limitations/UAT artifacts,
  failing-pane survivability verified against a live GUI session, baseline pane
  management commands preserved, and `respawn-pane` rechecked successfully against
  a freshly rebuilt live runtime
  Residual scope note:
  live desktop verification now includes Task Center additivity, but broader
  multi-window and multi-workspace coverage still leans on the targeted
  compatibility suite plus preserved baseline Kaku behavior
- [x] Phase 6: operator UI surfaces for mouse/keyboard-first control-plane use
  Shipped outcome:
  Task Center as the primary operator surface, compact tabbar discovery
  affordances, visible row actions, mouse+keyboard parity, and
  prompt/confirm metadata editing that stay additive to Kaku instead of
  introducing a dashboard shell
  Residual scope note:
  the targeted `kaku-gui` closeout suite is green, the rebuilt runtime now has
  a fresh responsive socket plus a rechecked native Task Center keyboard path,
  but the visual treatment is still more text-heavy and utilitarian than the
  desired reference-video operator polish
- [ ] Phase 7: operator visual polish toward the provided video reference
  Intended outcome:
  keep the native Rust/Kaku architecture, but replace the current utilitarian
  treatment with a more intentional DM Mono + icon-first operator look, remove
  texty affordances like `· ops` where appropriate, and push Task Center/tabbar
  composition closer to the provided reference without becoming a dashboard shell
  Reference-driven observations:
  the provided video shows a slim edge-attached left rail rather than a large
  floating card, compact dark chrome, a visually dominant main work area, and
  persistent pane hierarchy where navigation remains visible instead of opening
  as a separate dashboard overlay
  Fixed layout slots:
  `left rail`, `top chrome/status strip`, `primary terminal/editor area`,
  `secondary utility/context area`, and only compact inline state markers
  Phase 7 implementation order:
  1. make the left rail structurally match the reference first
  2. tighten top chrome and tab/title composition to match the reference density
  3. reduce text noise in markers and action hints
  4. only then refine colors, iconography, and spacing
  Exact files for Phase 7:
  [kaku-gui/src/termwindow/mod.rs](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs),
  [kaku-gui/src/termwindow/render/paint.rs](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/render/paint.rs),
  [kaku-gui/src/termwindow/render/fancy_tab_bar.rs](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/render/fancy_tab_bar.rs),
  [kaku-gui/src/tabbar.rs](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/tabbar.rs)
  Validation rule:
  do not call Phase 7 visually complete until a fresh runtime screenshot is
  compared against the reference frame structure and the rail/chrome hierarchy
  is no longer card-like or overlay-like
  Concrete reference elements to follow:
  - `left rail`
    narrow, edge-attached, full-height, low-contrast background, compact rows,
    no large rounded outer card, active item indicated subtly instead of a big
    filled block
  - `top chrome`
    minimal title/status strip with restrained density, no noisy helper labels,
    and a clearer separation between chrome and the primary work surface
  - `main work surface`
    terminal/editor area must remain visually dominant and start immediately to
    the right of the rail, not look boxed in by dashboard panels
  - `secondary context surface`
    if present, it should read as a real persistent pane or split, not a popup
    or floating dashboard card
  - `iconography`
    state should be conveyed with compact symbols and spacing, not punctuation
    or long textual helper hints
  New implementation approach:
  - stop iterating on `Task Center` as the primary visual target
  - treat the reference as a `main window layout` problem first, and an overlay
    problem second
  - redesign the persistent `termwindow` chrome and content slots before doing
    more polish on overlay rows
  - keep the existing Kaku/mux/task control behavior, but move its visibility
    into a quieter always-on structure
  Required technical work:
  - add a dedicated native left-rail renderer with row hit targets and active
    state that does not rely on a generic boxed `Element` card layout
  - tighten or partially replace the current fancy tab bar composition so the
    top strip reads like application chrome instead of a decorated terminal tab
  - introduce a native right-side contextual pane or reserved split region only
    if the reference structure requires it; do not fake this with overlay-only UI
  - add screenshot-based runtime checkpoints after each structural step
  What is likely needed beyond current code:
  - no web frontend stack is required
  - no Swift/Xcode layer is required
  - the current Rust rendering path is sufficient for another pass, but if we
    still cannot reach the target structure after a focused main-window rewrite,
    the next honest step would be introducing a proper native widget container
    layer inside Rust rather than continuing to force everything through the
    existing terminal-style box model
  Stop conditions:
  - if the rail still reads as a card after the dedicated rail renderer pass,
    stop and switch implementation strategy
  - if the top chrome still reads like stock Kaku tab UI after the chrome pass,
    stop and redesign that strip directly instead of polishing colors
  Phase 7 plans:
  1. `07-01-PLAN.md` — rebuild the persistent left rail first so it becomes a slim edge-attached native surface instead of another operator card
  2. `07-02-PLAN.md` — tighten top chrome and tab/title composition into quieter DM Mono-first app chrome with icon-first markers
  3. `07-03-PLAN.md` — rebalance the main work-surface hierarchy and stop for a structural screenshot gate before any overlay polish
  4. `07-04-PLAN.md` — refine Task Center only as a secondary surface after the persistent shell structure is approved
  5. `07-05-PLAN.md` — close Phase 07 with regression evidence, screenshot-based UAT, and honest docs on whether the current renderer was sufficient
  Current service follow-up:
  port the native-shell-tested workspace/inbox rail ideas back into `kaku-gui`
  without replacing Kaku's renderer; use the existing `termwindow` operator nav,
  task-center snapshot data, and workspace metadata cache so the current service
  gets a persistent workspace row, clearer inbox affordance, and workspace-scoped
  operator actions instead of overlay-only utility chrome
  Current service control-strip follow-up:
  add a full-width bottom shortcut strip to `kaku-gui` using the existing
  command registry and effective keymap, and back it with workspace-scoped
  task-center filter actions so the strip reflects real Kaku commands instead
  of static UI-only labels
- [ ] Phase 8: native Rust shell replatform on top of the existing Kaku core
  Intended outcome:
  stop forcing the reference UI through the existing `kaku-gui` terminal renderer,
  keep the current Kaku/mux/control-plane runtime as the backend source of truth,
  and build a new Rust-native application shell that owns the persistent rail,
  top chrome, main work surface, and context panes
  Current session focus:
  finish the incomplete terminal area so the native shell can be used like the
  original Kaku service instead of showing a small preview-style PTY block;
  prioritize a dominant, interactive main terminal surface before any broader
  shell polish
  Current polish follow-up:
  remove duplicate workspace headers inside the main shell pane, rename the
  primary pane chrome from `TERMINAL` to `SHELL`, equalize the left
  `WORKSPACES` and `INBOX` rail sections, and stop shell bootstrap commands
  from leaking visible `PROMPT=` lines into the terminal surface
  Current implementation step:
  replace the stopgap ANSI text buffer in the native shell with the repo's
  real `wezterm_term::Terminal` screen state so the main terminal surface
  renders from Kaku's actual terminal engine rather than an ad-hoc PTY preview
  Terminal renderer follow-up:
  stop using GTK `TextView` line/tag painting for fullscreen TUIs and move the
  dominant terminal surface onto a custom drawing path so `nvim`-style full
  background regions render as a continuous canvas instead of visible row bands
  Immediate layout follow-up:
  make `cmd+t` grow into real multi-shell splits in the dominant workspace
  area, move shortcut affordances into a bottom taskbar-style strip, and make
  `Activity` independently toggleable by keyboard without wasting terminal area
  Current UI follow-up:
  stretch the bottom shortcut strip across the full shell width under the left
  rail, remove visible chip boxes, and make the strip drag-scroll horizontally
  without exposing a scrollbar
  Current hardening follow-up:
  defer `cmd+t` session growth off the GTK action trampoline and shorten
  terminal-session borrow scopes so action-triggered rerenders stop aborting;
  upgrade the custom terminal surface from Cairo toy text calls to
  Pango-shaped text drawing so Korean and fallback glyphs render naturally
  Current pane-management follow-up:
  promote the per-terminal path strip into real pane chrome, style it like the
  rest of the shell headers, and allow drag/drop reordering of terminal panes
  directly inside the workspace surface
  Current performance follow-up:
  stop making `cmd+t` block on synchronous PTY/shell startup and stop leaving
  orphaned terminal refresh loops alive after rerenders; shell panes should
  appear immediately, finish booting asynchronously, and tear down their
  per-widget draw timers once the widget leaves the scene graph
  Current stabilization follow-up:
  remove the extra wrapper chrome around live shell panes, make the left
  `WORKSPACE` row reuse the same titlebar rhythm as `INBOX`, simplify
  terminal font selection to a real Pango font description that reliably
  resolves to `DM Mono`, and remove the custom scrollbar CSS that is still
  provoking GTK slider warnings during startup
  Current implementation + verification track:
  1. move native-shell runtime bootstrap off the GTK main thread so cold start
     no longer produces `응답없음` or hiservices startup aborts during direct runs
  2. make the shell renderer use one consistent DM Mono-first fallback chain so
     cursor placement, glyph width, and body text metrics stay aligned
  3. always show a friendly cwd/title per pane using home-relative `~/...`
     paths so multi-shell startup state is visible immediately
  4. implement native-shell pane lifecycle ergonomics before the next wide verification pass:
     focused pane tracking, focus-next/focus-prev, close-focused-pane, and split affordances
     that map to the bottom shortcut strip instead of static labels
  5. verify with user-like runtime attempts after each patch:
     launch, idle, `CMD+T`, pane reorder, typing, `ls`, `nvim`, close, relaunch
  5. keep a verifier loop running in parallel so code work and GUI regression
     checks do not serialize on the main agent
  Current verification harness follow-up:
  add a dev-only smoke runner inside `kaku-native-shell` so the app can launch,
  wait for bootstrap, create panes, inject terminal input, reset, and exit while
  recording whether the shell survived each step; use this to supplement fragile
  macOS focus-driven automation whenever live window activation is inconsistent
  Current verified baseline on 2026-03-29:
  smoke mode now launches, reaches `startup_online=true`, creates up to four
  panes, injects terminal text successfully, and emits a completion report; live
  manual runs now survive at least a 10-second idle window and still respond to
  `CMD+T` split creation after the recent terminal update and rerender batching
  passes
  Latest verified follow-up on 2026-03-29:
  smoke mode now also confirms `text_rendered_ok=true` and `reset_ok=true`, and
  direct live verification re-confirmed a friendly shell prompt plus successful
  `CMD+T` growth to three panes after the shell bootstrap changes
  Current direct-execution closeout loop:
  1. launch the app like a user would from a real terminal session, not just
     via `cargo check`, and treat startup hangs as blocking failures
  2. verify cold start, idle, `CMD+T`, typing, `ls`, pane focus, pane reorder,
     `nvim`, relaunch, and repeated split attempts in one loop before claiming
     any shell work complete
  3. keep the left `WORKSPACE` row and `INBOX` header on the same titlebar
     rhythm, remove any extra shell titlebar above the per-pane cwd strip, and
     let the cwd strip consume the full shell-pane top edge
  4. prefer event-driven terminal updates over permanent fast polling whenever
  Current shell-visibility follow-up:
  remove stray zsh `%` startup artifacts by disabling `PROMPT_EOL_MARK`, and
  make the left workspace rail plus inbox reflect local native-shell panes
  directly instead of relying only on mux task-pane/notification records
     idle performance or `응답없음` reports appear during verification
  Exact files for Phase 8 initial scaffold:
  [Cargo.toml](/Users/henry/Documents/code/vibe/hybrid/x_x/Cargo.toml),
  [PLANS.md](/Users/henry/Documents/code/vibe/hybrid/x_x/PLANS.md),
  [kaku-native-shell/Cargo.toml](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-native-shell/Cargo.toml),
  [kaku-native-shell/src/main.rs](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-native-shell/src/main.rs)
  Runtime migration rule:
  `kaku-gui` currently owns mux bootstrap plus `gui-sock-*` publication; Phase 8
  must extract or replace that runtime responsibility before declaring the new
  shell a full GUI replacement
  Phase 8 implementation order:
  1. add a compilable `GTK4 + libadwaita` native shell crate with a persistent
     rail, top chrome, center work surface, and right/bottom context panes
  2. bridge the new shell to Kaku control-plane snapshots using the existing
     `codec`, `wezterm-client`, and `mux` state models
  3. move mux bootstrap and socket publication into shared runtime code so the
     new shell can boot Kaku without depending on `kaku-gui`
  4. port operator actions and parity flows until the legacy GUI becomes optional
  MVP exclusions:
  no embedded browser, no new terminal renderer, and no attempt to embed
  `kaku-gui` inside the new shell; initial work is `native shell + Kaku backend bridge`

### Phase 8 Native-Shell Port Checklist

#### Essential Ports

- ~~Rust-native shell crate scaffold exists and boots without depending on `kaku-gui` UI code~~
- ~~shared Kaku runtime bootstrap is wired into the native shell~~
- ~~main shell work surface is dominant and no longer a tiny PTY preview block~~
- ~~`wezterm_term` state is used instead of the earlier ad-hoc ANSI buffer~~
- ~~custom terminal drawing path replaced GTK `TextView` row-band rendering~~
- ~~friendly per-pane cwd/title strip is shown with home-relative `~/...` formatting~~
- ~~multi-shell layout exists for 1/2/3/4 panes~~
- ~~pane drag/drop reorder exists inside the workspace surface~~
- ~~pane focus-next / focus-prev / close-focused-pane actions exist~~
- ~~bottom shortcut strip reflects real native-shell pane actions instead of static placeholders~~
- ~~native-shell can launch Lazygit, Yazi, `kaku config`, and `kaku doctor` from the focused pane~~
- ~~pane strip surfaces parseable shell cwd plus last-command failure state, including `NOT FOUND` for missing commands~~
- ~~two-pane split direction toggle and focused-pane zoom/unzoom exist in the native shell~~
- ~~workspace rail now uses two-line navigation rows with per-workspace attention indicators, so unread/failure state is visible without opening inbox~~
- ~~bundled `DMMono Nerd Font` assets are now the default mono family everywhere `DM Mono` was previously used~~
- shell-suite parity still needs to move beyond the current minimal prompt bootstrap and restore the productivity defaults that matter for daily use
- pane resize ergonomics still need a native-shell equivalent where they materially improve real workflows
- cold-start stability must still be hardened until `응답없음` and startup aborts are gone
- idle and split performance must still be hardened until pane creation no longer feels sluggish
- split startup cost is now reduced by reusing shared shell bootstrap config per shell family; keep pushing until pane creation feels immediate in live use
- live `CMD+T` / pane actions must be rechecked after each performance pass on a real app run
- shortcut-strip natural width inflation was fixed enough to restore live startup width to `1180x820`, but live tool-launch shortcut delivery still needs hardening under real macOS input
- terminal output backlog loss that degraded TUI rendering was fixed, but `lazygit`/`yazi` still need a final real-keyboard validation pass after shortcut delivery is tightened
- native-shell rerender path still rebuilds the shell tree too often; move to a persistent host and direct focus updates before deeper renderer optimization
- terminal pump idle cadence is now relaxed and hidden panes back off further, but full event-driven updates are still the target

#### Replacement Implementations

- ~~persistent left rail and inbox/context panes exist in the native shell~~
- ~~workspace status / progress / log / unread metadata is surfaced in native-shell side panels~~
- workspace rows should keep getting richer agent/worktree identity, including stronger agent naming and branch fidelity when multi-agent sessions become primary
- ~~smoke harness verifies startup, split growth, text injection, rendered output, close-pane, focus-cycle, and reset~~
- native-shell control plane still needs more direct Kaku parity for operator flows beyond the current rail/context surfaces
- terminal body font and glyph fallback still need final visual polish for Korean + Nerd Font icon parity
- cursor placement and live typing alignment still need final verification against real shell usage
- `nvim`/TUI rendering still needs end-to-end validation after the current performance fixes
- `nvim`/LazyVim alignment still needs a true cell-grid renderer path; segment-shaped drawing is still causing visible centering drift
- add an internal startup autorun hook for native-shell verification so fullscreen/live `nvim` checks do not depend on flaky macOS keystroke injection
- 2026-03-29 finalization pass is focused on four items only: lock `WORKSPACE`/`INBOX` rail rhythm so it stops regressing, enrich workspace rows with branch/status/progress identity, restore more shell-suite defaults inside the minimal bootstrap, and keep shaving visible input lag without reintroducing the width regression

#### Intentional Exclusions / Native Replacements

- full Kaku UI parity is not the goal when a clearer native-shell surface is better for solo multi-agent work
- original Kaku shortcut shapes may be replaced when a native-shell shortcut is clearer or more ergonomic
- remote-files and pure terminal-only chrome are not automatic must-port items unless they prove necessary in the native-shell workflow
- settings, assistant, and operator surfaces can be represented as persistent native panes instead of terminal-only overlays when that improves usability

## Non-Goals

- embedded browser
- PR/GitHub UI
- full detached tmux server
- port scanner
- Swift/Xcode sidecar app

## Acceptance

Done means:
- unread routing works
- status/progress/log metadata works
- failed panes persist
- failed panes rerun
- pipe-pane / tee works
- remain-on-exit and task-pane lifecycle state survive pane exit long enough for CLI and Task Center consumers
- old Kaku behavior still works
- known verification gaps or runtime mismatches are documented immediately in the phase limitations/UAT artifacts instead of being hidden

## Per-Phase Rule

Before each phase:
1. inspect existing Kaku extension points
2. list exact files to change
3. keep changes additive
4. avoid broad refactors

After each phase:
1. compile / check
2. run targeted verification
3. document changed files and contracts
4. update this plan

## Phase 0 Audit

### Exact Existing Extension Points

- `mux/src/lib.rs`
  Current ownership root for `Mux`, `MuxNotification`, subscriber fan-out, pane/window/tab resolution, workspace rename, and main-thread notification delivery. This is the best place for new control-plane stores and typed mux-level events.
- `mux/src/localpane.rs`
  Existing source of `Alert` emission, pane `user_vars`, `Progress`, and exit-behavior transitions. This is the current bridge between pane runtime state and mux notifications, and the likely hook for task-pane lifecycle signals.
- `mux/src/pane.rs`
  Shared pane trait already exposes `copy_user_vars`, `get_progress`, `get_current_working_dir`, and `exit_behavior`. This is the stable abstraction boundary for control-plane reads without teaching the GUI about concrete pane types.
- `mux/src/tab.rs`
  Owns tab focus, tab title updates, pane iteration, zoom-aware pane enumeration, and `PaneFocused` notifications. This is the right place to derive unread routing targets at tab granularity.
- `mux/src/window.rs`
  Owns workspace membership, active tab bookkeeping, and invalidation notifications. This is the right place to keep workspace-scoped aggregation separate from per-pane state.
- `kaku/src/cli/mod.rs`
  One-subcommand-per-file pattern, with clap wiring centralized here. New machine-readable control-plane commands should follow this layout rather than multiplexing into existing commands.
- `kaku/src/cli/list.rs`
  Existing example of a stable JSON contract with a table fallback. This is the contract style to mirror for notification and workspace metadata list commands.
- `kaku/src/cli/proxy.rs`
  Existing raw stream pipe for mux RPC. This is adjacent to, but not itself sufficient for, the planned JSONL event stream.
- `crates/codec/src/lib.rs`
  Canonical RPC PDU schema. Any new CLI capability that needs server cooperation or streaming must be declared here first.
- `crates/wezterm-client/src/client.rs`
  Client RPC shim and unilateral PDU handler. This is where new control-plane requests and streamed event PDUs will be exposed to the CLI and GUI.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
  Server dispatch for RPC and unilateral per-pane updates. This is the narrowest existing transport seam for control-plane queries and a JSONL-friendly event subscription path.
- `kaku-gui/src/termwindow/mod.rs`
  Current GUI state hub. Already owns per-pane unread bell state, overlay assignment/cancellation, tab and pane info snapshots, mux subscription filtering, and title/status update hooks. This is the main GUI integration point for unread routing, workspace metadata display, task-pane lifecycle surfacing, and Task Center activation.
- `kaku-gui/src/tabbar.rs`
  Existing synchronous tab title formatting and per-tab progress/title rendering. This is the right place for small additive unread/status markers without redesigning the shell.
- `kaku-gui/src/commands.rs`
  Existing GUI command registry. A Task Center entry should land here so it participates in the command palette and menu model like other native actions.
- `kaku-gui/src/overlay/mod.rs`
  Generic overlay bootstrap for tab- and pane-scoped overlays. This is the right creation/cancellation surface for a Task Center overlay.
- `kaku-gui/src/overlay/launcher.rs`
  Closest existing model for a searchable multi-source overlay with actions. Task Center should reuse this interaction shape rather than inventing a new shell.
- `config/src/keyassignment.rs`
  Home of `KeyAssignment`, launcher flags, and GUI action argument types. Any first-class Task Center action should be added here so config, commands, and UI stay aligned.
- `kaku-gui/src/frontend.rs`
  Already owns global unread bell badge accounting and workspace reconciliation. If unread routing expands beyond bells, this is the app-level place to keep badge/global notification behavior consistent.

### Exact Files To Add Or Change

#### Core mux data and events

- Change `mux/src/lib.rs`
  Add a mux-owned control-plane store entry point, new typed notifications for control-plane mutations, and read/query helpers used by CLI and GUI.
- Change `mux/src/localpane.rs`
  Emit lifecycle-relevant control-plane events on bell/progress/user-var/exit transitions without changing pane rendering behavior.
- Change `mux/src/tab.rs`
  Add helper methods for resolving next/previous unread pane targets within tab order and window order.
- Change `mux/src/window.rs`
  Add minimal helpers for workspace-scoped unread and metadata aggregation traversal.
- Add `mux/src/control_plane.rs`
  Shared in-memory control-plane state root that owns notification store, unread indexes, workspace metadata, and task-pane registry.
- Add `mux/src/notification_store.rs`
  Notification record model, unread indexing, and mutation APIs.
- Add `mux/src/workspace_state.rs`
  Workspace-scoped status/progress/log data model and query APIs.
- Add `mux/src/task_panes.rs`
  Task-pane lifecycle model: remain-on-exit intent, rerun metadata, failed/running state, silence flags, and output tee metadata.
- Add `mux/src/event.rs`
  Stable serializable event payload types for JSONL streaming and internal mux-to-CLI/gui fan-out.
- Change `mux/src/lib.rs` module exports
  Re-export the new control-plane modules without widening unrelated APIs.

#### CLI and command contracts

- Change `kaku/src/cli/mod.rs`
  Register all new subcommands and keep Phase 1/2/3/4 command boundaries explicit.
- Add `kaku/src/cli/notify.rs`
- Add `kaku/src/cli/list_notifications.rs`
- Add `kaku/src/cli/clear_notifications.rs`
- Add `kaku/src/cli/mark_read.rs`
- Add `kaku/src/cli/mark_unread.rs`
- Add `kaku/src/cli/jump_next_unread.rs`
- Add `kaku/src/cli/jump_prev_unread.rs`
- Add `kaku/src/cli/identify.rs`
- Add `kaku/src/cli/capabilities.rs`
- Add `kaku/src/cli/set_status.rs`
- Add `kaku/src/cli/clear_status.rs`
- Add `kaku/src/cli/list_status.rs`
- Add `kaku/src/cli/set_progress.rs`
- Add `kaku/src/cli/clear_progress.rs`
- Add `kaku/src/cli/log.rs`
- Add `kaku/src/cli/clear_log.rs`
- Add `kaku/src/cli/list_log.rs`
- Add `kaku/src/cli/watch_events.rs`
  Planned JSONL event stream CLI entry point. Keep this separate from `proxy` so the external contract is stable and control-plane-specific.

#### RPC / transport

- Change `crates/codec/src/lib.rs`
  Add request/response PDUs for control-plane mutations and queries, plus a unilateral or subscribed event payload for JSONL-friendly streaming.
- Change `crates/wezterm-client/src/client.rs`
  Add typed RPC wrappers for the new commands and client-side handling for streamed control-plane events.
- Change `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
  Dispatch the new PDUs into mux-owned control-plane state and bridge mux events into transport events.

#### GUI unread routing and Task Center

- Change `kaku-gui/src/termwindow/mod.rs`
  Replace bell-only unread bookkeeping with mux-backed unread routing reads, expose workspace metadata snapshots to format/status hooks, and wire Task Center activation/actions.
- Change `kaku-gui/src/tabbar.rs`
  Add tab-level unread/status/progress marker composition using the new mux-backed snapshot data.
- Change `kaku-gui/src/commands.rs`
  Add a native Task Center command and palette/menubar metadata.
- Change `config/src/keyassignment.rs`
  Add a first-class `ShowTaskCenter` assignment or equivalent action args so the feature is configurable without overloading launcher flags.
- Change `kaku-gui/src/overlay/mod.rs`
  Export Task Center overlay bootstrap alongside existing overlay surfaces.
- Add `kaku-gui/src/overlay/task_center.rs`
  Searchable overlay for unread notifications, failed panes, running panes, and workspace metadata. Shipped Phase 3 behavior uses one normalized row list, query-token filters, focus on `Enter`, clear-unread on selected unread rows, and rerun only when runtime rerun metadata is still available.
- Change `kaku-gui/src/overlay/launcher.rs`
  Reuse filtering/rendering helpers where practical, but keep Task Center semantics in its own file.
- Change `kaku-gui/src/frontend.rs`
  Reconcile global unread badge behavior with mux-backed unread counts instead of the current bell-only counter.

### Rough Event And Data Ownership

- `mux`
  Authoritative owner of notification records, unread indexes, workspace status/progress/log state, task-pane lifecycle state, and the canonical control-plane event log.
- `mux` event model
  Should emit typed control-plane events whenever store state changes. GUI and CLI stream consumers should subscribe to mux-owned events rather than recomputing from pane state.
- `local pane` and other pane implementations
  Producers of raw runtime signals only: bell, progress, user vars, cwd, exit status, and spawn metadata. They should not own notification history or unread indexes.
- `window/tab` traversal
  Ownership is derived, not stored: use existing window/tab/pane resolution for routing decisions, but keep unread truth in the mux store.
- `CLI`
  Stateless command surface. It should translate argv into codec requests, print stable JSON or JSONL, and avoid caching notification or metadata state locally.
- `GUI termwindow`
  Per-window presentation/controller only. It should cache the current snapshot needed for paint/overlay interactions, but not become the source of truth for unread or metadata state.
- `Task Center overlay`
  Pure view/controller over mux-owned snapshots. It should not own notifications or lifecycle state; it should request actions back through mux/commands.
- `frontend`
  App-level badge/OS-notification adapter only. Global unread counts should be derived from mux-backed unread totals, not from ad hoc pane flags.

### Phase-Specific Risk Notes

- Existing unread behavior is bell-only and partly GUI-owned.
  `kaku-gui/src/termwindow/mod.rs` and `kaku-gui/src/frontend.rs` currently track unread bell state locally. Migrating to mux-backed unread routing must avoid double-counting or desynchronizing badge state.
- `MuxNotification` fan-out is hot-path code.
  Adding overly chatty control-plane events could create noisy per-window work. New event payloads should be typed and filterable so `subscribe_to_pane_updates()` can stay cheap.
- Main-thread deadlock risk is real.
  `TermWindow` already avoids certain `resolve_pane_id` calls on the main thread. Any new unread-routing or Task Center actions must preserve that discipline.
- Overlay lifecycle already has leak-sensitive edges.
  `assign_overlay`, `cancel_overlay_for_tab`, and `cancel_overlay_for_pane` currently clean up overlay panes carefully. Task Center should reuse this pattern instead of inventing a detached overlay manager.
- Task-pane lifecycle overlaps existing `ExitBehavior`.
  `mux/src/localpane.rs` already has `Hold`, `Close`, and `CloseOnCleanExit` semantics. Remain-on-exit and rerun metadata should extend that behavior, not fork a second lifecycle model.
- JSONL stream contract needs version discipline.
  A `watch-events` stream is tempting to expose raw internal notifications, but Phase 0 should assume a separate stable event schema so future mux refactors do not break local tooling.
- Workspace metadata can sprawl quickly.
  Keep status/progress/log as workspace-scoped structured stores with explicit limits and clear mutation semantics; do not let ad hoc user vars become the persistence format.
- Task Center scope creep is a risk.
  Reuse launcher-style filtering and existing overlay surfaces. Do not turn Phase 3 planning into a sidebar, browser, or daemon design.

## Native Shell Functional Pass

- Screenshot facts to preserve:
  - Persistent left workspace rail stays visible at all times.
  - Thin dark top chrome remains compact and secondary to the work surface.
  - Center pane is the primary work surface; right and lower panes stay persistent rather than overlay-driven.
  - UI must reflect real Kaku runtime state instead of placeholder cards or fake telemetry.
- This pass focuses on runtime-backed interaction, not another layout rewrite.
- Exact files to change:
  - `kaku-native-shell/src/main.rs`
- Done for this pass means:
  - selecting a workspace in the rail updates center/right/lower panes
  - shell buttons visibly mutate real mux-backed state
  - status/progress/log/unread changes are observable in the running UI
  - code compiles and the native shell still boots shared runtime directly

### Current Stabilization Follow-up

- Keep scope tight around `kaku-native-shell` startup and pane reliability.
- Exact files to change:
  - `kaku-native-shell/src/lib.rs`
  - `kaku-native-shell/src/app_controller.rs`
  - `kaku-native-shell/src/terminal.rs`
- Work items:
  - Replace the `adw::Application` activate path with a plain Adwaita window bootstrap so macOS startup stops aborting in the activate trampoline.
  - Keep native-shell shortcuts on the window/widget path instead of relying on app-level accelerators.
  - Re-verify `CMD+T` and shell pane lifecycle only after startup is stable.
