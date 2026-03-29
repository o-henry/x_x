---
phase: 06
slug: operator-ui-surfaces
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for the native operator-UI work.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` / `cargo check` plus targeted manual native-GUI UAT |
| **Config file** | none — existing workspace test infrastructure |
| **Quick run command** | `cargo test --locked -p kaku-gui task_center -- --nocapture` |
| **Full suite command** | `cargo fmt --all --check && cargo test --locked -p mux task_center -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo test --locked -p kaku show_task_center_has_default_shortcut -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **Before trusting later waves:** establish a green Task Center quick suite with `cargo test --locked -p kaku-gui task_center -- --nocapture`
- **After every task commit:** run the quick suite above
- **After every wave:** run that wave's targeted command plus any compile checks
- **Before final verify-work:** the full suite above must be green and the manual GUI UAT checklist must be updated with real outcomes
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | UI-03, UI-04, UI-05 | integration/compile | `cargo fmt --all --check && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` | ✅ | ⬜ pending |
| 06-01-02 | 01 | 1 | UI-03, UI-04, UI-05 | unit/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui workspace_metadata -- --nocapture && cargo check --locked -p kaku-gui` | ✅ existing file — Wave 1 extends `kaku-gui/src/termwindow/mod.rs` coverage | ⬜ pending |
| 06-02-01 | 02 | 2 | UI-02, UI-03, UI-04, UI-05 | unit | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture` | ✅ | ⬜ pending |
| 06-02-02 | 02 | 2 | UI-02, UI-03, UI-04, UI-05 | unit/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p kaku-gui` | ✅ existing file — Wave 2 extends `kaku-gui/src/overlay/task_center.rs` coverage | ⬜ pending |
| 06-03-01 | 03 | 2 | UI-01, UI-02, UI-05 | unit | `cargo fmt --all --check && cargo test --locked -p kaku-gui tabbar -- --nocapture` | ✅ | ⬜ pending |
| 06-03-02 | 03 | 2 | UI-01, UI-02, UI-05 | unit/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 06-04-01 | 04 | 3 | UI-01, UI-02, UI-03, UI-04, UI-05 | regression/manual | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 06-04-02 | 04 | 3 | UI-01, UI-05 | docs | `! rg -n 'CLI-only' docs/KAKU_CONTROL_PLANE_SPEC.md PLANS.md .planning/phases/06-operator-ui-surfaces/06-UAT.md .planning/phases/06-operator-ui-surfaces/06-LIMITATIONS.md && ! rg -n 'browser-like' docs/KAKU_CONTROL_PLANE_SPEC.md PLANS.md .planning/phases/06-operator-ui-surfaces/06-UAT.md .planning/phases/06-operator-ui-surfaces/06-LIMITATIONS.md && ! rg -n 'sidebar' docs/KAKU_CONTROL_PLANE_SPEC.md PLANS.md .planning/phases/06-operator-ui-surfaces/06-UAT.md .planning/phases/06-operator-ui-surfaces/06-LIMITATIONS.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Phase 06 starts with three validation obligations that are now explicitly planned and assigned to execution waves:

- `kaku-gui/src/overlay/task_center.rs` needs explicit tests for mouse row selection, wheel scrolling, and inline-action dispatch before the new operator path is considered stable.
- `kaku-gui/src/termwindow/mod.rs` needs helper-level tests for metadata mutation request construction, scoped-open behavior, and remain-on-exit style toggle routing.
- Manual UAT must be written down before phase closeout because there is no GUI integration harness for mouse clicks, prompt/confirm flows, or tabbar marker entry today.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Task Center supports the common operator path with keyboard and mouse parity | UI-02, UI-03, UI-05 | Requires real overlay interaction, click timing, and visual selection state | Open Task Center from native UI, verify typing filters, Up/Down selection, single-click selection, double-click primary action, Enter focus, Esc close, and visible inline actions for clear unread, rerun/respawn, and remain-on-exit style toggle where applicable. |
| Workspace metadata can be edited from native UI prompts without CLI recall | UI-04, UI-05 | Requires real prompt/confirm overlay behavior and visible title/tab refresh | From Task Center, trigger edit status and edit progress for a workspace row, confirm prompt prefills when appropriate, submit changes, clear them through the UI path, and verify the tabbar/title state refreshes in place. |
| Tabbar operator markers stay additive and open scoped Task Center views | UI-01, UI-02, UI-05 | Requires native hover/click behavior and qualitative Kaku-feel judgment | With actionable operator state present, verify the tabbar marker is compact, opens the expected scoped Task Center view on click, and does not disrupt normal tab activation, drag, close, or new-tab behavior. |

---

## Fast Manual Command Smoke Anchors

Use these during Wave 3 UAT so the GUI checks have reproducible setup:

```bash
./target/debug/kaku-gui start --always-new-process
./target/debug/kaku cli notify --title "phase6-unread" --body "operator-ui" --kind task
./target/debug/kaku cli set-status "phase6 status"
./target/debug/kaku cli set-progress 0.42
./target/debug/kaku cli list-task-panes --format json
```

---

## Validation Sign-Off

- [x] Every planned task has an automated verification command
- [x] Sampling continuity is maintained with a fast `task_center` quick suite
- [x] Wave 0 gaps are explicitly listed for mouse, helper, and manual-UAT coverage
- [x] Manual validation covers mouse parity, metadata editing, and additive tabbar discoverability
- [x] `nyquist_compliant: true` is set in frontmatter
- [x] No watch-mode or open-ended verification commands are required

**Approval:** pending
