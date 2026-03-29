---
phase: 04
slug: task-pane-lifecycle
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` / `cargo check` |
| **Config file** | none — existing workspace test infrastructure |
| **Quick run command** | `cargo test --locked -p mux task_panes -- --nocapture` |
| **Full suite command** | `cargo test --locked -p mux task_panes -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --locked -p mux task_panes -- --nocapture`
- **After every plan wave:** Run the plan-specific verification command plus targeted `cargo check` for transport/GUI waves
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | LIFE-01 | unit | `cargo test --locked -p mux task_panes -- --nocapture` | ✅ | ⬜ pending |
| 04-01-02 | 01 | 1 | LIFE-05 | unit | `cargo test --locked -p mux task_panes -- --nocapture` | ✅ | ⬜ pending |
| 04-02-01 | 02 | 2 | LIFE-01, LIFE-02, LIFE-03 | integration | `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl && cargo test --locked -p codec -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture` | ✅ | ⬜ pending |
| 04-03-01 | 03 | 2 | LIFE-02, LIFE-05 | gui/integration | `cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 04-04-01 | 04 | 3 | LIFE-04 | unit/integration | `cargo test --locked -p mux task_panes -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture` | ✅ | ⬜ pending |
| 04-05-01 | 05 | 4 | LIFE-01, LIFE-02, LIFE-03, LIFE-04, LIFE-05 | regression | `cargo test --locked -p mux task_panes -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Remain-on-exit pane is still useful after the process exits | LIFE-01 | Requires real GUI pane exit behavior and visual persistence confirmation | Spawn a task pane configured for remain-on-exit, let the command exit, confirm the pane stays visible and the user can inspect it in Kaku. |
| Rerun/respawn action is offered only when durable rerun metadata exists | LIFE-02, LIFE-05 | Requires end-to-end runtime data and Task Center interaction | Create a failed pane with rerun metadata, open Task Center, verify rerun is shown and works; verify rows without metadata do not offer rerun. |
| Tee-to-file preserves normal pane output | LIFE-04 | Needs real pane output plus file side effects | Start pipe-pane/tee for a pane, produce output, confirm the pane still renders normally and the file receives the same output. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or existing infrastructure coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

