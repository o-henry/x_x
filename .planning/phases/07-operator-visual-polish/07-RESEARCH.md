# Phase 07: Operator Visual Polish - Research

**Researched:** 2026-03-28
**Domain:** Native Rust/Kaku visual restructuring toward the provided reference video
**Confidence:** MEDIUM

<user_constraints>
## User Constraints

No `07-CONTEXT.md` exists for this phase. The constraints below come from the current user prompt plus repo rules in `AGENTS.md` and `CLAUDE.md`.

### Locked Decisions
- Start planning from three questions in this order: what to imitate, how to implement, and what technology is required.
- Keep the existing Kaku fork plus shipped control-plane behavior; do not turn this into a different app shell.
- Move the UI materially closer to the provided reference video and extracted frames.
- Separate concrete visual observations from interpretation.
- Match these elements first: left rail, top chrome, main work-surface hierarchy, secondary context surface, iconography, and typography density.
- Rust-native only.
- No Swift, Xcode, React, Electron, Tauri, or WebView shell.
- Current-session request priority overrides conservative vanilla-Kaku preservation; close reference matching is more important than preserving the old chrome/layout.
- Do not pretend the reference can be matched if the current rendering path cannot do it.
- Do not write implementation code in this research artifact.

### Claude's Discretion
- How much of the reference can be achieved inside the existing `kaku-gui` renderer before a larger native container/widget layer is justified.
- Whether the secondary context surface in Phase 07 should be a persistent rendered strip, a disciplined use of existing pane splits, or remain deferred.
- How to slice the implementation into 3-5 executable plan files without wasting another iteration cycle.

### Deferred Ideas
- Web/browser embedding
- WebKitGTK usage
- Replatforming the app around a non-Kaku shell
- Large cross-toolkit rewrite as the default Phase 07 plan
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| POLISH-01 | Developer sees DM Mono as the default operator-facing mono typography in the shipped UI surfaces. | DM Mono is already the repo default in `config/src/font.rs` and bundled config; planning should focus on propagating it into chrome surfaces that still visually read as legacy Kaku. |
| POLISH-02 | Developer sees compact icon-first state and action affordances instead of text-heavy markers where clarity benefits. | Replace text-heavy operator labels and suffixes first in the persistent rail and top chrome, then in Task Center row affordances. |
| POLISH-03 | Developer sees Task Center and tabbar composition that feels materially closer to the provided video reference while still preserving Kaku's additive shell character. | Treat the reference as a main-window layout problem first; make rail/chrome/work-surface structure match before further overlay polish. |
| POLISH-04 | Developer can keep using the Phase 6 operator workflow after the visual polish without regressions to focus, rerun, remain-on-exit, metadata editing, or discoverability. | Keep control-plane truth in `mux`/`TermWindow`; restrict Phase 07 to rendering, hit-target, and composition changes with regression coverage carried forward from Phase 06. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Read and follow `AGENTS.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, and `PLANS.md`.
- Product scope is defined by `docs/KAKU_CONTROL_PLANE_SPEC.md` and `PLANS.md`.
- Hard constraints: Rust-only, no Swift/Xcode, no React/Electron/Tauri/WebView, no copied cmux/tmux code, preserve Kaku feel and architecture.
- Preferred extension points: `kaku/src/cli`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay`, `kaku-gui/src/termwindow`, `mux/src`.
- Do not start with sidebar, browser, PR UI, socket daemon, or full tmux detached semantics.

## Summary

Phase 07 should not be planned as "more Task Center polish." The reference frames show a persistent application layout with a slim full-height left rail, minimal top chrome, a visually dominant main work surface, and a persistent secondary context region. The current Phase 06 implementation is still centered on a full-tab overlay plus a text-heavy operator rail and tabbar suffixes. That mismatch is structural, not cosmetic.

The good news is that the repo already has the native Rust seams needed for one serious structural pass without introducing a new GUI stack. `kaku-gui/src/termwindow/render/paint.rs` already paints a persistent operator rail, `kaku-gui/src/termwindow/render/fancy_tab_bar.rs` already owns custom titlebar/tab composition, `kaku-gui/src/termwindow/mouseevent.rs` already routes hit-tested UI items, and `kaku-gui/src/overlay/task_center.rs` already has a compact section rail plus inline icon actions. That means Phase 07 can and should attempt a reference-first rewrite of persistent rail/chrome composition inside the current renderer before considering a bigger native container layer.

The honest limit is the secondary context surface. The current renderer can plausibly deliver a slimmer rail, quieter chrome, tighter iconography, better density, and a more reference-like hierarchy around existing panes. It is much less suited to building a rich persistent inspector/widget stack with complex controls, independent scrolling, and resizable container semantics. If the target still reads as overlay-driven or card-like after a deliberate rail/chrome/work-surface pass, the next step should be a separate architecture phase for a larger Rust-native container/widget layer, not another round of superficial polish.

**Primary recommendation:** Plan Phase 07 as a reference-first rewrite of persistent left rail plus top chrome inside the existing `kaku-gui` renderer, explicitly demoting overlay polish to secondary work and using a stop-condition that escalates to a dedicated native container architecture phase if the layout still cannot read like the reference.

## Visual Facts

### Concrete Layout Observations

Observed from `/tmp/kaku_ref_003.png`, `/tmp/kaku_ref_015.png`, and `/tmp/kaku_ref_030.png`:

- The left rail is persistent, edge-attached, full-height, and visually narrower than the current in-repo operator nav. It does not read like a floating card.
- The left rail uses compact rows with subtle active-state treatment. It carries workspace/navigation identity continuously rather than only inside a modal surface.
- The top chrome is a thin dark strip with small controls and low-contrast separators. It is visually quieter than the current Kaku fancy-tab composition.
- The main work surface begins immediately to the right of the left rail and dominates the window.
- The content hierarchy is persistent: the right side is not a popup. In the clearest frame, the window is split into a large left work area and a right column that is itself vertically divided.
- Typography is dense and compact. Labels are short. The UI relies on small iconography and low-contrast chrome rather than verbose helper text.
- The palette stays dark and restrained. Accent use is sparse and local.
- There are no floating dashboard cards, large rounded containers, or oversized pill markers in the visible structure.

### Interpretation For Planning

- The reference is primarily a window-composition target, not a row-styling target.
- The current persistent rail should be judged against the reference rail, not against the overlay's internal section rail.
- The current Task Center overlay is still useful, but it should no longer be treated as the main visual analogy for the reference.
- Matching the reference means changing where operator information lives in the main window hierarchy, not only how text markers are drawn.

## Current Gap Analysis

### What Exists Today

- `kaku-gui/src/termwindow/render/paint.rs` already paints a persistent `operator_nav` strip with hit targets and counts.
- `kaku-gui/src/termwindow/mouseevent.rs` already routes clicks for `UIItemType::OperatorNav`.
- `kaku-gui/src/termwindow/render/fancy_tab_bar.rs` already builds custom titlebar/tab elements using the box model.
- `kaku-gui/src/tabbar.rs` already exposes unread, status, progress, and operator markers.
- `kaku-gui/src/overlay/task_center.rs` already has icon-oriented row actions, a left section rail, and mouse interaction.
- DM Mono is already the default repo font family in `config/src/font.rs`, and the bundled `kaku.lua` adds the DM Mono directory and font rules when present.

### Where It Still Misses The Reference

- The persistent rail is too wide, too labeled, and too panel-like. `paint_operator_nav` still draws a branded "KAKU" block, uppercase workspace header, labeled rows, active background fills, and a card-like text panel rather than a slim navigation rail.
- The top chrome still reads like Kaku tab UI rather than restrained app chrome. `fancy_tab_bar.rs` emphasizes padded tab capsules and decorated titlebar composition.
- Passive surfacing is still text-heavy. `tabbar.rs` continues to append readable text suffixes such as workspace status and progress in the tab title path, plus the `◈` operator marker.
- The most polished left-rail treatment currently lives inside `TaskCenterOverlay`, but that is still an overlay, not persistent window structure.
- The secondary context surface is not yet a real persistent layout region. Today it exists as either overlay content or normal pane layout, but not as a deliberate operator-context surface.

### Practical Planning Implication

- Phase 07 must explicitly reverse the Phase 06 visual center of gravity:
  1. persistent rail first
  2. top chrome second
  3. work-surface hierarchy third
  4. overlay row polish last

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `kaku-gui` | `0.8.0` | Native GUI host for chrome, overlays, hit testing, and layout | The existing app shell already owns the surfaces that need the visual rewrite. |
| `window` | `0.1.0` | Native window integration and UI item routing | Current hit-tested chrome interactions already flow through this crate and `UIItemType`. |
| `termwiz` | `0.24.0` | Terminal-native input/surface model | The overlay/input path already depends on it; no new UI runtime is needed for the recommended Phase 07 pass. |
| `wezterm-font` / config font stack | workspace-local + bundled config | DM Mono, fallback fonts, title-font resolution | The typography requirement is already supported by the repo's font system. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `mux` | `0.1.0` | Source-of-truth operator counts and snapshot state | Use for unread/failed/running/status counts and scoped Task Center entry data. |
| `kaku-gui` box model (`Element`, `ComputedElement`, `UIItemType`) | repo-local | Custom chrome composition and hit testing | Use for the persistent rail and top chrome rewrite inside the current renderer. |
| Bundled `kaku.lua` font config | bundled repo/runtime asset | DM Mono default stack and window-frame defaults | Use to keep typography changes aligned with shipped runtime config. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Current renderer + box model rewrite | GTK4/libadwaita container rewrite | Could deliver richer native layout semantics, but it is a large architectural move and not additive to current Kaku surfaces. |
| Current renderer + pane hierarchy | WebKitGTK/browser-style context pane | Violates the explicit no-WebView-shell direction and is not needed for Phase 07. |
| Persistent rail/chrome rewrite | More Task Center overlay styling | Faster but misses the reference's structural hierarchy and likely repeats Phase 06's wasted iteration pattern. |

**Installation:** No new dependency is recommended for Phase 07's primary path.

**Version verification:** Versions above were verified from local workspace metadata on 2026-03-28 via `cargo metadata --no-deps --format-version 1`. The GTK4/libadwaita fallback path is intentionally not recommended for this phase, though the machine does currently report `gtk4 4.22.1` and `libadwaita-1 1.9.0` via `pkg-config`.

## Architecture Patterns

### Recommended Project Structure
```text
kaku-gui/src/
├── termwindow/
│   ├── mod.rs                  # width thresholds, nav activation, scoped Task Center handoff
│   ├── mouseevent.rs           # persistent rail hit testing
│   └── render/
│       ├── paint.rs            # persistent operator rail and structural chrome positioning
│       ├── fancy_tab_bar.rs    # top chrome/tab/title composition
│       └── tab_bar.rs          # bar height and viewport offset coupling
├── tabbar.rs                   # compact passive markers only
└── overlay/task_center.rs      # secondary operator surface, not the main structural target

assets/macos/Kaku.app/Contents/Resources/
└── kaku.lua                    # DM Mono defaults and runtime window-frame config
```

### Pattern 1: Reference Facts First, Interpretation Second
**What:** Lock a short list of observable structural facts before proposing implementation.
**When to use:** Before any plan file is written, and whenever the work risks drifting back into generic "polish."
**Example:** The reference has a persistent left rail and persistent right-side context region; therefore Phase 07 should not treat the overlay as the main target surface.

### Pattern 2: Persistent Chrome In `TermWindow`, Not In Overlay
**What:** Put reference-matching structure into `paint.rs`, `fancy_tab_bar.rs`, `tab_bar.rs`, and `mouseevent.rs`.
**When to use:** Left rail, title strip, viewport start position, compact markers, and chrome hit targets.
**Example:**
```rust
// Source: kaku-gui/src/termwindow/render/paint.rs
fn paint_operator_nav(&mut self) -> anyhow::Result<()> {
    if !self.operator_nav_enabled() {
        return Ok(());
    }
    // Persistent, hit-tested rail is already painted here.
}
```

### Pattern 3: Overlay As Secondary Context Surface
**What:** Keep Task Center as a scoped inspector and action surface, not the main composition analogy.
**When to use:** Search, filtered action execution, metadata editing, and keyboard-heavy operator workflows.
**Example:**
```rust
// Source: kaku-gui/src/termwindow/mod.rs
pub(crate) fn activate_operator_nav(&mut self, item: OperatorNavItem) {
    self.operator_nav_selection = item;
    match item.query() {
        Some(query) => self.show_task_center_with_scope(TaskCenterScope::with_query(query)),
        None => self.show_task_center(),
    }
}
```

### Pattern 4: Use Existing Pane Hierarchy Before Inventing A New Inspector Widget
**What:** If a persistent secondary context region is needed, prefer composition around existing pane splits and reserved native chrome regions before inventing a rich new widget stack.
**When to use:** Right-side context affordances, status summaries, or split-aware work-surface hierarchy changes.
**Example:** Phase 07 can make the right-side area read as intentional by tightening chrome, spacing, and viewport boundaries without needing a full custom inspector subsystem.

### Pattern 5: Stop When The Renderer Starts Fighting The Layout Goal
**What:** Bake a stop-condition into the plan.
**When to use:** After the left rail and top chrome passes.
**Example:** If the rail still reads as a card and the top strip still reads as stock Kaku tabs after the structural rewrite, stop and plan a separate architecture phase for a richer Rust-native container/widget layer.

### Anti-Patterns to Avoid
- **Overlay-first planning:** Treating `Task Center` as the main reference analog.
- **Color-only polish:** Changing palette/spacing while leaving the same hierarchy.
- **Text-to-icon swap without layout change:** Replacing labels with glyphs but leaving the rail/chrome proportions intact.
- **Large toolkit pivot as a hidden scope creep:** Smuggling GTK4/libadwaita into Phase 07 under the name of polish.

## Feasible Native Strategy

### Realistically Achievable In The Current Rust/Kaku Rendering Path

- DM Mono-first chrome cleanup across title/tab/rail surfaces
- Slimmer full-height persistent left rail with subtler active state
- Lower-noise top chrome by reducing tab capsule treatment and label density
- Stronger viewport hierarchy so the main pane area starts immediately to the right of the rail
- Compact iconography for unread, running, failed, status, and progress markers
- Better parity between persistent rail selection and scoped Task Center entry behavior

### Possible But Risky In The Current Path

- A modest persistent secondary context strip rendered inside the existing box model
- More sophisticated chrome hit targets and section switching beyond the current nav click model
- Deliberate visual framing for a right-side context region without drifting into a dashboard

### Likely Beyond The Current Path For This Phase

- A rich persistent inspector with multiple independently interactive widgets, complex scrolling regions, and resizable container semantics
- A true browser-like or IDE-like panel system with toolkit-grade layout primitives
- High-fidelity imitation of non-terminal content types from the reference video

### Bigger Native Container/Widget Assessment

- A larger Rust-native container/widget approach is technically possible in principle.
- This machine does have `gtk4` and `libadwaita-1` available, so a future exploration would not be blocked by missing system libraries.
- That said, switching this repo's main UI architecture toward GTK4/libadwaita would be a much bigger change than "Phase 07 polish," and it would not preserve the current Kaku renderer as an additive extension.
- For this repo, the more pragmatic fallback is not "rewrite Phase 07 in GTK"; it is "stop after the structural pass and create a new architecture phase for richer native containerization inside Rust."

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Reference alignment | Another overlay-only redesign | Persistent rail/chrome rewrite in `termwindow/render/*` | The reference is about window structure, not overlay styling. |
| Typography reset | Custom ad hoc font plumbing | Existing DM Mono defaults in `config/src/font.rs` and bundled `kaku.lua` | The repo already has the operator font stack. |
| Secondary context surface | Web/browser shell | Existing panes plus native chrome composition | Stays within hard constraints and Kaku architecture. |
| Native container escalation | Silent toolkit pivot during Phase 07 | Explicit stop-condition and separate architecture phase | Avoids wasted iteration and hidden scope growth. |

**Key insight:** Phase 07 should spend effort on structural relocation of operator affordances, not on inventing new control-plane behavior or new rendering technologies.

## Common Pitfalls

### Pitfall 1: Solving The Wrong Problem
**What goes wrong:** The plan improves Task Center rows and icons but leaves the main window hierarchy fundamentally unchanged.
**Why it happens:** Phase 06 already invested heavily in overlay quality, so it is tempting to continue there.
**How to avoid:** Make the first two plan files about persistent rail and top chrome only.
**Warning signs:** Most code changes land in `overlay/task_center.rs` before `paint.rs` or `fancy_tab_bar.rs`.

### Pitfall 2: Treating The Current Rail As "Good Enough"
**What goes wrong:** The existing persistent operator nav is kept mostly intact with minor styling tweaks.
**Why it happens:** It already exists and is clickable.
**How to avoid:** Judge it against the reference, not against the previous implementation.
**Warning signs:** The rail still contains large text labels, wide padding, or a branded panel block after the first pass.

### Pitfall 3: Trying To Fake Persistent Context With Overlays
**What goes wrong:** The UI still reads as popup-driven even after visual cleanup.
**Why it happens:** Overlays are easier to change than the main render path.
**How to avoid:** Restrict overlay work to scoped operator actions and filtered inspection.
**Warning signs:** The main comparison screenshots still require opening an overlay to see the intended structure.

### Pitfall 4: Smuggling In A Toolkit Rewrite
**What goes wrong:** The phase balloons into a GTK/libadwaita port because that seems closer to the reference.
**Why it happens:** The related external example makes the alternative attractive.
**How to avoid:** Keep toolkit changes out of Phase 07 and use the stop-condition honestly.
**Warning signs:** New crate additions, new app shell abstractions, or a new window/widget runtime appear in the plan.

### Pitfall 5: Breaking Phase 06 Behavior While Chasing Polish
**What goes wrong:** Focus, rerun, unread clearing, metadata editing, or discoverability regress because the work touches the same surfaces.
**Why it happens:** Phase 07 must modify the same files that currently host those behaviors.
**How to avoid:** Keep state ownership in `mux`/`TermWindow`; treat rendering and routing as separate from behavior.
**Warning signs:** New UI state stores appear outside `TermWindow` or `mux`.

## Code Examples

Existing seams that support the recommended strategy:

### Persistent Operator Nav Already Exists
```rust
// Source: kaku-gui/src/termwindow/render/paint.rs
const OPERATOR_NAV_WIDTH_PX: usize = 92;
const OPERATOR_NAV_MIN_WINDOW_WIDTH: usize = 720;
```

This means the current renderer already reserves persistent structure in the main window.

### DM Mono Is Already The Default Font Family
```rust
// Source: config/src/font.rs
impl Default for FontAttributes {
    fn default() -> Self {
        Self {
            family: "DM Mono".into(),
            // ...
        }
    }
}
```

### Task Center Already Has Compact Icon Actions
```rust
// Source: kaku-gui/src/overlay/task_center.rs
const ICON_FOCUS: &str = "⏎";
const ICON_CLEAR: &str = "●";
const ICON_RERUN: &str = "↺";
const ICON_HOLD: &str = "◎";
```

This makes Task Center a good secondary reference for icon language, but not the primary structural target.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Overlay-centric operator UI | Persistent rail plus overlay inspector | Phase 06/07 transition | Needed to match the reference's always-on composition. |
| Text-heavy tab/title suffixes | Icon-first compact chrome markers | Phase 07 target | Reduces noise and improves hierarchy. |
| Generic Kaku tab chrome | Purpose-built operator chrome | Phase 07 target | Required for reference alignment. |

**Deprecated/outdated for this phase:**
- Treating `Task Center` as the primary visual analog for the reference
- Spending another iteration primarily inside overlay row styling

## Recommended Plan Slicing

### 07-01-PLAN.md — Lock The Visual Contract And Baseline Checkpoints
- Turn the reference observations above into a short acceptance contract.
- Define 3-5 screenshot checkpoints using the extracted frames.
- Record stop-conditions before any implementation starts.

### 07-02-PLAN.md — Rebuild The Persistent Left Rail
- Rewrite `paint_operator_nav` and its hit-target behavior so the rail is slim, full-height, and subtle.
- Remove card/panel cues, reduce text labels, and make active state understated.
- Keep `activate_operator_nav` and scoped Task Center routing behavior intact.

### 07-03-PLAN.md — Rewrite Top Chrome And Tab Composition
- Tighten `fancy_tab_bar.rs`, `tabbar.rs`, and related height/offset coupling.
- Reduce pill/capsule feel, helper-label noise, and tabbar text density.
- Ensure the terminal/editor surface visually begins immediately after the rail.

### 07-04-PLAN.md — Rebalance Main Work Surface And Secondary Context Surface
- Decide what Phase 07 can honestly make persistent: reserved context strip, better split framing, or scoped overlay handoff.
- Make the main pane hierarchy read closer to the reference without pretending to build a full inspector stack.
- If this step cannot achieve the target hierarchy, stop and document the need for an architecture follow-up.

### 07-05-PLAN.md — Final Iconography, Typography, And Verification Closeout
- Finish icon-first markers and typography density cleanup.
- Re-run Phase 06 behavior coverage.
- Produce runtime screenshots, limitations, and a go/no-go conclusion on whether the current renderer is sufficient.

## Open Questions

1. **Should Phase 07 create a modest persistent right-side context strip, or stop at rail/chrome/work-surface hierarchy only?**
   - What we know: The reference has a persistent secondary context area.
   - What's unclear: Whether the repo can imitate that honestly without inventing a fragile pseudo-widget stack.
   - Recommendation: Keep this decision in `07-04-PLAN.md` after the rail/chrome pass, not before.

2. **Should the tabbar stay visible in its current form when the rail becomes the primary navigation identity?**
   - What we know: The top chrome in the reference is much quieter than the current Kaku tab treatment.
   - What's unclear: Whether the best match is a reduced fancy tab bar, a flatter bar, or conditional simplification.
   - Recommendation: Treat this as part of the chrome rewrite, validated by screenshots rather than by code preference.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build/test/verify Phase 07 changes | ✓ | `1.93.0` | — |
| `rustc` | Compile/check native Rust code | ✓ | `1.93.0` | — |
| DM Mono asset at `/Users/henry/Documents/asset/FONT/DM_Mono/DMMono-Regular.ttf` | POLISH-01 typography target | ✓ | local asset | Keep existing fallback stack if user-local asset disappears |
| `gtk4` via `pkg-config` | Future architecture fallback only | ✓ | `4.22.1` | Not needed for Phase 07 |
| `libadwaita-1` via `pkg-config` | Future architecture fallback only | ✓ | `1.9.0` | Not needed for Phase 07 |
| `webkitgtk-6.0` via `pkg-config` | External example pattern only | ✗ | — | None; also out of scope by product direction |

**Missing dependencies with no fallback:**
- None for the recommended Phase 07 path.

**Missing dependencies with fallback:**
- `webkitgtk-6.0` is unavailable, but it is not part of the recommended plan and should remain out of scope.

## Verification Implications

- Visual completion for Phase 07 cannot be declared from unit tests alone.
- The planner should require screenshot-based checkpoints after each structural pass, especially after the rail rewrite and the chrome rewrite.
- Manual review should compare structure first, not colors first.
- Behavior regression verification from Phase 06 must remain in the closeout because Phase 07 touches the same files and hit targets.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` / `cargo check` plus screenshot-based manual UAT |
| Config file | none — existing workspace test infrastructure |
| Quick run command | `cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture` |
| Full suite command | `cargo fmt --all --check && cargo test --locked -p mux task_center -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo test --locked -p kaku show_task_center_has_default_shortcut -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| POLISH-01 | DM Mono remains the default operator-facing font direction | manual/config smoke | `cargo check --locked -p kaku-gui` | ✅ |
| POLISH-02 | Icon-first markers replace text-heavy chrome affordances without breaking actions | unit + manual | `cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture` | ✅ |
| POLISH-03 | Main window structure reads materially closer to the reference | manual screenshot comparison | `cargo check --locked -p kaku-gui` | ✅ code exists, screenshot harness not yet formalized |
| POLISH-04 | Phase 06 workflows still work after polish | regression | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo check --locked -p kaku-gui` | ✅ |

### Sampling Rate
- **Per task commit:** `cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture`
- **Per wave merge:** `cargo check --locked -p kaku-gui`
- **Phase gate:** full suite green plus screenshot comparison against `/tmp/kaku_ref_003.png`, `/tmp/kaku_ref_015.png`, and `/tmp/kaku_ref_030.png`

### Wave 0 Gaps
- [ ] Formalize a screenshot checkpoint workflow for the main window, not just overlay behavior
- [ ] Add at least one targeted regression test for persistent operator-nav hit regions if the rail interaction model changes
- [ ] Define a short human checklist for "rail no longer reads as a card" and "top chrome no longer reads as stock Kaku tabs"

## Sources

### Primary (HIGH confidence)
- Repo-local phase docs:
  - `AGENTS.md`
  - `PLANS.md`
  - `.planning/ROADMAP.md`
  - `.planning/REQUIREMENTS.md`
  - `.planning/phases/06-operator-ui-surfaces/06-UI-SPEC.md`
  - `.planning/phases/06-operator-ui-surfaces/06-RESEARCH.md`
  - `.planning/phases/06-operator-ui-surfaces/06-VALIDATION.md`
  - `.planning/phases/06-operator-ui-surfaces/06-LIMITATIONS.md`
- Repo-local implementation seams:
  - `kaku-gui/src/termwindow/mod.rs`
  - `kaku-gui/src/termwindow/mouseevent.rs`
  - `kaku-gui/src/termwindow/render/paint.rs`
  - `kaku-gui/src/termwindow/render/fancy_tab_bar.rs`
  - `kaku-gui/src/tabbar.rs`
  - `kaku-gui/src/overlay/task_center.rs`
  - `config/src/font.rs`
  - `assets/macos/Kaku.app/Contents/Resources/kaku.lua`
- Reference artifacts supplied by the user:
  - `/tmp/kaku_ref_003.png`
  - `/tmp/kaku_ref_015.png`
  - `/tmp/kaku_ref_030.png`
- Official docs:
  - GTK4 `Paned` docs: https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Paned.html
  - libadwaita `OverlaySplitView` docs: https://docs.rs/libadwaita/latest/libadwaita/struct.OverlaySplitView.html

### Secondary (MEDIUM confidence)
- `cargo metadata --no-deps --format-version 1` for workspace crate versions
- `pkg-config --modversion gtk4`
- `pkg-config --modversion libadwaita-1`

### Tertiary (LOW confidence)
- Related external example named by the user: `https://github.com/am-will/limux`
  - Used only as a category signal for "what a larger widget-container path might resemble"
  - Not used as the implementation recommendation for Phase 07

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - based on repo-local crates, current code, and verified local environment
- Architecture: MEDIUM - current seams are clear, but reference fidelity from the existing renderer is still an informed judgment rather than a proven outcome
- Pitfalls: HIGH - grounded in the current code structure, Phase 06 limitations, and the visible structural mismatch

**Research date:** 2026-03-28
**Valid until:** 2026-04-27

## Planner Recommendation

Plan Phase 07 as a deliberate main-window composition pass, not another overlay-polish pass. Imitate the reference in this order: persistent left rail, top chrome, main work-surface hierarchy, then secondary context surface. Implement it inside the current `kaku-gui` rendering path first, because the repo already has persistent rail rendering, custom chrome composition, hit-tested UI items, and DM Mono support. Do not adopt GTK4/libadwaita as the default Phase 07 technology; keep that as a follow-up architecture option only if the rail/chrome rewrite still cannot make the window read like the reference.
