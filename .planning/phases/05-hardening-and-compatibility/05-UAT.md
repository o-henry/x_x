---
status: partial
phase: 05-hardening-and-compatibility
source:
  - 05-hardening-and-compatibility-01-SUMMARY.md
  - 05-hardening-and-compatibility-02-SUMMARY.md
  - 05-hardening-and-compatibility-03-SUMMARY.md
  - 05-hardening-and-compatibility-05-SUMMARY.md
started: 2026-03-27T11:16:06Z
updated: 2026-03-27T11:54:49Z
runtime:
  gui: ./target/debug/kaku-gui start --always-new-process
  cli: ./target/debug/kaku cli
---

## Current Test

[manual Wave 4 compatibility run completed with one open runtime limitation]

## Session Setup

1. Launched a real GUI runtime with `./target/debug/kaku-gui start --always-new-process`.
2. Confirmed the live GUI accepted CLI connections by running `./target/debug/kaku cli list --format json`.
3. Used the same local workspace for the full manual pass: `/Users/henry/Documents/code/vibe/hybrid/x_x`.

## Commands and Observations

### Baseline session state

```bash
./target/debug/kaku cli list --format json
./target/debug/kaku cli list-task-panes --format json
```

Observed:

- `list` returned the live default workspace window and pane `0`.
- `list-task-panes` already exposed live pane `0` plus prior dead task-pane records, confirming the mux lifecycle registry was readable before any new mutations.

### Baseline pane/tab behavior before lifecycle mutations

```bash
./target/debug/kaku cli spawn --cwd "$PWD" -- /bin/sh -lc 'printf phase5-smoke'
./target/debug/kaku cli split-pane --right --percent 50
./target/debug/kaku cli send-text --pane-id 3 'phase5 smoke\n'
./target/debug/kaku cli get-text --pane-id 3 --start-line -20
./target/debug/kaku cli set-tab-title 'phase5-smoke'
./target/debug/kaku cli list --format json
```

Observed:

- `spawn` returned pane `2` and `split-pane` returned pane `3`.
- `send-text` and `get-text` proved the live pane still accepted normal shell input and surfaced `phase5 smoke`.
- `set-tab-title` updated the active tab title to `phase5-smoke`.
- `list` showed the expected two-pane split in the default workspace, so the basic pane/tab flow remained usable after the lifecycle work.

### Lifecycle mutation path on a live pane

```bash
./target/debug/kaku cli set-remain-on-exit --pane-id 3 --remain-on-exit true
./target/debug/kaku cli silence-watchdog --pane-id 3 --silenced true
./target/debug/kaku cli pipe-pane --pane-id 3 --file /tmp/phase5-task-pane.log
./target/debug/kaku cli list-task-panes --pane-id 3 --format json
./target/debug/kaku cli send-text --pane-id 3 'echo tee-check-from-shell\n'
sleep 2
cat /tmp/phase5-task-pane.log
./target/debug/kaku cli get-text --pane-id 3 --start-line -20
```

Observed:

- `set-remain-on-exit`, `silence-watchdog`, and `pipe-pane` all returned successful JSON payloads.
- `list-task-panes --pane-id 3 --format json` recorded `remain_on_exit: true`, `silenced: true`, and `tee_path: "/tmp/phase5-task-pane.log"`.
- The tee file was created after shell output flowed through the pane and captured raw terminal traffic plus the typed command. This confirms additive duplication works, but the file is not a cleaned log stream.

### Prior failing-pane fixture path

```bash
./target/debug/kaku cli spawn --cwd "$PWD" -- /bin/sh -lc 'echo PHASE4_FAILING_TASK; false'
sleep 2
./target/debug/kaku cli list-task-panes --format json
```

Observed:

- The failing spawn returned pane `4`.
- Unlike the Phase 4 blocker, the GUI session stayed alive and `list-task-panes` continued working.
- After the short wait, pane `4` appeared as `is_dead: true`, `is_failed: true`, `rerun_available: true`, with the current working directory preserved as `file:///Users/henry/Documents/code/vibe/hybrid/x_x/`.
- The old `kaku.lua` recursion -> GUI socket EOF chain did not reproduce in this manual run.

### Rerun and respawn follow-through

```bash
cargo build --locked -p kaku -p kaku-gui -p wezterm-mux-server-impl
./target/debug/kaku-gui start --always-new-process
ls -la ~/.local/share/kaku/default-fun.tw93.kaku ~/.local/share/kaku/gui-sock-2572
WEZTERM_UNIX_SOCKET=$HOME/.local/share/kaku/gui-sock-2572 ./target/debug/kaku cli --no-auto-start --prefer-mux list --format json
WEZTERM_UNIX_SOCKET=$HOME/.local/share/kaku/gui-sock-2572 ./target/debug/kaku cli --no-auto-start --prefer-mux spawn --cwd "$PWD" -- /bin/sh -lc 'echo PHASE5_RESPAWN_FIXTURE; false'
sleep 2
WEZTERM_UNIX_SOCKET=$HOME/.local/share/kaku/gui-sock-2572 ./target/debug/kaku cli --no-auto-start --prefer-mux list-task-panes --pane-id 2 --format json
WEZTERM_UNIX_SOCKET=$HOME/.local/share/kaku/gui-sock-2572 ./target/debug/kaku cli --no-auto-start --prefer-mux rerun-pane --pane-id 2
WEZTERM_UNIX_SOCKET=$HOME/.local/share/kaku/gui-sock-2572 ./target/debug/kaku cli --no-auto-start --prefer-mux respawn-pane --pane-id 2
WEZTERM_UNIX_SOCKET=$HOME/.local/share/kaku/gui-sock-2572 ./target/debug/kaku cli --no-auto-start --prefer-mux list-task-panes --format json
```

Observed:

- Fresh rebuilt binaries were used for the rerun because the previously published default GUI socket was stale. The new `start --always-new-process` session published `~/.local/share/kaku/default-fun.tw93.kaku -> ~/.local/share/kaku/gui-sock-2572`, and the live CLI verification used that exact socket via `WEZTERM_UNIX_SOCKET`.
- The retained failing fixture on pane `2` stayed visible as `is_dead: true`, `is_failed: true`, and `rerun_available: true`, confirming the rerun/respawn metadata path was available in the fresh session.
- `rerun-pane --pane-id 2` succeeded and returned:

```json
{
  "pane_id": 2,
  "spawned_pane_id": 3,
  "status": "rerun"
}
```

- `respawn-pane --pane-id 2` also completed successfully and returned:

```json
{
  "pane_id": 2,
  "spawned_pane_id": 4,
  "status": "respawn"
}
```

- No runtime/server code change was required after the fresh rebuild. The stale mismatch from the earlier Wave 4 artifact did not reproduce once the CLI was pointed at the newly published live GUI socket.

## Tests

### 1. Prior failing-pane fixture path no longer kills the GUI session
expected: spawning `/bin/sh -lc 'echo PHASE4_FAILING_TASK; false'` no longer reproduces the old `kaku.lua` recursion -> GUI socket EOF chain, and `list-task-panes --format json` stays usable afterwards.
result: pass

### 2. Baseline pane/tab behavior still works before and after lifecycle mutations
expected: `spawn`, `split-pane`, `send-text`, `get-text`, and `set-tab-title` remain usable in the live runtime while lifecycle metadata stays inspectable.
result: pass

### 3. Remain-on-exit, silence-watchdog, and pipe-pane mutations remain visible through `list-task-panes`
expected: live-pane lifecycle mutations return stable JSON and remain visible in the mux-owned task-pane registry.
result: pass

### 4. Pipe-pane duplicates output to a file without replacing normal pane interaction
expected: tee output is appended to a file while pane interaction remains normal.
result: pass
notes: "The tee file contains raw terminal traffic and shell echo, so duplication is additive rather than sanitized."

### 5. Rerun and respawn are both proven end-to-end from retained task-pane records
expected: both `rerun-pane` and `respawn-pane` return the documented machine-readable contract in the live runtime.
result: pass

### 6. Task Center remains an additive consumer overlay in the live app shell
expected: the app still presents as a normal Kaku window and Task Center remains an additive overlay rather than a new dashboard shell.
result: partial
notes: "The live session remained a standard Kaku window and targeted `cargo test --locked -p kaku-gui task_center -- --nocapture` stayed green, but this manual run did not complete a visible desktop overlay interaction."

## Summary

total: 6
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 0
partial: 1
