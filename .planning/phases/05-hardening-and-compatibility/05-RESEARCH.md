# Phase 05: Hardening and Compatibility - Research

**Researched:** 2026-03-27
**Domain:** Kaku control-plane regression hardening, CLI contract stability, compatibility verification, and Phase 4 follow-through
**Confidence:** HIGH

## Summary

Phase 5 should be planned as a narrow hardening pass over the control-plane surfaces already added in Phases 1 through 4, not as another feature phase. The repo already has the right ownership model for this: mux owns lifecycle state, the CLI exposes machine-readable contracts, and the GUI consumes snapshots instead of becoming the source of truth. The remaining work is about making those seams safe under failure, preserving existing Kaku pane/tab/workspace behavior, and closing the documentation and verification gaps honestly.

The strongest planning signal is not just the documented Phase 4 limitations. It is the concrete UAT blocker from Phase 4: spawning a failing pane triggered repeated recursion in [`assets/macos/Kaku.app/Contents/Resources/kaku.lua:166`](/Users/henry/Documents/code/vibe/hybrid/x_x/assets/macos/Kaku.app/Contents/Resources/kaku.lua#L166) through [`assets/macos/Kaku.app/Contents/Resources/kaku.lua:179`](/Users/henry/Documents/code/vibe/hybrid/x_x/assets/macos/Kaku.app/Contents/Resources/kaku.lua#L179), followed by GUI socket EOF that prevented `list-task-panes` verification. Even if the exact root cause is not yet proven, that sequence tells us something important: the highest-risk path is not steady-state lifecycle state, but failure-path interaction between pane spawn, config evaluation, GUI process health, and mux-backed lifecycle inspection. Phase 5 planning should therefore start with reproducer-driven hardening around spawn/failure behavior and only then broaden into regression sign-off.

There is also one repo-local validation gap that should be treated as a Phase 5 entry condition: the Phase 4 quick command from [`04-VALIDATION.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/04-task-pane-lifecycle/04-VALIDATION.md) is not currently green. `cargo test --locked -p mux task_panes -- --nocapture` fails on the `rerun_metadata_from_command_builder_falls_back_to_joined_argv` assertion in [`mux/src/task_panes.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/task_panes.rs). That does not block research, but it means the first Phase 5 wave should stabilize the lifecycle-test baseline before treating it as a safety net.

**Primary recommendation:** Plan Phase 5 as four additive waves: failure-path triage and baseline repair, contract hardening, compatibility regression coverage, then docs/limitations closeout.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-01 | Developer can continue using existing Kaku CLI pane-management commands without behavior regressions. | Add explicit regression coverage for `list`, `spawn`, `split-pane`, `send-text`, `get-text`, `set-tab-title`, and lifecycle commands against the same mux/gui runtime. |
| COMP-02 | Developer can continue using existing pane/tab creation, splitting, navigation, focus, and workspace switching without behavior regressions. | Use compatibility-focused manual and targeted Rust checks around pane spawn/focus/workspace switching, especially after failure-path lifecycle operations. |
| COMP-03 | Developer can rely on stable machine-readable contracts for each new CLI surface added by this control plane. | Harden lifecycle JSON payloads, naming, rerun metadata normalization, and cross-command contract docs. |
| COMP-04 | Developer can verify each phase with targeted tests or checks before the phase is considered done. | Phase 5 should repair the current failing mux slice, define a short quick suite, and map commands to requirement coverage. |
| COMP-05 | Developer can use the fork without it feeling like a different app shell or a cmux clone. | Keep work inside mux, CLI, tabbar, overlay, termwindow, and existing docs. Do not add new shell concepts or broaden UX scope. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Read `AGENTS.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, and `PLANS.md` first.
- GSD is only for planning/orchestration/state tracking; product scope is defined by the spec and plans.
- Rust-only.
- No Swift or Xcode.
- No React, Electron, Tauri, or WebView.
- No copied code from cmux or tmux.
- Preserve Kaku feel and architecture.
- Prefer extending `kaku/src/cli`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay`, `kaku-gui/src/termwindow`, and `mux/src`.
- Do not start with a sidebar, browser, PR UI, socket daemon, or full tmux detached semantics.

## Goal Recap

Phase 5 exists to make the shipped control-plane slice safe to keep using as Kaku, not to add another major capability. Ground truth from [`docs/KAKU_CONTROL_PLANE_SPEC.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/docs/KAKU_CONTROL_PLANE_SPEC.md#L280), [`PLANS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/PLANS.md), and [`ROADMAP.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/ROADMAP.md#L81) is consistent:

- lock down tests
- lock down docs
- clean up naming and contract consistency
- check event consistency
- run regression and performance sanity passes
- preserve existing pane/tab/workspace behavior as a compatibility surface

## Repo-Grounded Findings

### 1. Phase 5 should harden existing surfaces, not create new ones

The canonical spec and project docs repeatedly constrain the work to existing Kaku extension points:

- [`docs/KAKU_CONTROL_PLANE_SPEC.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/docs/KAKU_CONTROL_PLANE_SPEC.md)
- [`PLANS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/PLANS.md)
- [`CLAUDE.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/CLAUDE.md)
- [`AGENTS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/AGENTS.md)

The correct planning stance is therefore additive hardening in:

- `mux/src`
- `kaku/src/cli`
- `crates/codec/src/lib.rs`
- `crates/wezterm-client/src/client.rs`
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
- `kaku-gui/src/termwindow/mod.rs`
- `kaku-gui/src/overlay/task_center.rs`

Confidence: HIGH. This is locked by repo instructions, not inference.

### 2. The mux lifecycle store is the right hardening center of gravity

Phase 4 already landed a real mux-owned lifecycle model:

- [`mux/src/lib.rs:1616`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs#L1616) records exit state and emits `MuxNotification::TaskPaneLifecycleChanged`.
- [`mux/src/lib.rs:1663`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs#L1663) persists remain-on-exit.
- [`mux/src/lib.rs:1699`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs#L1699) persists silence intent.
- [`mux/src/lib.rs:1724`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs#L1724) persists tee state.
- [`mux/src/task_panes.rs:12`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/task_panes.rs#L12) defines the durable task-pane record.

This means Phase 5 should not move lifecycle truth elsewhere. It should harden:

- record retention and pruning rules
- rerun metadata normalization
- notification/event consistency around lifecycle changes
- failure-path invariants when panes die, respawn, or disappear

Confidence: HIGH. This is directly visible in source and Phase 4 changed-files documentation.

### 3. The current contract surface is partly stable, but not yet trustworthy enough for final hardening sign-off

The good news:

- lifecycle CLI contract tests in `kaku` pass:
  - `cargo test --locked -p kaku lifecycle_contracts -- --nocapture`
- the CLI already follows the repo’s one-command-per-file pattern:
  - [`kaku/src/cli/list_task_panes.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/list_task_panes.rs)
  - [`kaku/src/cli/set_remain_on_exit.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/set_remain_on_exit.rs)
  - [`kaku/src/cli/rerun_pane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/rerun_pane.rs)
  - [`kaku/src/cli/respawn_pane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/respawn_pane.rs)
  - [`kaku/src/cli/silence_watchdog.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/silence_watchdog.rs)
  - [`kaku/src/cli/pipe_pane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/pipe_pane.rs)

The problem:

- the Phase 4 quick suite is red:
  - `cargo test --locked -p mux task_panes -- --nocapture`
- the failing assertion is about rerun-command fallback quoting in [`mux/src/task_panes.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/task_panes.rs), which is directly relevant to `rerun-pane` / `respawn-pane` behavior.

Planning implication: Phase 5 should start by deciding which rerun-command representation is canonical and updating both implementation and tests to match it. Otherwise every later compatibility claim rests on a shaky command-reconstruction rule.

Confidence: HIGH. Verified by local test runs.

### 4. The UAT blocker should shape the entire Phase 5 plan even before root cause is proven

Phase 4 UAT records one blocking failure:

- spawning a failing pane triggered repeated recursion in `kaku.lua` `is_low_resolution_screen()`
- then the GUI socket hit EOF
- then `list-task-panes` verification could not complete

Evidence:

- [`04-UAT.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/04-task-pane-lifecycle/04-UAT.md)
- [`assets/macos/Kaku.app/Contents/Resources/kaku.lua:166`](/Users/henry/Documents/code/vibe/hybrid/x_x/assets/macos/Kaku.app/Contents/Resources/kaku.lua#L166) through [`assets/macos/Kaku.app/Contents/Resources/kaku.lua:179`](/Users/henry/Documents/code/vibe/hybrid/x_x/assets/macos/Kaku.app/Contents/Resources/kaku.lua#L179)

Why this matters for Phase 5 planning:

- It happened on the exact path needed to validate `LIFE-02` and `LIFE-05`, so final lifecycle compatibility is still unproven.
- The symptom chain crosses boundaries: GUI config/runtime, spawn behavior, pane failure fixture generation, and mux inspection.
- Even if the Lua recursion is not ultimately caused by Phase 4 code, Phase 5 is the correct place to harden the fork against this class of failure because `COMP-01`, `COMP-02`, and `COMP-04` all require the baseline Kaku experience and the verification workflow to remain usable.
- A GUI socket EOF after spawn/failure means the fork currently lacks a stable failure-path verification lane. Hardening should therefore include “can inspect lifecycle state after a failing spawn without destabilizing the GUI process” as a top-level planning target.

The right planning approach is not “fix Lua because UAT mentioned Lua” in isolation. It is “add a first wave that reproduces and contains spawn/failure-path instability, then verify the lifecycle surfaces on top of that stabilized baseline.”

Confidence: MEDIUM-HIGH. The exact root cause is unproven, but the planning consequence is strongly supported by the failure sequence.

### 5. The remaining Phase 4 limitations map cleanly into Phase 5 hardening tasks

The shipped limitations are already honest and actionable:

- [`04-LIMITATIONS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/04-task-pane-lifecycle/04-LIMITATIONS.md)
- [`docs/KAKU_CONTROL_PLANE_SPEC.md:274`](/Users/henry/Documents/code/vibe/hybrid/x_x/docs/KAKU_CONTROL_PLANE_SPEC.md#L274)

They cluster into four Phase 5 work items:

- watchdog policy semantics
- rerun metadata seeding and normalization
- stale task-pane cleanup / retention
- broader lifecycle event consistency beyond the narrow local happy path

Confidence: HIGH. These were explicitly deferred into Phase 5 by prior docs.

## Standard Stack

### Core

| Library / Crate | Version | Purpose | Why Standard Here |
|-----------------|---------|---------|-------------------|
| `kaku` | 0.8.0 | CLI and app entrypoints | Existing product surface; compatibility must be preserved rather than replaced. |
| `mux` | 0.1.0 | Authoritative control-plane and lifecycle state | Already owns notifications, workspace metadata, task panes, and snapshot assembly. |
| `codec` | 0.1.0 | Typed machine-readable RPC contracts | Existing stable transport seam for CLI contract hardening. |
| `wezterm-client` | 0.1.0 | Client RPC wrappers | Existing CLI transport path; needed for contract consistency checks. |
| `wezterm-mux-server-impl` | 0.1.0 | Server-side dispatch into mux | Existing narrow seam for lifecycle request handling. |

### Supporting

| Library / Crate | Version | Purpose | When to Use |
|-----------------|---------|---------|-------------|
| `kaku-gui` | workspace crate | Existing GUI/Task Center consumer | For compatibility checks, rerun affordances, overlay refresh, and failure-path validation. |
| bundled `kaku.lua` config | repo-local | GUI startup/runtime behavior | For reproduction and containment of the UAT recursion/EOF failure path. |

**Version verification:** repo-local workspace versions verified with:

```bash
cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | [.name,.version] | @tsv' | rg '^(kaku|mux|wezterm-client|wezterm-mux-server-impl|codec|config)\t'
```

## Architecture Patterns

### Recommended Project Structure

```text
mux/src/                   # authoritative lifecycle state, retention, events, and invariants
kaku/src/cli/              # explicit machine-readable commands and JSON/table contract tests
crates/codec/src/lib.rs    # request/response schema stability
crates/wezterm-client/     # typed client wrappers
crates/wezterm-mux-server-impl/  # dispatch and mux bridge
kaku-gui/src/termwindow/   # consumer refresh, focus, rerun wiring
kaku-gui/src/overlay/      # Task Center presentation only
.planning/phases/05-hardening-and-compatibility/  # validation/docs closeout artifacts
```

### Pattern 1: Mux-first hardening

**What:** Preserve mux as the source of truth and harden invariants there before changing GUI behavior.

**When to use:** Any Phase 5 task involving lifecycle state, event timing, stale record cleanup, rerun availability, or compatibility assertions.

**Example:**

```rust
// Source: mux/src/lib.rs
let record = self.task_panes.write().record_exit(TaskPaneExitRecord {
    pane_id,
    workspace,
    window_id,
    tab_id,
    remain_on_exit,
    is_failed,
    exit_behavior,
    current_working_dir,
    rerun,
});
self.notify(MuxNotification::TaskPaneLifecycleChanged(pane_id));
```

Planning implication: Phase 5 should extend the invariants around this pattern, not move ownership into `termwindow` or Task Center.

### Pattern 2: CLI contract hardening through request-shape and JSON-shape tests

**What:** Each CLI command keeps its own parser/request tests and output-shape tests in the same command file.

**When to use:** Contract cleanup, naming normalization, or JSON-field stability work.

**Example:**

```rust
// Source: kaku/src/cli/list_task_panes.rs
#[test]
fn lifecycle_contracts_list_task_panes_json_shape() {
    let json = ListTaskPanesCommand::render_json(&[sample_task_pane()]).expect("json");
    assert!(json.contains("\"pane_id\": 4"));
    assert!(json.contains("\"remain_on_exit\": true"));
    assert!(json.contains("\"rerun_available\": true"));
    assert!(json.contains("\"tee_path\": \"/tmp/task.log\""));
}
```

Planning implication: Phase 5 should expand this pattern to compatibility commands and normalize wording/fields where current behavior is ambiguous.

### Pattern 3: GUI remains a consumer, not the lifecycle authority

**What:** `TermWindow` reads mux task-pane records and live panes, then performs UI actions like rerun/focus without owning state.

**When to use:** Any hardening around Task Center behavior, cache refreshes, or GUI compatibility after lifecycle actions.

**Example:**

```rust
// Source: kaku-gui/src/termwindow/mod.rs
let mux = Mux::get();
let record = mux.task_pane_record(pane_id);
let pane = mux.get_pane(pane_id);
```

Planning implication: fix stale reads and fallback rules here, but keep state truth in mux.

### Anti-Patterns to Avoid

- **Do not add a new lifecycle daemon or detached task server:** This violates the repo’s Kaku-native constraint and widens scope beyond Phase 5.
- **Do not fix the UAT blocker by bypassing existing Kaku spawn/config paths wholesale:** That would hide the compatibility problem instead of hardening it.
- **Do not make Task Center or GUI caches authoritative:** That recreates the same split-brain risk earlier phases were designed to avoid.
- **Do not broaden Phase 5 into visual redesign or UX experiments:** `COMP-05` is about preserving feel, not inventing a new shell.

## Don’t Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Lifecycle truth | GUI-local dead-pane cache | existing mux `TaskPaneStore` | One source of truth already exists and is wired into notifications. |
| Contract verification | ad hoc manual JSON inspection | existing per-command Rust tests in `kaku/src/cli` | Keeps machine-readable contracts regression-safe. |
| Compatibility verification | one large end-to-end script only | targeted command slices plus small manual GUI checks | The highest-risk failure is cross-boundary; narrow checks isolate breakage faster. |
| Failure repro | speculative redesign of spawn semantics | minimal failing-pane fixture on current spawn path | Needed to prove whether Phase 4 or bundled runtime behavior is actually unstable. |

**Key insight:** Phase 5 is about sharpening the existing control-plane seams, not replacing them with “cleaner” parallel infrastructure.

## Common Pitfalls

### Pitfall 1: Treating the Phase 4 blocker as “just a Lua config bug”

**What goes wrong:** Planning isolates the issue to `kaku.lua` and ignores its effect on lifecycle verification and GUI socket health.

**Why it happens:** The visible stack overflow is in Lua, so it is tempting to route all hardening into config only.

**How to avoid:** Make the first Phase 5 wave explicitly reproduce the full chain: spawn failing pane, observe config/runtime behavior, confirm GUI survives, then inspect task-pane state.

**Warning signs:** A proposed plan mentions changing `kaku.lua` but does not mention `list-task-panes`, Task Center, or GUI socket stability.

### Pitfall 2: Using a red quick suite as if it were a safety net

**What goes wrong:** Later waves assume `cargo test --locked -p mux task_panes -- --nocapture` is trustworthy even though it currently fails.

**Why it happens:** The command is documented in Phase 4 validation, so it looks authoritative.

**How to avoid:** Make “repair or intentionally rewrite the failing rerun quoting expectation” an explicit first task.

**Warning signs:** Plans reuse the existing quick suite unchanged without mentioning the current failure.

### Pitfall 3: Hardening only new commands and forgetting baseline Kaku commands

**What goes wrong:** `COMP-01` and `COMP-02` remain unproven because the plan tests only `list-task-panes`, `rerun-pane`, and related lifecycle commands.

**Why it happens:** The new code is easier to enumerate than existing Kaku compatibility surfaces.

**How to avoid:** Add a compatibility matrix covering `list`, `spawn`, `split-pane`, `send-text`, `get-text`, `set-tab-title`, focus, pane switching, and workspace switching.

**Warning signs:** A validation section contains only control-plane commands.

### Pitfall 4: Letting stale task-pane records linger without a defined policy

**What goes wrong:** Task Center and CLI output become noisy or misleading after repeated reruns and dead panes.

**Why it happens:** Phase 4 intentionally deferred cleanup semantics.

**How to avoid:** Define explicit prune/retention rules in mux and document which records are removed automatically versus retained for inspection.

**Warning signs:** `list-task-panes` grows indefinitely in long-lived sessions or after repeated failing fixtures.

## Recommended Plan Shape

### Wave 1: Baseline repair and blocker triage

**Objective:** Make the verification baseline honest and reproduce the spawn/failure instability deterministically.

Tasks:

- repair or normalize the failing `mux task_panes` rerun quoting expectation
- build a minimal failing-pane fixture that uses the existing spawn path
- verify whether the current bundled `kaku.lua` guard actually prevents recursion during that fixture path
- capture whether GUI socket EOF still occurs after a failing spawn

Target files:

- `mux/src/task_panes.rs`
- `mux/src/lib.rs` if rerun fallback semantics need implementation adjustment
- `assets/macos/Kaku.app/Contents/Resources/kaku.lua` only if the reproducer proves the guard is still insufficient
- `.planning/phases/05-hardening-and-compatibility/*` docs as evidence accumulates

### Wave 2: Contract and event hardening

**Objective:** Make lifecycle contracts and notifications consistent enough for downstream consumers.

Tasks:

- normalize rerun/respawn naming or result semantics if current behavior is ambiguous
- harden `TaskPaneLifecycleChanged` and related refresh behavior for dead/live transitions
- define stale-record cleanup and retention rules
- document exact contract guarantees for lifecycle commands and their failure modes

Target files:

- `mux/src/lib.rs`
- `mux/src/task_panes.rs`
- `crates/codec/src/lib.rs`
- `crates/wezterm-client/src/client.rs`
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
- `kaku/src/cli/*.rs`

### Wave 3: Compatibility regression pass

**Objective:** Prove the control plane did not break baseline Kaku behavior.

Tasks:

- add targeted command-level regression checks for existing pane-management CLI commands
- add focused GUI/manual verification for pane spawn, split, focus, tab navigation, and workspace switching before and after lifecycle operations
- confirm Task Center still behaves as a consumer overlay and does not widen scope

Target files:

- `kaku/src/cli/*.rs`
- `kaku-gui/src/termwindow/mod.rs`
- `kaku-gui/src/overlay/task_center.rs`
- validation docs under `.planning/phases/05-hardening-and-compatibility/`

### Wave 4: Docs closeout and honest limitations

**Objective:** Leave the fork auditable and ready for final verification.

Tasks:

- update `docs/KAKU_CONTROL_PLANE_SPEC.md`
- update `PLANS.md`
- create changed-files, validation, and limitations artifacts for Phase 5
- record any residual failure-path limits honestly if not fully eliminated

## State of the Art

| Earlier Phase Shape | Current Hardening Need | Why It Changed | Impact |
|---------------------|------------------------|----------------|--------|
| Feature-first implementation waves | failure-path-first hardening wave | Phase 4 UAT exposed spawn/config/GUI instability on the exact lifecycle verification path | Hardening must begin with reproducibility and baseline repair. |
| Limited-mode task failure heuristics | durable mux-owned task-pane records | Phase 4 shipped lifecycle persistence | Phase 5 can focus on semantics and retention instead of inventing storage. |
| Command-shape tests only | command-shape plus compatibility verification | `COMP-01` and `COMP-02` require old behavior preservation, not just new command validity | Validation must include baseline Kaku surfaces. |

**Deprecated/outdated for planning:**

- Treating Phase 5 as “just docs and tests” is outdated. The UAT blocker shows it also needs cross-boundary runtime hardening.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Rust tests/checks and contract verification | ✓ | 1.93.0 | — |
| `rustc` | Build/test baseline | ✓ | 1.93.0 | — |
| `jq` | local metadata/version inspection helpers | ✓ | 1.7.1 | optional shell parsing |
| `rg` | fast repo search during execution | ✓ | 15.1.0 | `grep` |
| bundled `kaku.lua` in app resources | reproducing GUI/config blocker path | ✓ | repo-local | none |

**Missing dependencies with no fallback:**

- None identified for planning research. Manual GUI verification still depends on a working local GUI runtime, but the repo already contains the bundled resources needed for the documented blocker path.

**Missing dependencies with fallback:**

- None.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` / `cargo check` plus targeted manual GUI verification |
| Config file | none — existing workspace test infrastructure |
| Quick run command | `cargo test --locked -p mux task_panes -- --nocapture` |
| Full suite command | `cargo test --locked -p mux task_panes -- --nocapture && cargo test --locked -p kaku lifecycle_contracts -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-01 | Existing pane-management CLI commands still behave | unit/integration | `cargo test --locked -p kaku lifecycle_contracts -- --nocapture` plus targeted command-specific checks to be added | ⚠️ Partial |
| COMP-02 | Existing pane/tab/workspace interactions still work | manual + targeted checks | `cargo check --locked -p kaku-gui` plus manual GUI flow verification | ⚠️ Partial |
| COMP-03 | New CLI surfaces keep stable contracts | unit | `cargo test --locked -p kaku lifecycle_contracts -- --nocapture` | ✅ |
| COMP-04 | Each phase has targeted verifications | validation architecture | quick suite plus phase full suite recorded in Phase 5 validation doc | ⚠️ Needs update |
| COMP-05 | Fork still feels like Kaku | manual compatibility review | manual UAT against existing tabbar/overlay/termwindow behavior | ⚠️ Manual-only |

### Targeted Validation Commands for Phase 5

Use these as the plan’s concrete verification anchors:

```bash
# Wave 1 baseline repair
cargo test --locked -p mux task_panes -- --nocapture

# CLI contract stability
cargo test --locked -p kaku lifecycle_contracts -- --nocapture

# Transport and GUI compile safety
cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui

# Optional narrower snapshot regressions already present in repo
cargo test --locked -p kaku-gui task_center -- --nocapture
```

Manual compatibility checks the Phase 5 plan should preserve:

```bash
./target/debug/kaku cli list --format json
./target/debug/kaku cli split-pane --help
./target/debug/kaku cli spawn --help
./target/debug/kaku cli send-text --help
./target/debug/kaku cli get-text --help
./target/debug/kaku cli set-tab-title --help
./target/debug/kaku cli list-task-panes --format json
```

Reason for including `--help` on baseline commands in planning: these are fast contract-smoke anchors for command survival and argument shape before deeper runtime UAT.

### Sampling Rate

- **Per task commit:** `cargo test --locked -p mux task_panes -- --nocapture`
- **Per wave merge:** `cargo test --locked -p kaku lifecycle_contracts -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui`
- **Phase gate:** all automated commands above green plus manual spawn/failure-path UAT completed without Lua recursion or GUI socket EOF

### Wave 0 Gaps

- [ ] Repair the currently failing `mux task_panes` quick suite before treating it as a safety net.
- [ ] Add explicit compatibility checks for legacy pane-management CLI commands, not just lifecycle commands.
- [ ] Add a documented manual repro/verify recipe for the failing-pane spawn path from Phase 4 UAT.

## Risks and Anti-Patterns

- **Risk: chasing the wrong root cause too early.** The planning target should be the reproducible failure chain, not a guessed single-line fix.
- **Risk: widening scope into new architecture.** The blocker may tempt a daemon or deeper shell-model rewrite; that would violate the roadmap and repo constraints.
- **Risk: overfitting to happy-path CLI tests.** The current red mux test and blocked UAT already show happy-path coverage is not enough.
- **Risk: undocumented naming drift.** `rerun-pane` and `respawn-pane` are close enough semantically that operator meaning can blur unless documented and tested clearly.

## Open Questions

1. **What is the canonical rerun fallback string shape?**
   - What we know: the current mux test expects one quoting style, while the implementation produces another.
   - What's unclear: whether the implementation or the test reflects intended operator-facing semantics.
   - Recommendation: resolve this in Wave 1 and treat the result as contract truth.

2. **Is the Phase 4 UAT blocker caused by current control-plane code, bundled config behavior, or a broader spawn/reload interaction?**
   - What we know: a failing-pane spawn triggered recursion in `kaku.lua` and then GUI socket EOF.
   - What's unclear: whether the lifecycle feature path introduced the trigger or only exposed an existing Kaku-local instability.
   - Recommendation: reproduce with the narrowest possible failing spawn fixture before making architectural changes.

3. **What should the stale task-pane retention policy be?**
   - What we know: Phase 4 explicitly deferred cleanup semantics.
   - What's unclear: how long dead/failed records should persist and when automatic pruning is safe.
   - Recommendation: define a documented mux-side policy in Wave 2 and verify it through both CLI and Task Center views.

4. **How far should lifecycle event consistency go in Phase 5?**
   - What we know: local GUI consumers refresh, but broader lifecycle fan-out was intentionally conservative.
   - What's unclear: whether Phase 5 needs only local consistency or also stronger multi-client guarantees.
   - Recommendation: keep scope to local compatibility unless a specific local regression proves stronger fan-out is required.

## Sources

### Primary (HIGH confidence)

- [`docs/KAKU_CONTROL_PLANE_SPEC.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/docs/KAKU_CONTROL_PLANE_SPEC.md) - canonical Phase 4 boundaries and Phase 5 hardening scope
- [`PLANS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/PLANS.md) - file-level extension points and lifecycle/hardening progress
- [`CLAUDE.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/CLAUDE.md) - project constraints
- [`AGENTS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/AGENTS.md) - local fork rules
- [`04-UAT.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/04-task-pane-lifecycle/04-UAT.md) - blocker evidence
- [`04-LIMITATIONS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/04-task-pane-lifecycle/04-LIMITATIONS.md) - explicit Phase 5 carryover items
- [`mux/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs) - lifecycle ownership and notifications
- [`mux/src/task_panes.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/task_panes.rs) - task-pane record model and tests
- [`kaku/src/cli/list_task_panes.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli/list_task_panes.rs) - CLI contract pattern
- [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs) - GUI consumer behavior

### Secondary (MEDIUM confidence)

- local test results from 2026-03-27:
  - `cargo test --locked -p mux task_panes -- --nocapture` -> failed on rerun quoting assertion
  - `cargo test --locked -p kaku lifecycle_contracts -- --nocapture` -> passed

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - based on repo-local workspace structure and versions.
- Architecture: HIGH - directly supported by spec, plans, and current source ownership.
- Pitfalls: MEDIUM-HIGH - UAT blocker root cause remains unproven, but the planning implications are strongly evidenced.

**Research date:** 2026-03-27
**Valid until:** 2026-04-10
