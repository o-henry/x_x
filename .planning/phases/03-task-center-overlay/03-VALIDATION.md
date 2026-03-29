---
phase: 03
slug: task-center-overlay
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-27
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust inline unit tests + targeted cargo check |
| **Config file** | Existing workspace Cargo targets; no new test harness required |
| **Quick run command** | `cargo check --locked -p kaku-gui` |
| **Full phase command** | `cargo test --locked -p mux task_center -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --locked -p kaku-gui`
- **After every plan wave:** Run the targeted `mux` or `kaku-gui` task-center test slice for touched files
- **Before `$gsd-verify-work`:** Build `kaku-gui` and manually confirm overlay open/filter/action behavior
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 03-01-01 | 03-01 | 1 | TASK-01, TASK-02 | unit | `cargo test --locked -p mux task_center -- --nocapture` | ⬜ pending |
| 03-02-01 | 03-02 | 2 | TASK-01 | unit | `cargo test --locked -p kaku-gui task_center -- --nocapture` | ⬜ pending |
| 03-03-01 | 03-03 | 2 | TASK-01, TASK-02 | unit | `cargo test --locked -p kaku-gui task_center -- --nocapture` | ⬜ pending |
| 03-04-01 | 03-04 | 3 | TASK-03, TASK-04 | integration | `cargo check --locked -p kaku-gui` | ⬜ pending |
| 03-05-01 | 03-05 | 4 | TASK-05 | unit | `cargo test --locked -p kaku-gui task_center -- --nocapture` | ⬜ pending |
| 03-05-02 | 03-05 | 4 | TASK-01, TASK-02, TASK-03, TASK-04, TASK-05 | integration | `cargo check --locked -p kaku-gui && cargo test --locked -p mux task_center -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `mux` task-center snapshot tests for workspace/tab/pane/unread aggregation and failed/running classification
- [ ] `kaku-gui` task-center overlay tests for search/filter behavior and row action availability
- [ ] `kaku-gui` tests for single overlay entrypoint and action dispatch without widening beyond Phase 3

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Searchable Task Center overlay opens and feels Kaku-native | TASK-01, TASK-02 | Visual/interaction feel matters | Build `kaku-gui`, open Task Center, confirm search/filter behavior on real overlay rows |
| Selected entry can focus target and clear unread | TASK-03, TASK-04 | Window focus and unread UX are runtime-sensitive | Create unread work, open Task Center, trigger focus and clear actions, then confirm the target changes and unread count drops |
| Failed pane rerun is offered only when rerun metadata exists | TASK-05 | Consumer-side affordance depends on runtime metadata | Seed one failed pane with rerun metadata and one without, then confirm only the eligible row offers rerun |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or explicit Wave 0 expectations
- [x] Sampling continuity keeps feedback loops below 120 seconds
- [x] Manual checks are limited to overlay feel and runtime actions
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
