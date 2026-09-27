# fastMedium Implementation Plan

A step-by-step plan for building a Medium-like publishing platform with Rust and Dioxus. The goal is to keep each step small enough to implement and verify yourself.

## Project context

The current project is a Dioxus `0.7.1` application with:

- Dioxus Router enabled
- Dioxus fullstack enabled
- Web as the default target
- A starter `src/main.rs` containing a home route, blog route, navbar, and example server function
- Static assets under `assets/`

This plan keeps the current Dioxus approach. Do not introduce Axum separately unless the project later needs a dedicated API server.

## Product scope

### First usable version

Build only these features first:

1. View a feed of published posts.
2. Open an individual post.
3. Register an account.
4. Log in and log out.
5. Create and save a draft.
6. Publish, edit, and delete your own posts.
7. View an author profile.

### Later features

Add these only after the first usable version works:

- Comments
- Claps/reactions
- Following authors
- Bookmarks/reading list
- Tags
- Search
- Markdown or rich-text editing
- Notifications
- Image uploads
- Moderation and reporting

## Development workflow

For each milestone:

1. Define the user-visible behavior.
2. Decide the route and the data it needs.
3. Implement the smallest working version.
4. Run formatting and checks.
5. Test the behavior manually in the browser.
6. Add tests for logic and server functions.
7. Refactor only after the behavior works.

Useful commands:

```bash
cargo fmt -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
dx serve --platform web
```

Use the command supported by your installed Dioxus CLI if the `dx` options differ.

## Phase 0: Understand the starter app

### Goal

Replace the Dioxus starter experience with the initial fastMedium shell.

### Tasks

- Remove or repurpose the starter `Hero`, `Blog`, and `Echo` examples.
- Keep a root `App` component and a router.
- Create a shared layout with:
  - fastMedium branding
  - Home link
  - Write link
  - Login/register links for logged-out users
  - Profile/logout controls for logged-in users later
- Add a simple home page with an empty-state message.
- Keep the existing stylesheet initially; replace starter styles incrementally.

### Checkpoint

`dx serve --platform web` opens the app, the navigation works, and the home page clearly represents fastMedium.

## Phase 1: Establish the application structure

### Goal

Prevent all application code from growing inside `src/main.rs`.

### Suggested structure

```text
src/
├── main.rs
├── app.rs
├── routes.rs
├── components/
│   ├── mod.rs
│   └── layout.rs
├── pages/
│   ├── mod.rs
│   ├── home.rs
│   ├── post.rs
│   └── profile.rs
├── models/
│   ├── mod.rs
│   ├── post.rs
│   └── user.rs
├── server/
│   ├── mod.rs
│   └── posts.rs
└── error.rs
```

Do not create empty modules just to match the diagram. Move code when there is a real responsibility to separate.

### Checkpoint

The application still runs after moving the router, layout, and pages into modules.

## Phase 2: Design the domain model before adding a database

### Goal

Decide what the system means before implementing persistence.

### Initial entities

#### User

- `id`
- `username`
- `display_name`
- `email`
- `password_hash`
- `bio`
- `avatar_url` (optional)
- `created_at`

#### Post

- `id`
- `author_id`
- `title`
- `slug`
- `excerpt`
- `content`
- `status` (`draft` or `published`)
- `created_at`
- `updated_at`
- `published_at` (optional)

### Important rules

- IDs are internal identifiers.
- Slugs are public URL identifiers and must be unique.
- Drafts must not appear in the public feed.
- Only the author can edit, publish, unpublish, or delete their own post.
- Store password hashes, never plain-text passwords.

### Checkpoint

Write down the fields, ownership rules, and state transitions before writing SQL or server functions.

## Phase 3: Add persistence

### Recommended first database

Use SQLite for local development. It keeps the project simple and can be replaced or migrated later if needed.

Potential dependencies to evaluate when you reach this phase:

- `sqlx` with SQLite and Tokio runtime features
- `dotenvy` or another environment configuration tool
- `time` or `chrono` for timestamps, using one consistently
- A password hashing crate such as `argon2`
- A session or signed-cookie solution compatible with the chosen Dioxus fullstack setup

Check current Dioxus 0.7 fullstack guidance before choosing session and server-state APIs.

### Tasks

- Create a database connection configuration.
- Add migrations rather than creating tables at runtime manually.
- Add the `users` and `posts` tables.
- Add indexes for:
  - user email
  - username
  - post slug
  - published posts ordered by publication date
- Add a small seed strategy for local development, if useful.

### Checkpoint

The application can connect to the local database and read an empty post list without crashing.

## Phase 4: Build the public reading experience

### Routes

Start with routes similar to:

```text
/
/posts/:slug
/@:username
```

Choose the exact Dioxus route syntax supported by the current router version.

### Tasks

- Create a typed post model for data returned to the UI.
- Implement a server function for published posts.
- Show a feed containing:
  - title
  - excerpt
  - author name
  - publication date
  - reading-time estimate later
- Implement the post detail page.
- Render missing posts as a friendly not-found state.
- Add basic pagination only after a single-page feed works.

### Checkpoint

A seeded published post appears on the home page and opens at its slug URL. A draft does not appear publicly.

## Phase 5: Add authentication

### Routes

```text
/register
/login
/logout
```

### Tasks

- Validate registration input.
- Normalize email and username consistently.
- Hash passwords with Argon2 or another well-reviewed password hashing algorithm.
- Create a session after successful login.
- Make the current-user state available to the layout and protected pages.
- Implement logout and session invalidation.
- Return safe, user-friendly errors without exposing sensitive details.

### Security requirements

- Never return password hashes to the client.
- Do not identify whether an email exists in a way that enables account enumeration during sensitive flows.
- Add CSRF protection if using cookie-authenticated state-changing requests.
- Use secure, HTTP-only cookies in production.

### Checkpoint

A user can register, log in, see authenticated navigation, and log out. Refreshing the page preserves or removes the session as expected.

## Phase 6: Build the author dashboard and publishing flow

### Routes

```text
/dashboard
/write
/write/:id
```

### Tasks

- Restrict dashboard and editor routes to authenticated users.
- Build a form for title, excerpt, and content.
- Save a new post as a draft.
- Edit an existing draft.
- Publish a draft.
- Unpublish a published post.
- Delete only posts owned by the current user.
- Generate and validate a unique slug when publishing.
- Display clear loading, success, and error states.

### Checkpoint

A logged-in user can create a draft, edit it, publish it, view it publicly, unpublish it, and delete it. Another user cannot modify it.

## Phase 7: Improve the reading and author experience

### Tasks

- Build the author profile page.
- List an author's published posts.
- Add a responsive article layout.
- Add navigation back to the feed and profile.
- Add accessible labels, focus states, semantic headings, and keyboard-friendly forms.
- Add pagination or “load more” behavior.
- Add an estimated reading time based on word count.

### Checkpoint

A reader can move naturally from feed → article → author profile → another article on a phone-sized viewport.

## Phase 8: Add social features one at a time

Implement each feature as a separate slice with its own data model, server operation, UI, and tests.

### Comments

- Add `comments` with `post_id`, `author_id`, `body`, timestamps, and optional moderation state.
- Require authentication to comment.
- Allow authors to delete or moderate comments according to the rules you define.

### Claps

- Add a table enforcing one user/post relationship.
- Decide whether a user can clap once or multiple times.
- Return aggregate clap counts with post data.

### Following

- Add a `follows` table with a unique follower/followed pair.
- Add follow/unfollow operations.
- Add a feed mode for followed authors after the general feed works.

### Bookmarks

- Add a `bookmarks` table with a unique user/post pair.
- Add save/remove actions and a private reading-list page.

## Phase 9: Search, validation, and production readiness

### Tasks

- Add server-side validation for every write operation.
- Add database constraints in addition to UI validation.
- Add search over title, excerpt, and content.
- Add rate limiting for login, registration, comments, and reactions.
- Add structured logging and request error reporting.
- Add a consistent application error type.
- Add authorization tests for every protected operation.
- Add database backup and migration procedures.
- Configure production secrets through environment variables.
- Review cookie, CORS, headers, and content sanitization behavior.

## Testing strategy

### Unit tests

Test pure logic such as:

- Slug generation
- Slug normalization
- Input validation
- Reading-time calculation
- Pagination calculations
- Post state transitions

### Server-function or integration tests

Test:

- Published posts are returned.
- Drafts are excluded from public queries.
- Login creates a valid authenticated state.
- Unauthenticated users cannot write.
- Users cannot edit another user's post.
- Publishing sets publication data correctly.
- Duplicate usernames, emails, and slugs are rejected.

### Manual checks

For each feature, verify:

- Loading state
- Empty state
- Success state
- Validation errors
- Server errors
- Refresh behavior
- Direct navigation to the URL
- Mobile layout

## Suggested implementation order for your next sessions

### Session 1

- Replace the starter home/blog content with fastMedium branding.
- Keep the router and shared layout.
- Make the home page show a deliberate empty state.

### Session 2

- Split the router, layout, and home page into modules.
- Add a typed in-memory `Post` model.
- Render a small hard-coded feed.

### Session 3

- Add the post detail route.
- Use a slug rather than the starter integer blog ID.
- Add a not-found state.

### Session 4

- Choose and add SQLite persistence.
- Create the first migration for users and posts.
- Replace hard-coded posts with database reads.

### Session 5 onward

- Add authentication.
- Add the dashboard and publishing flow.
- Add tests before expanding into social features.

## Immediate next task

Implement only this slice:

1. Replace the starter `Hero`, `Blog`, and `Echo` experience with fastMedium terminology.
2. Keep a route for `/` and a route for a post detail page.
3. Add a shared navigation bar with links to Home and Write.
4. Make the home page display an intentional empty feed state, such as “No stories published yet.”
5. Keep the post detail page using temporary in-memory data or a placeholder, because the database is a later phase.
6. Run:

```bash
cargo fmt -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
```

Do not add authentication or a database in this task. Once this slice works, inspect the result in the browser and then move to the in-memory feed task.
