---
status: complete
phase: 01-notification-routing
source:
  - 01-notification-routing-01-SUMMARY.md
  - 01-notification-routing-02-SUMMARY.md
  - 01-notification-routing-03-SUMMARY.md
  - 01-notification-routing-04-SUMMARY.md
  - 01-notification-routing-05-SUMMARY.md
started: 2026-03-27T01:42:37Z
updated: 2026-03-27T03:36:40Z
---

## Current Test

[testing complete]

## Tests

### 1. Create a workspace-scoped notification
expected: Run `kaku cli notify --workspace unity --kind build.failed --title "Build failed"` and the command prints a JSON notification record with null scope ids for workspace-only scope.
result: pass

### 2. List and mutate notification records from the CLI
expected: `list-notifications --format json`, `mark-read`, `mark-unread`, and `clear-notifications` each return stable JSON with the documented notification record fields or mutation fields (`updated_count`, `cleared_count`, `notification_ids`).
result: pass

### 3. Jump and discovery commands resolve unread targets
expected: `jump-next-unread`, `jump-prev-unread`, `identify`, and `capabilities` return stable JSON. Jump commands should print `{"pane_id": <id|null>}`, `identify` should print workspace/window/tab/pane ids, and `capabilities` should list notification commands and unread modes.
result: pass

### 4. Unread notifications appear on the existing tabbar
expected: When a tab has unread notifications, its default tab title is prefixed with `! `. Tabs without unread notifications keep the previous title behavior.
result: pass
reported: "초기에는 marker가 보이지 않았지만, workspace-only unread count를 tab count에 포함시키고 실제 `format-tab-title` Lua 런타임 경로에도 prefix를 반영한 뒤, 사용자 스크린샷에서 하단 탭 제목 `code/vibe` 앞에 선행 marker가 표시됨. 현재 폰트에서는 `!` 가 매우 가는 세로획처럼 보임."

### 5. Clear-on-focus vs sticky unread behavior works
expected: A `clear-on-focus` notification becomes read when its target pane is focused, while a `sticky` notification remains unread until `mark-read` or `clear-notifications` changes it.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
