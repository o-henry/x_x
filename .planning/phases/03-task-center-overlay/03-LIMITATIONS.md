# Known limitations

- Failed and running rows still use Phase 3 limited-mode heuristics from current mux pane state (`is_dead` plus progress/user-var signals). Phase 4 still owns durable lifecycle semantics.
- Rerun is consumer-side only. It is offered only for failed rows marked `rerun_available`, and it still depends on runtime pane user vars such as `KAKU_RERUN_COMMAND` being present when the action is invoked.
- Task Center filter UX is query-token based in this phase. There is no separate chip bar or secondary filter panel yet.
- Focus and clear-unread actions are wired through existing Kaku paths and prioritize correctness over keeping the overlay live-updating across every mutation.
