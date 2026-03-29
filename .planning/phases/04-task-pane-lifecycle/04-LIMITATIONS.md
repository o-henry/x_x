# Known limitations

- `silence-watchdog` currently persists pane-scoped silence intent and exposes it through CLI / Task Center consumers, but deeper watchdog policy semantics still belong to Phase 5 hardening.
- `rerun-pane` and `respawn-pane` depend on durable rerun metadata captured from pane user vars. Panes that never seed rerun metadata correctly will not expose rerun actions later.
- `respawn-pane` currently reuses the same durable spawn path as rerun and differs mainly in reported status and operator intent; Phase 5 can harden semantics further if they need to diverge.
- `pipe-pane` is additive append-only tee behavior over decoded pane output. It does not replace pane IO, provide rotation, or guarantee atomic multi-pane file coordination.
- Task-pane records are durable enough for Phase 4 CLI and Task Center consumers, but stale-record cleanup and longer-lived retention policies remain hardening concerns for Phase 5.
- Remote / multi-client lifecycle fan-out is intentionally conservative in this phase. Local Kaku GUI consumers refresh correctly, while broader lifecycle event streaming remains a Phase 5 consistency concern.
