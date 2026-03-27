---
phase: 1
slug: notification-routing
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-26
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust inline/unit tests plus workspace `cargo` commands |
| **Config file** | `Makefile`, `.github/workflows/ci.yml` |
| **Quick run command** | `cargo check --locked -p mux -p kaku -p kaku-gui` |
| **Full suite command** | `cargo test --locked -p mux && cargo test --locked -p kaku && cargo test --locked -p kaku-gui` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --locked -p mux -p kaku -p kaku-gui`
- **After every plan wave:** Run `cargo test --locked -p mux && cargo test --locked -p kaku && cargo test --locked -p kaku-gui`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 1 | NOTF-01, NOTF-03 | unit | `cargo test --locked -p mux notification_store -- --nocapture` | ✅ inline | ⬜ pending |
| 1-01-02 | 01 | 1 | NOTF-01, NOTF-04, NOTF-05 | unit | `cargo test --locked -p mux notification_store -- --nocapture` | ✅ inline | ⬜ pending |
| 1-02-01 | 02 | 2 | NOTF-01, NOTF-02, NOTF-03 | unit | `cargo test --locked -p codec -- --nocapture` | ✅ inline | ⬜ pending |
| 1-02-02 | 02 | 2 | NOTF-01, NOTF-02, NOTF-03, NOTF-04, NOTF-05 | compile | `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl` | ✅ inline | ⬜ pending |
| 1-03-01 | 03 | 3 | NOTF-01, NOTF-02, NOTF-03 | unit | `cargo test --locked -p kaku notification_contracts -- --nocapture` | ✅ inline | ⬜ pending |
| 1-04-01 | 04 | 2 | NOTF-06 | compile | `cargo check --locked -p kaku-gui` | ✅ inline | ⬜ pending |
| 1-04-02 | 04 | 2 | NOTF-06 | unit | `cargo test --locked -p kaku-gui tabbar -- --nocapture` | ✅ inline | ⬜ pending |
| 1-05-01 | 05 | 4 | NOTF-04, NOTF-05 | unit | `cargo test --locked -p kaku notification_discovery_contracts -- --nocapture` | ✅ inline | ⬜ pending |
| 1-05-02 | 05 | 4 | NOTF-04, NOTF-05 | regression | `cargo fmt --all --check && cargo test --locked -p mux notification_store -- --nocapture && cargo test --locked -p mux tab::tab_splitting -- --exact --nocapture && cargo test --locked -p kaku notification_contracts -- --nocapture && cargo test --locked -p kaku notification_discovery_contracts -- --nocapture && cargo test --locked -p kaku existing_cli_regressions -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture` | ✅ inline | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Inline Test Creation Strategy

Phase 1 does not use a separate Wave 0 test-scaffolding plan. Each execution plan owns the tests or checks it needs:

- Plan `01-01` creates `mux/src/notification_store.rs` with inline store tests for creation, mutation, unread modes, and unread routing.
- Plan `01-03` creates the CLI contract tests inside the new notification command modules.
- Plan `01-04` adds tabbar unread-marker assertions in `kaku-gui/src/tabbar.rs`.
- Plan `01-05` adds jump/discovery JSON-contract tests and runs the explicit closeout regression slice for existing Kaku behavior.

Because tests are created inside the implementing plans, `wave_0_complete` is true and downstream plans depend only on the real implementation plans, not a synthetic prereq wave.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tabbar unread surfacing still feels like vanilla Kaku | NOTF-06 | Visual subtlety and overall shell feel are subjective beyond unit assertions | Run the GUI with a background pane that raises unread notifications, verify markers are visible but additive, and confirm no sidebar or redesign appears |
| CLI navigation preserves familiar pane/tab focus behavior | NOTF-04, NOTF-05 | Traversal ordering can be asserted in unit tests, but lived navigation feel still needs human confirmation | Create multiple panes across tabs, mark several unread, run next/previous unread commands, confirm focus movement matches normal Kaku navigation expectations, then spot-check existing `activate-pane`, `activate-tab`, `list`, and workspace switching flows still behave normally |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or inline test-creation coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] No synthetic Wave 0 dependency remains; inline plan-owned test creation covers missing test modules
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
