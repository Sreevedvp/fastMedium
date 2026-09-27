# fastMedium

An editorial writing app with a Dioxus 0.7 browser interface and an independent
Axum JSON API running on Tokio. Dioxus Fullstack and server-function macros are
not used. SQLite accounts, sessions, and stories remain in the existing database.

## Run both services

Prerequisites: Rust/Cargo, the `wasm32-unknown-unknown` target, Dioxus CLI 0.7.10,
and Python 3 (used by the launcher and API regression checks).

```sh
rustup target add wasm32-unknown-unknown
# If dx is not already installed:
cargo install dioxus-cli --version 0.7.10 --locked
cp .env.example .env  # only when .env does not already exist
./dev.sh
```

Open http://127.0.0.1:8080. The API runs at http://127.0.0.1:8082/api/health.
The launcher builds the backend, waits for database/API readiness, and starts
the Dioxus browser development server. Ctrl+C stops both process groups.
It reports occupied ports rather than killing unrelated processes.
Dioxus watches frontend changes; restart `./dev.sh` after backend changes.

```sh
WEB_PORT=8090 API_PORT=8092 ./dev.sh
```

Keep the browser on `127.0.0.1` or `localhost`; the API client uses the same
hostname so session cookies work. CORS permits only these local origins on the
configured web port. This setup binds to loopback for local development.

## Configuration

The API reads `.env` at the project root. Existing environment variables win.
`DATABASE_URL` defaults to `sqlite:fastmedium.db`. The launcher always runs from
the project root so restarting it does not create a database in another folder.
Existing SQLite files are opened in place; schema creation is idempotent.

SMTP settings remain optional. Placeholder credentials skip email delivery.
No credentials are embedded in the browser build. Do not commit `.env` or SQLite files.
`WEB_PORT`/`API_PORT` should be exported when using the launcher; it passes the
same configuration to both services. `FASTMEDIUM_API_URL` is an optional
compile-time client override for a separately deployed API; deployment also
requires configuring trusted origins and HTTPS/Secure cookies appropriately.

## Architecture

- `src/main.rs`, `src/pages`, `src/layouts`: client-rendered Dioxus UI.
- `src/client`: typed HTTP requests with browser credentials included.
- `src/bin/api.rs`: Tokio runtime, listener, and graceful shutdown.
- `src/server/api.rs`: explicit Axum routes and local CORS configuration.
- `src/server/auth.rs`: Argon2 passwords and HttpOnly SQLite-backed sessions.
  Password hashing/verification runs in Tokio blocking tasks.
- `src/server/posts.rs`, `profiles.rs`: application operations and ownership checks.
- `src/api_types.rs`, `src/models`: shared JSON contracts.

## API

| Method | Route | Purpose |
|---|---|---|
| GET | `/api/health` | Readiness |
| POST | `/api/auth/register`, `/api/auth/login`, `/api/auth/logout` | Session actions |
| GET | `/api/auth/me` | Current account |
| GET / POST | `/api/posts` | Public feed / create private draft |
| GET | `/api/posts/mine` | Current author's stories |
| GET | `/api/posts/by-slug/{slug}` | Published story |
| GET / PUT / DELETE | `/api/posts/{id}` | Read/edit/delete own story |
| POST | `/api/posts/{id}/publish`, `/api/posts/{id}/unpublish` | Publication state |
| GET | `/api/profiles/{username}` | Public profile |
| GET | `/api/profiles/{username}/posts` | Author's published stories |

## Verify

```sh
cargo check --bin fast-medium --target wasm32-unknown-unknown --no-default-features --features web
cargo build --bin fast-medium-api --no-default-features --features server
python3 scripts/check_api.py
```

The API check starts its own server on an unused port with a temporary database,
disables outgoing email, exercises auth and story permissions, and removes its
fixture data on exit. It never changes `fastmedium.db`.

See `ISSUES.md` for completed work, verification evidence, and remaining scope.
The empty feed includes a labeled design-preview essay adapted from the supplied
reference; it is not inserted into the publication database. The connected-ideas
illustration is decorative, not a live analytics or Notion integration.
