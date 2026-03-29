---
status: partial
phase: 04-task-pane-lifecycle
source:
  - 04-task-pane-lifecycle-01-SUMMARY.md
  - 04-task-pane-lifecycle-02-SUMMARY.md
  - 04-task-pane-lifecycle-03-SUMMARY.md
  - 04-task-pane-lifecycle-04-SUMMARY.md
  - 04-task-pane-lifecycle-05-SUMMARY.md
started: 2026-03-27T08:50:32Z
updated: 2026-03-27T09:10:30Z
---

## Current Test

[testing paused — 2 items outstanding]

## Tests

### 1. Remain-on-exit intent is visible through the lifecycle CLI
expected: `./target/debug/kaku cli set-remain-on-exit --pane-id <live-pane-id> --remain-on-exit true` 를 실행한 뒤 `./target/debug/kaku cli list-task-panes --pane-id <same-pane-id> --format json` 를 보면, 해당 pane record가 JSON 배열에 나타나고 `remain_on_exit: true`, `is_dead: false`, `is_failed: false` 가 보인다.
result: pass

### 2. Silence-watchdog and pipe-pane mutations are reflected in lifecycle state
expected: `./target/debug/kaku cli silence-watchdog --pane-id <live-pane-id> --silenced true` 와 `./target/debug/kaku cli pipe-pane --pane-id <live-pane-id> --file <path>` 를 실행한 뒤 `list-task-panes --pane-id <same-pane-id> --format json` 를 보면 `silenced: true` 와 `tee_path` 가 같이 보인다.
result: pass

### 3. Pipe-pane tee duplicates pane output to a file without breaking normal pane output
expected: tee를 켠 pane에 텍스트를 출력하면 Kaku pane에는 기존처럼 출력이 보이고, 지정한 파일에도 같은 내용이 append 된다.
result: pass

### 4. Dead task panes remain discoverable and rerunnable after exit
expected: rerun metadata가 있는 task pane이 실패 종료된 뒤에도 `list-task-panes --format json` 또는 Task Center에 dead/failed row가 남아 있고, `rerun-pane` 또는 `respawn-pane` 을 실행하면 새 pane id가 반환된다.
result: issue
reported: "`spawn` 으로 failing pane을 띄우자 `assets/macos/Kaku.app/Contents/Resources/kaku.lua:167-192` 의 `is_low_resolution_screen()` 경로에서 `C stack overflow` 가 반복 출력됐고, 그 뒤 `list-task-panes` 는 GUI socket EOF로 실패함"
severity: blocker

### 5. Task Center keeps durable failed/rerun state after pane exit
expected: failed pane이 종료된 뒤 Task Center를 열면 live pane이 사라진 뒤에도 failed row가 남아 있고, rerun metadata가 있는 경우에만 rerun affordance가 유지된다.
result: blocked
blocked_by: prior-phase
reason: "Test 4에서 failing pane fixture를 만드는 순간 kaku.lua recursion으로 GUI socket이 죽어서, dead-pane after-exit 상태를 Task Center에서 이어서 확인할 수 없었음"

## Summary

total: 5
passed: 3
issues: 1
pending: 0
skipped: 0
blocked: 1

## Gaps

- truth: "rerun metadata가 있는 task pane이 실패 종료된 뒤에도 `list-task-panes --format json` 또는 Task Center에 dead/failed row가 남아 있고, `rerun-pane` 또는 `respawn-pane` 을 실행하면 새 pane id가 반환된다."
  status: failed
  reason: "User reported: `spawn` 으로 failing pane을 띄우자 `assets/macos/Kaku.app/Contents/Resources/kaku.lua:167-192` 의 `is_low_resolution_screen()` 경로에서 `C stack overflow` 가 반복 출력됐고, 그 뒤 `list-task-panes` 는 GUI socket EOF로 실패함"
  severity: blocker
  test: 4
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
