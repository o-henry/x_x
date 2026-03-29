#!/usr/bin/env bash
set -euo pipefail

PAYLOAD="${1:-}"
if [ -z "${PAYLOAD}" ]; then
  PAYLOAD="$(cat 2>/dev/null || true)"
fi

MSG="Turn complete"

if command -v jq >/dev/null 2>&1; then
  PARSED="$(
    printf '%s' "$PAYLOAD" \
      | jq -r '."last-assistant-message" // .message // empty' 2>/dev/null \
      | head -c 140 \
      || true
  )"
  if [ -n "${PARSED}" ] && [ "${PARSED}" != "null" ]; then
    MSG="${PARSED}"
  fi
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE="${KAKU_NOTIFY_WORKSPACE:-default}"
KIND="${KAKU_NOTIFY_KIND:-codex.turn.complete}"
TITLE="${KAKU_NOTIFY_TITLE:-Codex}"

find_kaku_bin() {
  if [ -n "${KAKU_BIN:-}" ] && [ -x "${KAKU_BIN}" ]; then
    printf '%s\n' "${KAKU_BIN}"
    return 0
  fi

  if [ -x "${REPO_ROOT}/target/debug/kaku" ]; then
    printf '%s\n' "${REPO_ROOT}/target/debug/kaku"
    return 0
  fi

  if command -v kaku >/dev/null 2>&1; then
    command -v kaku
    return 0
  fi

  return 1
}

if KAKU_CLI="$(find_kaku_bin 2>/dev/null)"; then
  if "${KAKU_CLI}" cli notify \
    --workspace "${WORKSPACE}" \
    --kind "${KIND}" \
    --title "${TITLE}" \
    --body "${MSG}" \
    --unread-mode sticky \
    >/dev/null 2>&1; then
    exit 0
  fi
fi

osascript \
  -e 'on run argv' \
  -e 'display notification (item 1 of argv) with title "Codex"' \
  -e 'end run' \
  -- "${MSG}" >/dev/null 2>&1 || true
