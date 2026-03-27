---
phase: 02
slug: workspace-metadata-plane
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-27
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust inline unit tests + Cargo test/nextest |
| **Config file** | `Makefile` defines standard commands; no separate test runner config found for this phase |
| **Quick run command** | `cargo check --locked -p mux -p kaku -p kaku-gui -p wezterm-mux-server-impl` |
| **Full suite command** | `make test` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --locked -p mux -p kaku -p kaku-gui -p wezterm-mux-server-impl`
- **After every plan wave:** Run targeted `cargo test` commands for the touched crates
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 02-01 | 1 | META-01, META-02, META-03, META-04 | unit | `cargo test -p mux workspace_state -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-01-02 | 02-01 | 1 | META-01, META-02, META-03, META-04 | unit | `cargo test -p mux workspace_rename -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02-02 | 2 | META-01, META-02, META-03, META-04 | unit | `cargo test -p codec workspace_metadata -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02-02 | 2 | META-01, META-02, META-03, META-04 | integration | `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl` | ✅ | ⬜ pending |
| 02-03-01 | 02-03 | 3 | META-01, META-02, META-03, META-04 | unit | `cargo test -p kaku workspace_metadata -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-04-01 | 02-04 | 2 | META-05 | unit | `cargo test -p kaku-gui workspace_metadata -- --nocapture` | ❌ W0 | ⬜ pending |
| 02-05-01 | 02-05 | 4 | META-01, META-02, META-03, META-04, META-05 | integration | `cargo check --locked -p mux -p kaku -p kaku-gui -p wezterm-mux-server-impl` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `mux/src/workspace_state.rs` tests for status/progress/log mutation, ordering, truncation, and workspace rename behavior
- [ ] `kaku/src/cli/{set_status,clear_status,list_status,set_progress,clear_progress,log,clear_log,list_log}.rs` request-shape and JSON contract tests
- [ ] `crates/codec/src/lib.rs` PDU round-trip tests for all new metadata request/response structs
- [ ] `kaku-gui/src/tabbar.rs` and/or `kaku-gui/src/termwindow/mod.rs` tests for metadata-driven title/status refresh triggers
- [ ] `cargo-nextest` local install if full `make test` becomes required during the phase

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Workspace status/progress visibly surface on existing Kaku runtime UI without feeling like a new shell | META-05 | The final judgment is user-visible and Kaku-feel-sensitive even when render tests exist | Build `kaku-gui`, set workspace status/progress through CLI, and visually confirm tabbar/right-status surfacing in the running GUI |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
