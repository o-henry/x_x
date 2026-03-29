# Known limitations

- The rebuilt Phase 06 binary is now freshly proven for socket responsiveness plus the core keyboard operator path (`Task Center` open, `Arrow Down`, `Enter`), but the latest live pass still leans on automated coverage for mouse-only actions, metadata prompt round-trips, and tabbar-marker click handoff.
- Tabbar operator markers remain an entry affordance, not a workflow surface. They intentionally hand off to Task Center instead of offering inline editing or multi-action chrome in the tab rail.
- Workspace metadata editing is intentionally scoped to status and progress through compact prompt/confirm flows. Workspace logs still rely on the CLI path rather than a new editor surface.
- The current shipped visual treatment is still more utilitarian than the provided video reference: it remains text-heavy in places, keeps the compact ` · ops` marker, and has not yet reached the desired DM Mono plus icon-first operator polish.
