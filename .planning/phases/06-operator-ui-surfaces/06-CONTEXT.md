# Phase 06 Context — Operator UI Surfaces

## Decisions

- The next phase must prioritize native UI over CLI recall for everyday control-plane work.
- Common control-plane tasks should be operable from Kaku with mouse and keyboard, without requiring the developer to remember subcommand names.
- The UI must remain additive to Kaku rather than turning the app into a separate dashboard shell.
- The desired tone is closer to the attached reference video at `/Users/henry/Downloads/566510994-6f3047c2-e2b6-49f2-b536-570a1570d0f8.mp4`: dark, focused, tool-like, and workspace-centric.
- The visual direction should feel more intentional and operable than the current CLI-first control plane, while still respecting Kaku's terminal-native character.

## The Agent's Discretion

- Which specific native surfaces should carry the common-path actions first: Task Center, tabbar affordances, context menus, inspector overlays, or command surfaces.
- How strongly the phase should emphasize mouse click targets versus keyboard discoverability, as long as both are supported.
- How much of the workspace metadata editing flow belongs inline versus in overlays.
- Which pieces of the reference video's visual language are useful to borrow without creating a non-Kaku shell.

## Deferred Ideas

- Full dashboard shell replacement
- Browser-style side panels or embedded web content
- Any UI that requires React, Tauri, Electron, WebView, Swift, or Xcode
