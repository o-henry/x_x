# Known limitations

- Phase 1 keeps notification surfacing to the existing tabbar marker and does not add pane-adjacent hints or a separate Task Center overlay.
- The CLI contract tests validate stable request and JSON response shapes locally; they do not stand up a live mux server end-to-end.
- Notification timestamps currently follow the server-side string formatting already used by the transport layer.
- Bell-bridge follow-up work, richer unread navigation UX, and workspace metadata flows remain deferred to later phases.
