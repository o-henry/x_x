# Codebase Concerns

**Analysis Date:** 2026-03-26

## Tech Debt

**Oversized orchestration modules:**
- Issue: Several core surfaces concentrate a large amount of logic in single files, which raises change risk and review difficulty.
- Files: `kaku/src/ai_config/tui.rs`, `kaku-gui/src/termwindow/mod.rs`, `mux/src/tab.rs`, `kaku-gui/src/commands.rs`, `mux/src/lib.rs`, `kaku/src/config_tui/mod.rs`
- Impact: Control-plane work that touches these files can easily create regressions outside the immediate feature.
- Fix approach: Prefer submodule-level additions first, and keep changes scoped to the preferred extension points listed in `AGENTS.md`.

**Control-plane state not yet modeled explicitly:**
- Issue: The current mux model already has workspaces, alerts, progress, and overlays, but there is no dedicated notification store, unread index, workspace metadata store, or failed-task registry matching the spec.
- Files: `mux/src/lib.rs`, `mux/src/pane.rs`, `mux/src/tab.rs`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/overlay/launcher.rs`
- Impact: Phase 1-4 features must be introduced carefully so they compose with existing pane/tab/workspace state instead of becoming parallel state systems.
- Fix approach: Add mux-owned state and notification events first, then surface them through CLI and GUI layers.

## Known Bugs

**Clipboard and pixel metadata gaps in pane-management paths:**
- Symptoms: Existing comments mark incomplete clipboard and pixel-size propagation in some pane creation/move flows.
- Files: `mux/src/lib.rs`
- Trigger: Spawn/split/move operations that rely on metadata currently marked `FIXME`.
- Workaround: No explicit workaround detected; feature work should avoid depending on these fields being complete without validation.

## Security Considerations

**Remote domain and update surfaces:**
- Risk: SSH, TLS credential, and update/download paths introduce trust boundaries that are sensitive to command execution, remote output, and downloaded content.
- Files: `mux/src/ssh.rs`, `kaku/src/cli/tls_creds.rs`, `kaku/src/update.rs`, `kaku-gui/src/download.rs`, `crates/wezterm-ssh/`
- Current mitigation: Typed Rust APIs, transport-specific crates, and local-only product positioning reduce some exposure.
- Recommendations: Keep new control-plane features local-first, avoid widening network-facing behavior, and keep new CLI contracts purely data-oriented.

## Performance Bottlenecks

**Render and window orchestration hot paths are dense:**
- Problem: Large rendering and term-window modules already manage caches, shaping, overlays, and notification reactions.
- Files: `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/termwindow/render/mod.rs`, `kaku-gui/src/shapecache.rs`, `kaku-gui/src/tabbar.rs`
- Cause: GUI logic fans into many state dimensions including selection, overlays, progress, panes, hyperlinks, shaping, and window chrome.
- Improvement path: Prefer incremental metadata rendering on existing invalidation paths instead of new broad redraw triggers or polling loops.

## Fragile Areas

**`TermWindow` integration surface:**
- Files: `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/termwindow/render/`, `kaku-gui/src/overlay/mod.rs`
- Why fragile: `TermWindow` is the main convergence point for input, rendering, alerts, overlays, and mux subscriptions.
- Safe modification: Add narrowly-scoped helper modules or reuse existing overlay/tabbar hooks before editing shared core flows.
- Test coverage: There are local tests in `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/termwindow/resize.rs`, and render submodules, but no full-stack GUI regression harness.

**Mux notification core:**
- Files: `mux/src/lib.rs`, `mux/src/tab.rs`, `mux/src/window.rs`
- Why fragile: Notifications drive many downstream UI updates, and changes can silently affect CLI/GUI sync.
- Safe modification: Extend `MuxNotification` additively, keep subscribers backward-compatible, and test workspace/pane/tab edge cases explicitly.
- Test coverage: Some pane/tab logic is tested in `mux/src/pane.rs`, `mux/src/pane_encoding.rs`, and `mux/src/tab.rs`, but notification fan-out behavior is not comprehensively isolated.

## Scaling Limits

**Per-pane buffering and in-memory state:**
- Current capacity: `mux/src/lib.rs` uses a reduced 256 KB pane buffer (`BUFSIZE`) to lower per-pane memory overhead.
- Limit: Large numbers of active panes or heavy output streams will still accumulate cost in pane buffers, render caches, and mux registries.
- Scaling path: Keep new metadata stores compact and keyed by existing pane/tab/workspace IDs; avoid duplicating pane output or history for control-plane features.

## Dependencies at Risk

**Large WezTerm-derived dependency surface:**
- Risk: The workspace includes many tightly-coupled internal crates and vendored/native deps, so broad architectural shifts would have large blast radius.
- Impact: New features that bypass `mux`, `tabbar`, `overlay`, or `termwindow` extension points can become expensive to maintain.
- Migration plan: Stay inside the existing crate/module topology and favor additive Kaku-native behavior over introducing new app layers.

## Missing Critical Features

**Notification store and unread navigation:**
- Problem: The canonical spec requires unread attention state and next/previous unread routing, but the current repo only exposes existing alerts/progress hooks.
- Blocks: Phase 1 acceptance criteria in `docs/KAKU_CONTROL_PLANE_SPEC.md`.

**Workspace status/progress/log store:**
- Problem: Workspaces exist, and panes already expose `Progress`, but there is no dedicated workspace metadata plane.
- Blocks: Phase 2 acceptance criteria in `docs/KAKU_CONTROL_PLANE_SPEC.md`.

**Task-center overlay and task-pane lifecycle controls:**
- Problem: Launcher/copy/quickselect overlays exist, but there is no task-centric overlay or remain-on-exit/rerun/pipe-pane lifecycle surface matching the spec.
- Blocks: Phase 3 and Phase 4 acceptance criteria in `docs/KAKU_CONTROL_PLANE_SPEC.md`.

## Test Coverage Gaps

**Control-plane extension points are not yet covered as a unified slice:**
- What's not tested: There is no current test suite for notification/unread routing, workspace metadata stores, or task-pane lifecycle because those features do not exist yet.
- Files: future work will likely touch `mux/src/lib.rs`, `mux/src/pane.rs`, `mux/src/tab.rs`, `kaku/src/cli/`, `kaku-gui/src/tabbar.rs`, and `kaku-gui/src/overlay/`
- Risk: New cross-layer state can regress old Kaku behavior if introduced without targeted CLI + mux + UI tests.
- Priority: High

**Large orchestrator modules rely heavily on local tests:**
- What's not tested: End-to-end behavior across `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/commands.rs`, and `mux/src/lib.rs` is only partially exercised by localized unit tests.
- Files: `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/commands.rs`, `mux/src/lib.rs`
- Risk: Feature work can accidentally break command routing, overlays, or notification handling without a single test clearly identifying the regression.
- Priority: High

---

*Concerns audit: 2026-03-26*
