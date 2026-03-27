# Known limitations

- The targeted `kaku-gui` test slices now cover the common operator path end-to-end inside the shipped controller/overlay/tabbar seams, but the 2026-03-27 closeout plan did not complete a fresh live desktop pass because `./target/debug/kaku-gui start --always-new-process` did not publish a new responsive GUI socket during the reattempt. The prior Phase 05 live runtime remains the most recent desktop shell confirmation.
- The live reattempt also means this closeout did not freshly re-confirm every inline Task Center action against a desktop session on the rebuilt Phase 06 binary. Focus, clear unread, rerun/remain-on-exit affordances, metadata prompts, and tabbar marker routing are all covered by automated tests, but not by a new manual desktop capture in this plan.
- Tabbar operator markers remain an entry affordance, not a workflow surface. They intentionally hand off to Task Center instead of offering inline editing or multi-action chrome in the tab rail.
- Workspace metadata editing is intentionally scoped to status and progress through compact prompt/confirm flows. Workspace logs still rely on the CLI path rather than a new editor surface.
