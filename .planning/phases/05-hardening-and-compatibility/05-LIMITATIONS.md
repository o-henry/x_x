# Known limitations

- `pipe-pane` duplicates live pane output, but the tee file is raw terminal traffic rather than a normalized log stream. Manual verification captured shell echo and escape sequences in `/tmp/phase5-task-pane.log`.
- Task-pane records intentionally remain inspectable after failure, rerun, or remain-on-exit flows. Long-lived sessions still depend on explicit cleanup semantics rather than aggressive automatic pruning.
- Task Center additivity is now covered by both the targeted `kaku-gui` test slice and a live desktop check, but that manual confirmation exercised the default selected row rather than an exhaustive set of overlay actions.
- The final manual compatibility pass focused on the default workspace, failing-pane survivability, and lifecycle mutations. Broader multi-window and multi-workspace navigation still rely on the preserved baseline CLI compatibility slice and existing Kaku behavior.
