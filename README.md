# OffLeash

Multi-tenant SaaS for managing AKC dog sport trials. Background, scope,
and schedule live under [docs/](./docs/); start with
[docs/README.md](./docs/README.md) and follow the reading order there.

This file is the developer quickstart. If you are trying to understand
the product, read the docs first.

## Repository layout

```
api/           Rust: axum + tokio + sqlx (offleash-api binary)
workers/       Rust: NATS consumer and offline jobs (offleash-workers binary)
shared/        Rust: types and helpers used by both binaries
web/           Next.js 16 (App Router, TypeScript strict, Tailwind v4)
db/migrations/ sqlx-managed SQL migrations
db/seed/       Reference data seeds (AKC classes, breeds, titles)
db/docker-init/ One-shot SQL for the local Postgres container
infra/         Terraform (not populated yet)
scripts/       Developer helpers
docs/          Architecture, requirements, domain glossary
```

## Prerequisites

- Rust toolchain via [rustup](https://rustup.rs). The pinned version in
  `rust-toolchain.toml` is installed automatically the first time you
  run `cargo` in this repo.
- Node.js 20 or newer, npm 10 or newer.
- Docker 25 or newer with the Compose v2 plugin.

## Local stack

Bring up Postgres, Keycloak, NATS, and Valkey, then confirm each one is
actually serving, not just running:

```
docker compose up -d
scripts/smoke-compose.sh
```

Default exposed ports on `localhost`:

| Service  | Port | Notes                                                  |
|----------|------|--------------------------------------------------------|
| Postgres | 5432 | `postgres/postgres` superuser; app DB `offleash/offleash` |
| Keycloak | 8080 | Admin console at `/`, bootstrap admin `admin/admin`    |
| NATS     | 4222 | Client port; monitoring on 8222                        |
| Valkey   | 6379 | No auth                                                |

If another stack on your host (Mediacast Platform, for example) is
already bound to these ports, copy `.env.example` to `.env` and
uncomment the alt-port block. Compose and `scripts/smoke-compose.sh`
read the same `.env` file, so no other changes are needed.

Tear the stack down with `docker compose down`. Add `-v` to also drop
the named volumes and start fresh next time.

## Build and run the Rust crates

From the repository root:

```
cargo check --workspace           # fast feedback while editing
cargo build --workspace           # produce debug binaries
cargo run -p offleash-api         # logs "starting" and exits
cargo run -p offleash-workers     # logs "starting" and exits
cargo fmt --all
cargo clippy --workspace --all-targets
```

Log format defaults to structured JSON. Set `OFFLEASH_LOG_FORMAT=text`
for a human readable format during local work. `RUST_LOG` applies as
usual, for example `RUST_LOG=debug`.

## Run the web app

```
cd web
npm install              # only needed on fresh clones
npm run dev              # http://localhost:3000
npm run build            # production build
npm run lint
```

## What Phase 0 is and is not

Phase 0 is scaffolding only. There is no business logic, no real
schema, no authentication wiring, and no tenant model yet. Every
directory exists so the next phase has somewhere to put its output.
See [docs/ROADMAP.md](./docs/ROADMAP.md) for the phased plan.
