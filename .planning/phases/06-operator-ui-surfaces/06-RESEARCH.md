# Phase 06: Operator UI Surfaces - Research

**Researched:** 2026-03-27
**Domain:** Native Kaku operator UX on existing Rust GUI surfaces
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- The next phase must prioritize native UI over CLI recall for everyday control-plane work.
- Common control-plane tasks should be operable from Kaku with mouse and keyboard, without requiring the developer to remember subcommand names.
- The UI must remain additive to Kaku rather than turning the app into a separate dashboard shell.
- The desired tone is closer to the attached reference video at `/Users/henry/Downloads/566510994-6f3047c2-e2b6-49f2-b536-570a1570d0f8.mp4`: dark, focused, tool-like, and workspace-centric.
- The visual direction should feel more intentional and operable than the current CLI-first control plane, while still respecting Kaku's terminal-native character.

### The Agent's Discretion
- Which specific native surfaces should carry the common-path actions first: Task Center, tabbar affordances, context menus, inspector overlays, or command surfaces.
- How strongly the phase should emphasize mouse click targets versus keyboard discoverability, as long as both are supported.
- How much of the workspace metadata editing flow belongs inline versus in overlays.
- Which pieces of the reference video's visual language are useful to borrow without creating a non-Kaku shell.

### Deferred Ideas
- Full dashboard shell replacement
- Browser-style side panels or embedded web content
- Any UI that requires React, Tauri, Electron, WebView, Swift, or Xcode
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-01 | Developer can discover core control-plane actions from native Kaku UI without remembering CLI subcommands. | Reuse `ShowTaskCenter`, `Shell -> Task Center`, command palette metadata, and visible row actions rather than adding a new shell. |
| UI-02 | Developer can inspect notification, unread, failed-task, and workspace status context through mouse-friendly and keyboard-friendly UI surfaces. | Extend the existing mux snapshot, full-tab overlay list, and tabbar metadata rail; add mouse parity to the existing overlay loop. |
| UI-03 | Developer can trigger common lifecycle and attention actions from the UI, including focus, clear unread, rerun/respawn, and remain-on-exit style toggles where applicable. | Route UI actions through `TermWindow` helpers and existing mux/task-pane state rather than inventing a second action path. |
| UI-04 | Developer can update workspace-facing metadata through UI affordances for the common path, with CLI preserved as an advanced fallback rather than the primary path. | Use prompt/confirm overlays plus existing codec/client RPCs for status/progress updates; keep CLI commands as fallback. |
| UI-05 | New control-plane UI remains additive and Kaku-native, avoiding a dashboard-shell rewrite while still feeling more operable than a CLI-only workflow. | Keep work inside `kaku-gui/src/overlay`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, and `kaku-gui/src/termwindow/mod.rs`; do not add side panels or web surfaces. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Read and follow `AGENTS.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, and `PLANS.md`.
- Product scope is defined by `docs/KAKU_CONTROL_PLANE_SPEC.md` and `PLANS.md`.
- Hard constraints: Rust-only, no Swift/Xcode, no React/Electron/Tauri/WebView, no copied cmux/tmux code, preserve Kaku feel and architecture.
- Preferred extension points: `kaku/src/cli`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay`, `kaku-gui/src/termwindow`, `mux/src`.
- Do not start with: sidebar, browser, PR UI, socket daemon, or full tmux detached semantics.

## Summary

Phase 6 is not a greenfield UI phase. The core operator surfaces already exist in partial form: `mux` owns a control-plane snapshot, `kaku-gui/src/overlay/task_center.rs` renders a normalized full-tab overlay, `kaku-gui/src/commands.rs` already exposes `Task Center` in the Shell menu and command system, and `kaku-gui/src/tabbar.rs` already surfaces unread and workspace metadata inline. Planning should assume the phase is about upgrading discoverability, mouse parity, and metadata editing on top of those existing seams rather than introducing a new app shell or a broad refactor.

The biggest planning distinction is between view state and source-of-truth state. The current code already gets this mostly right: `mux` builds `TaskCenterEntry` rows, `TermWindow` caches and dispatches actions, and overlays render/transiently filter a snapshot. That pattern should stay intact. New UI affordances should call back into `TermWindow` and existing client/mux APIs, not store long-lived workflow state inside overlays or the tabbar.

The highest-risk gaps are mouse semantics and metadata editing. The current Task Center loop is keyboard-first and has no mouse handling at all, while launcher/selector overlays already implement scroll and left-click behavior. Metadata reads already use async client-domain RPC, but there is no equivalent UI mutation flow yet. The safest Phase 6 plan is to extend Task Center with explicit click targets and reuse prompt/confirm overlays plus existing workspace status/progress RPCs, keeping the tabbar as a passive attention rail.

**Primary recommendation:** Implement Phase 6 as an additive upgrade to `Task Center` plus compact tabbar affordances, with UI actions dispatched through `TermWindow` and metadata edits routed through existing prompt/confirm overlays and workspace RPCs.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `kaku-gui` | `0.8.0` | Native operator UI host | This is the shipped GUI crate and already owns overlays, commands, tabbar rendering, and `TermWindow`. |
| `mux` | `0.1.0` | Source-of-truth control-plane state | Already owns notifications, workspace metadata, task-pane lifecycle state, and the Task Center snapshot builder. |
| `termwiz` | `0.24.0` | Terminal-native overlay rendering and input | Existing overlays render with `TermWizTerminal` and consume `InputEvent`, `KeyEvent`, and `MouseEvent` here. |
| `window` | `0.1.0` | Native window/event surface | Already used for window focus, notifications back into `TermWindow`, and native Kaku shell integration. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `codec` | `0.1.0` | Stable request/response contracts | Use for any UI metadata mutation that must travel through the existing RPC/client path. |
| `wezterm-client` | `0.1.0` | Async client-domain RPC wrapper | Use from GUI for workspace metadata mutation/read flows so remote/current-domain behavior stays aligned. |
| `config` / `KeyAssignment` | `0.1.0` | Command/menu/config integration | Use when exposing discoverable commands, shortcuts, and prompt/confirm actions. |
| `textwrap` | `0.16.2` | Compact confirm dialog wrapping | Already used by confirm overlays; reuse for terse prompt/confirm UX rather than adding layout helpers. |
| `clap` | `4.0` workspace pin | CLI fallback and contract parity | Keep CLI as advanced fallback, not the primary UI path. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Existing full-tab overlay | Sidebar/inspector shell | Violates locked constraints and drifts away from Kaku's additive feel. |
| Existing `TaskCenterEntry` snapshot | Separate UI-only view model store | Creates stale-state bugs and duplicates mux-owned truth. |
| Existing prompt/confirm overlays | Dedicated metadata editor panel | More surface area, more shell drift, and less alignment with current Kaku interaction patterns. |
| Existing command/menu integration | Hidden hotkeys or CLI-only actions | Fails UI-01 discoverability. |

**Installation:** No new dependencies are recommended for Phase 6. Use the existing workspace crates and patterns.

**Version verification:** Versions above come from repo-local manifests and `cargo metadata` on 2026-03-27. This phase should avoid adding new GUI dependencies unless planning uncovers a truly blocking gap.

## Architecture Patterns

### Recommended Project Structure
```text
kaku-gui/src/
├── commands.rs          # Menu + command palette discoverability
├── overlay/
│   ├── task_center.rs   # Primary operator surface
│   ├── prompt.rs        # Metadata edit input flow
│   └── confirm.rs       # Destructive confirmation flow
├── tabbar.rs            # Passive unread/status/progress rail
└── termwindow/mod.rs    # Action dispatch, snapshot cache, overlay bootstrap

mux/src/
└── task_center.rs       # Normalized snapshot rows; keep source-of-truth here
```

### Pattern 1: Mux-Owned Snapshot, GUI-Owned Presentation
**What:** `mux` builds `TaskCenterEntry` rows, `TermWindow` caches them, and the overlay renders/filters that snapshot locally.
**When to use:** For any operator surface that must combine unread, failed, running, workspace, and rerun state.
**Example:**
```rust
// Source: mux/src/task_center.rs, kaku-gui/src/termwindow/mod.rs
let entries = Mux::get().task_center_snapshot();
let (overlay, future) = start_overlay(self, &tab, move |_tab_id, term| {
    crate::overlay::task_center::task_center(entries, term, window)
});
```

### Pattern 2: Full-Tab Overlay Bootstrap
**What:** Create overlays with `start_overlay`, assign them to the active tab, and let the cancellation lifecycle stay inside the existing overlay manager.
**When to use:** For Task Center and any metadata edit/confirm affordance that should feel like Kaku rather than a floating app shell.
**Example:**
```rust
// Source: kaku-gui/src/overlay/mod.rs, kaku-gui/src/termwindow/mod.rs
let (overlay, future) = start_overlay(term_window, &tab, move |_tab_id, term| {
    launcher(args, term, window, initial_choice_idx)
});
self.assign_overlay(tab.tab_id(), overlay);
promise::spawn::spawn(future).detach();
```

### Pattern 3: Overlay -> TermWindow Action Handoff
**What:** Overlays should not mutate long-lived state directly; they should notify `TermWindow`, which already owns focus changes, unread clearing, rerun logic, and title refresh.
**When to use:** Focus, clear unread, rerun/respawn, and future inline metadata actions.
**Example:**
```rust
// Source: kaku-gui/src/overlay/task_center.rs
window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
    let _ = term_window.focus_task_center_entry(&entry);
})));
```

### Pattern 4: Metadata Read/Write Through Client-Domain RPC
**What:** The GUI already fetches workspace metadata through `ClientDomain` and `wezterm-client`. UI metadata edits should follow the same path.
**When to use:** Setting/clearing workspace status and progress from UI.
**Example:**
```rust
// Source: kaku-gui/src/termwindow/mod.rs, crates/wezterm-client/src/client.rs
let inner = ClientDomain::get_client_inner_for_domain(domain_id)?;
let statuses = inner
    .client
    .list_workspace_status(ListWorkspaceStatus { workspace: None })
    .await?;
```

### Pattern 5: Tabbar as Visibility Rail Only
**What:** The tabbar appends compact unread and workspace metadata suffixes and should stay text-led.
**When to use:** Passive attention surfacing and Task Center entry affordances, not full workflows.
**Example:**
```rust
// Source: kaku-gui/src/tabbar.rs
fn prefix_unread_marker(tab: &TabInformation, title: String) -> String {
    if tab.unread_notification_count > 0 {
        format!("! {title}")
    } else {
        title
    }
}
```

### Anti-Patterns to Avoid
- **Dashboard shell rewrite:** Do not replace Kaku's frame with a split dashboard, side panel, or browser-like inspector.
- **Overlay-owned source of truth:** Do not persist operator state inside `TaskCenterOverlay`; treat it as a transient filtered view.
- **Tabbar workflow hub:** Do not stuff lifecycle editing or multi-step workflows into the tabbar.
- **Polling refresh loops:** Prefer mux-triggered refresh and existing `TermWindowNotif` update paths over background polling.
- **Single-click immediate launch without selection state:** The UI spec requires single-click selection and double-click primary action, so launcher-style left-click-to-activate cannot be copied blindly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Primary operator shell | New dashboard/sidebar/window | Existing full-tab `Task Center` overlay | Preserves Kaku feel and reuses shipped overlay lifecycle. |
| Operator data model | UI-only row store | `mux::task_center::TaskCenterEntry` snapshot | Keeps unread/failed/rerun/workspace truth centralized. |
| Metadata edit UI | New editor framework | Existing `prompt.rs` + `confirm.rs` overlays | Already matches Kaku's terse, modal, terminal-native interaction style. |
| Action dispatch | Ad hoc closures from multiple surfaces | `TermWindowNotif::Apply` -> `TermWindow` helpers | Existing focus/unread/rerun flows already live there. |
| Mouse row behavior | Entirely custom mouse grammar | Extend current overlay mouse patterns from `launcher.rs` / `selector.rs` | Reuses known scroll/select mechanics and minimizes UI inconsistency. |
| Metadata transport | GUI-only direct store mutation | Existing `codec` + `wezterm-client` workspace RPCs | Keeps local and client-domain behavior aligned. |

**Key insight:** Phase 6 should mostly compose existing pieces. The repo already has the hard parts: snapshot assembly, overlay hosting, command discoverability, and action dispatch. The plan should spend effort on surface polish and parity, not new architecture.

## Common Pitfalls

### Pitfall 1: Treating Task Center as a New Product Surface
**What goes wrong:** The phase grows into a dashboard shell, inspector panel, or separate app-within-app.
**Why it happens:** UI-01 and UI-02 can tempt broader visual redesign.
**How to avoid:** Keep Task Center a full-tab overlay with one normalized row list and terse header/help lines.
**Warning signs:** New sidebars, card stacks, persistent inspectors, or multi-pane operator layouts appear in the plan.

### Pitfall 2: Duplicating Mux Truth in the GUI
**What goes wrong:** Clear unread, rerun, or metadata actions update one view but leave other surfaces stale or inconsistent.
**Why it happens:** Overlay code starts owning state instead of asking `mux`/`TermWindow` for a fresh snapshot.
**How to avoid:** After each action, refresh the snapshot from `Mux::get().task_center_snapshot()` and keep `TermWindow` responsible for title/cache refresh.
**Warning signs:** New long-lived maps/vectors inside overlays or tabbar code for unread/failed state.

### Pitfall 3: Bypassing Client-Domain RPC for Metadata Edits
**What goes wrong:** UI edits work only for one local path and diverge from existing CLI/domain behavior.
**Why it happens:** It is tempting to call mux setters directly from GUI code because local state is in-process.
**How to avoid:** Route metadata edits through the existing `wezterm-client` workspace RPC methods, mirroring the current read path.
**Warning signs:** UI code mutates workspace metadata without touching `codec`/`wezterm-client` or domain resolution.

### Pitfall 4: Copying Launcher Mouse Semantics Directly
**What goes wrong:** Single left click activates immediately, which conflicts with the Phase 6 UI contract of single-click select and double-click activate.
**Why it happens:** `launcher.rs` and `selector.rs` already have simple mouse handlers that launch on left click.
**How to avoid:** Plan explicit click-streak/double-click handling for Task Center rows before implementation starts.
**Warning signs:** The implementation proposal says "reuse launcher mouse handling as-is."

### Pitfall 5: Forgetting Termwiz MouseEvent Limitations
**What goes wrong:** The plan assumes overlays already receive double-click metadata.
**Why it happens:** `termwiz::input::MouseEvent` only contains `x`, `y`, `mouse_buttons`, and `modifiers`; it does not carry click count.
**How to avoid:** Treat double-click as a real design/implementation question and decide whether to plumb click-streak state from `TermWindow` or emulate it inside the overlay loop.
**Warning signs:** Design docs mention double-click behavior without naming where click streak is tracked.

### Pitfall 6: Overloading the Tabbar
**What goes wrong:** The tabbar becomes a dense action surface and loses Kaku's text-led feel.
**Why it happens:** It is visible everywhere, so it is tempting to make it the main workflow hub.
**How to avoid:** Keep the tabbar limited to compact visibility markers and optional Task Center entry affordances.
**Warning signs:** Plans add multiple buttons, badges, or editing flows directly into tab titles.

## Code Examples

Verified patterns from the current codebase:

### Task Center Command Discoverability
```rust
// Source: kaku-gui/src/commands.rs
ShowTaskCenter => CommandDef {
    brief: "Task Center".into(),
    doc: "Open the task center overlay".into(),
    keys: vec![(Modifiers::SUPER.union(Modifiers::SHIFT), "j".into())],
    args: &[ArgType::ActiveWindow],
    menubar: &["Shell"],
    icon: None,
},
```

### Overlay Rendering Pattern
```rust
// Source: kaku-gui/src/overlay/task_center.rs
Change::Text(
    "Enter=focus C=clear unread R=rerun Esc=close | tokens: unread failed running workspace:<name> source:<kind> kind:<kind>"
        .to_string()
        .into(),
),
```

### Clear-Unread Handoff
```rust
// Source: kaku-gui/src/termwindow/mod.rs
let updated = Mux::get().mark_notifications_read(&notification_ids);
if updated > 0 {
    self.refresh_task_center_cache();
    self.update_title_post_status();
}
```

### Metadata Prompt Primitive
```rust
// Source: kaku-gui/src/overlay/prompt.rs
editor.set_prompt(&args.prompt);
let line =
    editor.read_line_with_optional_initial_value(&mut host, args.initial_value.as_deref())?;
```

### Confirm Overlay With Mouse Support
```rust
// Source: kaku-gui/src/overlay/confirm.rs
if y == button_row && x >= yes_button.x && x < yes_button.x + yes_button.width {
    active = ActiveButton::Yes;
    if mouse_buttons == MouseButtons::LEFT {
        return Ok(true);
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| CLI recall for control-plane actions | Native `ShowTaskCenter` command in menu/palette/shortcut | Phase 3 | Discoverability exists, but common-path UI still needs richer affordances. |
| Bell-only unread surfacing | Mux-backed unread counts on Task Center and tabbar suffix/prefix formatting | Phases 1 and 3 | The data model is already unified enough for richer operator UI. |
| Task Center as keyboard-only overlay | Full-tab overlay with filtering, focus, clear unread, and rerun hooks | Phase 3 | Mouse parity and metadata edits are the missing pieces, not the overlay shell itself. |
| CLI-only workspace metadata mutation | RPC-backed workspace metadata contracts exist; GUI only reads them today | Phase 2 onward | Phase 6 can add UI mutation flows without inventing new transport. |

**Deprecated/outdated:**
- Treating Task Center as a Phase 3-only overlay is outdated. It is now the intended first-class operator surface for Phase 6.
- Treating tabbar unread markers as bell-only state is outdated. The tabbar already consumes mux-backed unread and workspace metadata fields.

## Open Questions

1. **How should Task Center implement double-click activation?**
   - What we know: The UI contract wants single-click select and double-click activate. `launcher.rs` activates on single left click, and `termwiz::input::MouseEvent` carries no click count.
   - What's unclear: Whether to plumb `TermWindow` click-streak behavior into overlays or implement overlay-local timing/state.
   - Recommendation: Resolve this explicitly in planning before any mouse-task starts. This is the main design/implementation fork in the phase.

2. **Which metadata flows are "common path" for UI-04?**
   - What we know: Status and progress already have codec/client support, prompt overlays support initial values, and confirm overlays support destructive actions.
   - What's unclear: Whether Phase 6 should include status only, status + progress, or status + progress + log append/clear in the first wave.
   - Recommendation: Plan status and progress as required common-path edits. Treat log editing as optional unless the user explicitly wants it in this phase.

3. **Should tabbar clicks open a prefiltered Task Center view?**
   - What we know: The UI spec allows a tabbar operator marker to open Task Center prefiltered to current workspace/tab, but forbids turning tabbar into a workflow hub.
   - What's unclear: Whether the best first implementation is click-to-open, hover-only, or keyboard/menu only.
   - Recommendation: Treat tabbar click-to-prefilter as a secondary enhancement after Task Center row actions and metadata prompts are planned.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build/test/verification | ✓ | `1.93.0` | — |
| `rustc` | Native Rust build | ✓ | `1.93.0` | — |
| `rustfmt` | Phase formatting gate | ✓ | `1.8.0-stable` | — |
| macOS desktop runtime | Native GUI manual UAT | ✓ | `macOS 15.7.4` | — |

**Missing dependencies with no fallback:**
- None identified from the current environment audit.

**Missing dependencies with fallback:**
- None identified from the current environment audit.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in tests via `cargo test` |
| Config file | none |
| Quick run command | `cargo test -p kaku-gui task_center` |
| Full suite command | `cargo test -p mux && cargo test -p kaku-gui && cargo test -p kaku` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UI-01 | Task Center remains discoverable via native command surfaces | unit | `cargo test -p kaku-gui show_task_center_has_default_shortcut` | ✅ |
| UI-02 | Operator rows expose unread/failed/running/workspace context and filters | unit | `cargo test -p kaku-gui task_center` | ✅ |
| UI-03 | Focus / clear unread / rerun-action gating remain correct | unit | `cargo test -p kaku-gui task_center && cargo test -p mux task_center` | ✅ |
| UI-04 | Metadata edit flow invokes correct prompt/confirm and RPC plumbing | manual + unit | `cargo test -p kaku-gui task_center` | ❌ Wave 0 |
| UI-05 | Surface stays additive and Kaku-native under real runtime use | manual UAT | `cargo test -p kaku-gui task_center` | ❌ manual-only |

### Sampling Rate
- **Per task commit:** `cargo test -p kaku-gui task_center`
- **Per wave merge:** `cargo test -p mux task_center && cargo test -p kaku-gui task_center && cargo test -p kaku compatibility_cli`
- **Phase gate:** Full suite green plus live GUI UAT for mouse flows, metadata edits, and shell additivity before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `kaku-gui/src/overlay/task_center.rs` — add mouse interaction tests for row selection, wheel scroll, and click gating before implementing row actions.
- [ ] `kaku-gui/src/termwindow/mod.rs` — add tests for any new helper that constructs metadata mutation requests or decides action availability.
- [ ] `kaku-gui/src/commands.rs` — add tests if new operator commands/menu entries are introduced beyond `ShowTaskCenter`.
- [ ] Manual UAT script/artifact — define explicit steps for mouse select vs activate, prompt/confirm flows, and tabbar-to-Task-Center affordances because no GUI integration harness exists today.

## Sources

### Primary (HIGH confidence)
- `./CLAUDE.md` - locked repo constraints and forbidden directions
- `./AGENTS.md` - Kaku fork rules, extension points, and quality gates
- `./docs/KAKU_CONTROL_PLANE_SPEC.md` - product shape and per-phase intent
- `./PLANS.md` - shipped Phase 3/4/5 outcomes and additive-surface guidance
- `./.planning/phases/06-operator-ui-surfaces/06-CONTEXT.md` - locked user decisions and discretion areas
- `./.planning/phases/06-operator-ui-surfaces/06-UI-SPEC.md` - approved interaction and visual contract for Phase 6
- `./.planning/REQUIREMENTS.md` - UI-01 through UI-05 requirement definitions
- `./kaku-gui/src/overlay/task_center.rs` - current Task Center behavior, action gating, and tests
- `./kaku-gui/src/termwindow/mod.rs` - snapshot caching, focus/clear/rerun helpers, metadata reads, and overlay bootstrap
- `./kaku-gui/src/tabbar.rs` - compact unread and workspace metadata rendering
- `./kaku-gui/src/commands.rs` - menu/palette discoverability and shortcut definition
- `./kaku-gui/src/overlay/prompt.rs` - prompt overlay primitive with initial value support
- `./kaku-gui/src/overlay/confirm.rs` - confirm overlay with mouse-capable buttons
- `./kaku-gui/src/overlay/launcher.rs` and `./kaku-gui/src/overlay/selector.rs` - current overlay mouse/scroll handling patterns
- `./mux/src/task_center.rs` - normalized Task Center snapshot builder
- `./crates/wezterm-client/src/client.rs` - existing workspace metadata RPC methods
- `./crates/codec/src/lib.rs` - stable workspace metadata request/response contracts
- `cargo metadata --no-deps --format-version 1` - repo-local crate version verification

### Secondary (MEDIUM confidence)
- `./termwiz/src/input.rs` - confirms `MouseEvent` shape lacks click-count metadata; useful for planning but still requires implementation validation in overlay context

### Tertiary (LOW confidence)
- None. This research did not rely on unverified web/community sources because the repo source and approved planning artifacts are authoritative for this phase.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all recommended crates and versions come from the current workspace manifests and running `cargo metadata`.
- Architecture: HIGH - recommendations directly follow the shipped `mux`, `TermWindow`, overlay, and tabbar seams already in the repo.
- Pitfalls: MEDIUM - most pitfalls are source-backed, but double-click handling and common-path metadata scope still need an explicit planning decision.

**Research date:** 2026-03-27
**Valid until:** 2026-04-10
