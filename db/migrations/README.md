# Database migrations

QTrial uses sqlx-managed SQL migrations. Every schema change lands as
a matched `.up.sql` / `.down.sql` pair in this directory, and both
dev and CI apply them via `sqlx migrate run`. This README is the
working contract: future contributors (and future Claude Code
sessions) should be able to follow it without reading every prior
migration.

## Naming

    YYYYMMDDHHMMSS_description.up.sql
    YYYYMMDDHHMMSS_description.down.sql

- 14-digit timestamp prefix; sqlx applies migrations in strict
  timestamp order.
- Within one day, increment the last four digits by **100** per
  migration (not by 1). This leaves room to slot a dependent
  migration between two existing ones without renumbering.
- `description` is a short snake_case summary of what the migration
  does. "create_countries_and_states", "enable_rls_on_event_setup",
  "tighten_clubs_with_check".

## Up and down are both required

Every `.up.sql` has a matching `.down.sql` that reverses its changes.
Reversibility is non-negotiable for this project:

- Dev iteration: `sqlx migrate revert` backs out the most recent
  migration so you can refine its contents and reapply.
- CI verification: the CI round-trip test (not enabled yet, but
  planned) applies every up, then every down in reverse, then every
  up again. A migration that does not round-trip surfaces early.

Down migrations drop in **reverse** of the up: if up creates tables
A, B, C in order, down drops C, B, A. Guard DROPs with `IF EXISTS`
so a partial earlier failure does not make the down migration itself
fail.

## Creating a new migration

    sqlx migrate add -r <description>

The `-r` flag emits a reversible (up/down) pair with the current
timestamp. Edit both files; the down is not optional.

Bump the timestamp by 100 seconds (or more) over the latest existing
migration if the `-r` default collides or falls inside an already-
taken slot.

## Running migrations

Local (against the dev Compose Postgres):

    export DATABASE_URL=postgres://qtrial:qtrial@localhost:5432/qtrial
    sqlx migrate run --source db/migrations

Verify a new migration round-trips before committing:

    sqlx migrate run     # apply
    sqlx migrate revert  # undo the just-applied one
    sqlx migrate run     # reapply

If revert fails or leaves stranded objects, the down is wrong; fix
before committing. If reapply fails, the up has a non-idempotent side
effect; fix before committing.

CI applies migrations from scratch on every run against a fresh
Postgres 16 service container. A migration that relies on local dev
state (uncommitted types, orphaned rows) will fail there first.

## Migrations vs seed data

- **Migrations** create and alter schema: tables, indexes, constraints,
  ENUM types, RLS policies, grants. They also seed values that are
  authoritative to the repo (registry codes, canonical class catalog,
  AKC fee schedules).
- **The seed loader** (`cargo run -p qtrial-workers --bin qtrial-seed-loader`)
  populates registry-scoped reference tables from `db/seed/akc/` CSVs:
  countries, states, breed catalog, title catalog, jump heights,
  obedience exercises + class layout, scoring lookups, sport time
  defaults. The loader is idempotent and re-running it with the same
  CSV inputs is a no-op.

Rule of thumb: if the data shape or set is versioned with the code
(and a schema change to ship a new value is acceptable), it belongs
in a migration. If the data is an external reference source we expect
to refresh periodically (AKC's breed catalog, title catalog, fee
schedule), it belongs in a seed CSV and is loaded after migrations
complete.

## RLS conventions

Every table falls into one of two categories:

- **Reference tables** (shared across tenants): `registries`,
  `akc_fee_schedules`, `canonical_classes`, plus the 14 reference
  tables added in PR 2a. Each gets:

        ALTER TABLE <table> ENABLE ROW LEVEL SECURITY;
        CREATE POLICY <table>_read_all ON <table>
            FOR SELECT USING (TRUE);
        GRANT SELECT ON <table> TO qtrial_tenant;

  No INSERT/UPDATE/DELETE grant and no corresponding policy means
  tenants cannot write; the `qtrial` table owner still can, which is
  how the seed loader and admin tooling operate.

- **Tenant tables** (per-club data): carry a `club_id UUID NOT NULL
  REFERENCES clubs(id)` column. RLS policies filter rows by
  `current_setting('app.current_club_id')::uuid`. Grants allow full
  CRUD to `qtrial_tenant`; the policy does the row-level gating.

RLS policies land in grouped `enable_rls_on_*` migrations rather
than one per table. Examples in the repo:

- `20260419130300_enable_rls_and_grants.up.sql` (tenancy: clubs,
  users, user_club_roles, plus the first three reference tables)
- `20260419140500_enable_rls_on_event_setup.up.sql`
- `20260420120700_enable_rls_on_entry_layer.up.sql`
- `20260423120700_enable_rls_on_reference_data_foundation.up.sql`
  (the 14 reference tables from PR 2a)

## Role separation

Two roles, created by `db/docker-init/01-create-databases.sql`:

- `qtrial` (LOGIN, owns every table, bypasses RLS by Postgres
  convention). Migrations run as this role. The seed loader also
  runs as this role.
- `qtrial_tenant` (NOLOGIN, assumed via `SET LOCAL ROLE
  qtrial_tenant` inside a transaction). API request paths downshift
  to this role so RLS policies apply.

Platform-admin paths run as `qtrial` and deliberately bypass RLS;
those code paths must log every access.

## When a migration needs to change

Do not edit a committed migration. Add a new one instead. Editing
an applied migration is almost always wrong:

- Other developers' local databases already applied the old text; a
  silent change leaves their schema in an undefined state relative
  to the new text.
- CI restores from cache in ways that can mask drift.

The one exception: fixing an up-only migration before anyone else
has pulled. If you are sure no one else has applied the migration,
`sqlx migrate revert` locally, edit, and reapply is acceptable.
Anything beyond that: write a new migration.
