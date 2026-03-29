---
phase: 05
slug: hardening-and-compatibility
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-27
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for Phase 5 hardening and compatibility work.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` / `cargo check` plus targeted manual GUI compatibility verification |
| **Config file** | none — existing workspace test infrastructure |
| **Quick run command** | `cargo test --locked -p mux task_panes -- --nocapture` |
| **Full suite command** | `cargo test --locked -p mux task_panes -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture && cargo test --locked -p kaku compatibility_cli -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **Before trusting any later wave:** repair the currently red quick suite `cargo test --locked -p mux task_panes -- --nocapture`
- **After every task commit:** run `cargo test --locked -p mux task_panes -- --nocapture`
- **After every wave:** run that wave's targeted automated command plus any required compile checks
- **Before final verify-work:** the full suite above must be green and the manual compatibility checklist must be updated with real outcomes
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | COMP-04 | unit | `cargo test --locked -p mux task_panes -- --nocapture` | ✅ | ⬜ pending |
| 05-01-02 | 01 | 1 | COMP-02, COMP-04 | unit/compile | `cargo test --locked -p mux task_panes -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 05-02-01 | 02 | 2 | COMP-03, COMP-04 | integration | `cargo test --locked -p mux task_panes -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl` | ✅ | ⬜ pending |
| 05-02-02 | 02 | 2 | COMP-03 | unit | `cargo test --locked -p kaku lifecycle_contracts -- --nocapture` | ✅ | ⬜ pending |
| 05-03-01 | 03 | 3 | COMP-01 | integration | `cargo test --locked -p kaku compatibility_cli -- --nocapture` | MISSING — Wave 3 creates `kaku/tests/compatibility_cli.rs` with parser/help plus at least one behavior-level legacy-command dispatch slice | ⬜ pending |
| 05-03-02 | 03 | 3 | COMP-02, COMP-05 | gui/integration | `cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 05-04-01 | 04 | 4 | COMP-01, COMP-02, COMP-03, COMP-04, COMP-05 | manual/regression | `cargo test --locked -p mux task_panes -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture && cargo test --locked -p kaku compatibility_cli -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Phase 5 starts with one real precondition gap:

- `cargo test --locked -p mux task_panes -- --nocapture` is currently red because `task_panes::tests::rerun_metadata_from_command_builder_falls_back_to_joined_argv` fails.
- Until `05-01-01` repairs that quick suite, do not treat the quick command as a trustworthy per-task safety net.
- The manual/UAT recipe for the Phase 4 blocker must stay focused on the real failure chain: failing pane spawn -> bundled config behavior -> GUI survivability -> `list-task-panes` inspection.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Failing pane spawn no longer triggers the old `kaku.lua` recursion -> GUI socket EOF chain | COMP-02, COMP-04 | Requires real GUI runtime, bundled config evaluation, and failure-path observation | Start the local GUI build, spawn a minimal failing pane using the same shell fixture shape from the Phase 4 blocker (`/bin/sh -lc 'echo PHASE4_FAILING_TASK; false'`), confirm the GUI stays responsive, and then run `./target/debug/kaku cli list-task-panes --format json` successfully. |
| Baseline pane/tab/workspace behavior is preserved after lifecycle operations | COMP-01, COMP-02, COMP-05 | Requires real focus, split, navigation, and workspace switching behavior in the GUI | In one session, verify normal pane spawn, split, focus movement, tab switching, and workspace switching before and after using `set-remain-on-exit`, `silence-watchdog`, `pipe-pane`, and a rerun/respawn flow. Confirm the app still feels like Kaku and not a new shell. |
| Task Center remains an additive consumer overlay | COMP-05 | Needs real overlay interaction and qualitative compatibility check | Open Task Center after lifecycle changes, confirm it still focuses targets and refreshes from mux-backed state, and verify it does not replace baseline pane/tab navigation or introduce a broader dashboard shell. |

---

## Fast Manual Command Smoke Anchors

Use these during the explicit Wave 4 manual run to confirm baseline command behavior before deeper UI checks:

```bash
./target/debug/kaku cli list --format json
./target/debug/kaku cli spawn --cwd "$PWD" -- /bin/sh -lc 'printf phase5-smoke'
./target/debug/kaku cli split-pane --right --percent 50
./target/debug/kaku cli send-text --pane-id <pane-id> 'phase5 smoke'
./target/debug/kaku cli get-text --pane-id <pane-id> --start-line -20
./target/debug/kaku cli set-tab-title 'phase5-smoke'
./target/debug/kaku cli list-task-panes --format json
```

---

## Validation Sign-Off

- [x] Every planned task has an automated verification command
- [x] Sampling continuity is maintained with the mux quick suite after Wave 1 repairs it
- [x] The Phase 4 UAT blocker is explicitly represented in the validation strategy
- [x] Manual compatibility checks cover baseline Kaku behavior as well as new lifecycle paths
- [x] `nyquist_compliant: true` is set in frontmatter
- [x] No watch-mode or open-ended verification commands are required

**Approval:** pending
