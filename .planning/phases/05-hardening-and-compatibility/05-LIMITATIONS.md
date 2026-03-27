# Known limitations

- `respawn-pane` is still not verified end-to-end against the live GUI runtime. During the 2026-03-27 manual Phase 5 run, `./target/debug/kaku cli respawn-pane --pane-id 2` failed because the CLI received unexpected server status `respawned` instead of the documented `respawn`.
- `pipe-pane` duplicates live pane output, but the tee file is raw terminal traffic rather than a normalized log stream. Manual verification captured shell echo and escape sequences in `/tmp/phase5-task-pane.log`.
- Task-pane records intentionally remain inspectable after failure, rerun, or remain-on-exit flows. Long-lived sessions still depend on explicit cleanup semantics rather than aggressive automatic pruning.
- Task Center additivity is well covered by targeted `kaku-gui` tests, but this Wave 4 closeout did not complete a visible overlay interaction in the desktop session. The live runtime confirmation in this phase is that Kaku still launched as a normal terminal window without a new dashboard shell.
- The final manual compatibility pass focused on the default workspace, failing-pane survivability, and lifecycle mutations. Broader multi-window and multi-workspace navigation still rely on the preserved baseline CLI compatibility slice and existing Kaku behavior.
