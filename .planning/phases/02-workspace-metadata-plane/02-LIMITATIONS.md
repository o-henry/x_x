# Phase 2 Limitations

- Phase 2 workspace logs are in-memory only and bounded to the newest `200` entries per workspace.
- Default GUI surfacing for workspace metadata is currently the existing tab title path; this phase does not add a dedicated metadata overlay or sidebar.
- Phase 2 exposes `set-progress` and `clear-progress`, but does not add a separate `list-progress` command.
- Workspace metadata is runtime state owned by mux and is not persisted across full application restarts in this phase.
