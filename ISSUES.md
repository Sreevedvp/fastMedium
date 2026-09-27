# fastMedium issues and migration checklist

This file tracks the requested architecture change, login repair, and reference redesign.
Check an item only when its implementation and relevant verification are complete.

## Architecture and development
- [x] Replace Dioxus Fullstack server functions with a standalone Axum + Tokio API.
- [x] Connect the Dioxus browser app through typed JSON HTTP requests.
- [x] Preserve existing SQLite accounts, sessions, and stories.
- [x] Add one development script that starts both processes, checks ports/readiness, and stops both on exit.
- [x] Document setup, environment variables, API routes, and the run command.

## Authentication and correctness
- [x] Reproduce stale signed-out navigation after successful login (session cookie is created).
- [x] Refresh shared auth state after login, registration, and logout.
- [x] Verify valid/invalid login, session persistence after reload, logout, and protected routes.
- [x] Verify draft creation/editing, publish/unpublish, and ownership checks through the new API.
- [x] Add repeatable regression checks using an isolated temporary database.

## Reference design
- [x] Apply the warm paper palette, serif typography, green accents, and publication header.
- [x] Complete the feed, reference reader, and responsive side rail.
- [x] Match dashboard, editor, profile, and auth screens to the same design.
- [x] Verify desktop/mobile rendering and working navigation.

## Completion
- [x] Run browser and server build checks and regression tests.
- [x] Start and verify the app through the single development script.
- [x] Remove temporary verification records and record any remaining limitations here.

## Verification log
- Both independent `cargo check` commands pass (Tokio backend and Wasm browser).
- Runtime and browser regression checks are complete.
- Isolated API regression suite passed: auth, cookie flags, invalid passwords, logout, expiry, ownership, CRUD, publish/unpublish, CORS, and persistence after an API restart.
- Existing browser session was recognized by the new API against the original SQLite file.
- Browser verified: invalid-password error, immediate authenticated navigation, reload persistence, logout, protected-route redirect, draft save, and edit loading. No browser console errors during these checks.
- Visually reviewed editor and reader at desktop and 390px phone widths; document width remains within the viewport.
- Launcher lifecycle passed: both HTTP services become ready; a second launch reports occupied ports; Ctrl+C returns success and releases both listeners. Fixed TIME_WAIT false positives in the port preflight check.
- Removed the synthetic verification account and its draft/session records. Existing user content was preserved.

## Scope notes
- The app is configured for local development. `./dev.sh` is the single start command.
- Frontend changes reload automatically. Restart the script after backend changes.
- The reference’s visual style is implemented across the app. The connected-ideas diagram is illustrative; live graph analytics, Notion synchronization, and block-editor integrations are not implemented.
- The labeled sample essay is a design preview, not a database post. Real published stories appear in the feed.

## Skills, publications, and The Notion Bar refresh (in progress)
- [x] Inspect reference fonts, colors, borders, and navigation; confirm skills mean profile expertise tags.
- [x] Persist expertise tags with owner-only add/remove controls and public profile display.
- [x] Add publications with create/edit/delete, public pages, and story organization.
- [x] Protect publication changes and story assignment with ownership checks; keep drafts private.
- [x] Replace the previous palette/layout with blush, coral, Poppins, and Geist Mono from the new reference.
- [x] Verify migrations preserve existing content and add API regression coverage.
- [ ] Verify the actual browser flows and responsive styling, then restart with dev.sh.

- Latest verification: both targets compile; expanded isolated API suite passes. Browser verified expertise add/reload/remove and publication create/edit. Reference fonts load and desktop has no horizontal overflow. Temporary fixtures removed. Phone verification remains pending after switching to the requested portfolio work.

## Architecture hardening (2026-09-25)
- [x] Audit the current app and write ARCHITECTURE_PLAN.md.
- [ ] Make migrations transactional and versioned; verify backup/restore.
- [ ] Protect Markdown output and strengthen validation and session handling.
- [ ] Add origin checks, auth limits, request deadlines, private cache headers, and request IDs.
- [ ] Prevent lost story updates, slug races, and duplicate publication transitions.
- [ ] Move search to the server and make pagination safe.
- [ ] Confirm destructive story actions and improve editor error/retry behavior.
- [ ] Pass isolated regressions, both Rust targets, and live desktop/mobile verification.
