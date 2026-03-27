---
status: testing
phase: 03-task-center-overlay
source:
  - 03-task-center-overlay-01-SUMMARY.md
  - 03-task-center-overlay-02-SUMMARY.md
  - 03-task-center-overlay-03-SUMMARY.md
  - 03-task-center-overlay-04-SUMMARY.md
  - 03-task-center-overlay-05-SUMMARY.md
started: 2026-03-27T15:40:00+09:00
updated: 2026-03-27T15:44:00+09:00
---

## Current Test

number: 2
name: Task Center search and filters narrow the normalized list
expected: |
  Task Center에서 free-text 검색과 `unread`, `failed`, `running`, `workspace:<name>`, `source:<kind>`, `kind:<kind>` 토큰 필터를 사용하면 결과 목록이 해당 조건으로 정상적으로 줄어든다.
awaiting: user response

## Tests

### 1. Task Center overlay opens from a native command
expected: Kaku GUI가 켜진 상태에서 Task Center native command를 실행하면, 기존 launcher와 별개인 전용 overlay가 열린다. overlay 제목은 Task Center이고, 빈 상태여도 크래시하지 않고 정상적인 empty state를 보여준다.
result: issue
reported: "cmd+shift+J 아무것도 안뜸"
severity: major

### 2. Task Center search and filters narrow the normalized list
expected: Task Center에서 free-text 검색과 `unread`, `failed`, `running`, `workspace:<name>`, `source:<kind>`, `kind:<kind>` 토큰 필터를 사용하면 결과 목록이 해당 조건으로 정상적으로 줄어든다.
result: pending

### 3. Task Center row actions focus target and clear unread
expected: unread가 있는 row를 선택해 focus action을 실행하면 해당 target으로 이동하고, clear unread action을 실행하면 같은 row의 unread 상태가 기존 notification 경로를 통해 사라진다.
result: pending

### 4. Task Center rerun is shown only when rerun metadata exists
expected: failed row 중 rerun metadata가 있는 경우에만 rerun action이 보이거나 동작하고, metadata가 없는 row에는 rerun이 제공되지 않는다.
result: pending

## Summary

total: 4
passed: 0
issues: 1
pending: 3
skipped: 0
blocked: 0

## Gaps

- truth: "Kaku GUI가 켜진 상태에서 Task Center native command를 실행하면, 기존 launcher와 별개인 전용 overlay가 열린다. overlay 제목은 Task Center이고, 빈 상태여도 크래시하지 않고 정상적인 empty state를 보여준다."
  status: failed
  reason: "User reported: cmd+shift+J 아무것도 안뜸"
  severity: major
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
