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

## Migration history by PR

A non-exhaustive index of how migrations group into the PRs that
landed them. Useful for orienting against `git log` without reading
every file. The complete list lives in the directory itself; the
PR-by-PR view below gives one-line summaries of the major blocks.

### Phase 0 (PRs #3-#7, 2026-04-22)

Foundational schema across five PRs: registries / akc_fee_schedules
/ canonical_classes plus the migration tooling and up/down convention
(PR #3); clubs / users / user_club_roles with RLS plus the
qtrial_tenant role (PR #4); events / event_days / trials /
trial_class_offerings / judges / judge_assignments / trial_awards
with RLS and the parent_club_id helper (PR #6); owners / dogs /
dog_titles / dog_sport_participation / teams / entries / entry_lines
/ entry_line_results with RLS and the entry-line state machine
(PR #7); the cross-tenant FK validation helper
(`shared/src/fk_validation.rs::TenantTable`, PR #10).

### PR 2a: reference-data foundation (2026-04-23)

14 reference tables (countries, states, breed_groups, breeds,
breed_varieties, title_prefixes, title_suffixes, jump_heights,
obedience_exercises, obedience_class_exercises, otch_points,
om_points, rally_rach_points, sport_time_defaults) with
permissive-read RLS, plus the `qtrial-seed-loader` binary that
populates them idempotently from `db/seed/akc/` CSVs. Anchor
migration: `20260423120700_enable_rls_on_reference_data_foundation.up.sql`.

### PR 2b: tenant-scoped table gap-fill (2026-04-24)

Eight tenant-scoped tables (dog_ownerships, dog_trial_jump_heights,
armband_assignments, email_templates, submission_records, payments,
refunds, audit_log) with direct-club_id RLS, plus `platform_admins`
which is non-tenant and has no qtrial_tenant grant. Two new ENUMs
(`submission_type`, `submission_status`) plus `refund_reason`.
Anchor migration:
`20260424121000_enable_rls_on_tenant_scoped_gap_fill.up.sql`.

### PR 2c-surgery: entry-pipeline reconciliation (2026-04-25)

Six migrations reshaping entries / entry_lines / entry_line_results
plus a dog_title_source ENUM extension. Handler identity moves from
entries to entry_lines (`handler_contact_id`,
`junior_handler_akc_number`); armband routes through
`armband_assignments` via `entry_lines.armband_assignment_id`; jump
height moves to `dog_trial_jump_heights` per (dog, trial). Adds
`entry_line_results` timing columns and `rach_points`. Files
`20260425120000` through `20260425120500`.

### PR 2c-beta: dogs reconciliation (2026-04-26)

Five migrations reconciling the dogs table against DATA_MODEL §4.
Adds `registration_type` ENUM, four name-parser columns
(`parsed_name_root`, `parsed_prefix_titles`, `parsed_suffix_titles`,
`unparsed_title_tokens`), and FK constraints on `breed_id` and
`breed_variety_id` deferred from Phase 0. Drops `co_owners_text`
(superseded by `dog_ownerships`) and the four sire/dam
prefix/suffix sub-columns (sire and dam parse at display time
against the same title catalog as the dog's own name). Files
`20260426120000` through `20260426120400`.

### PR 2d: events / clubs / awards plumbing (2026-04-26)

Nine migrations + two seed CSVs landing Deborah's 2026-04-23
plumbing. Files `20260426120500` through `20260426121300`:

| Filename | Description |
|---|---|
| `20260426120500_add_events_mixed_breeds_allowed` | `events.mixed_breeds_allowed BOOL NOT NULL DEFAULT TRUE`. Per the 2026-04-26 Decisions-log "BOOL only, defer breed-list" scope-lock. |
| `20260426120600_add_events_trial_chair_and_secretary_fks` | `events.trial_chair_user_id` and `events.event_secretary_user_id`, FK to users with ON DELETE SET NULL, both nullable; partial indexes on `deleted_at IS NULL`. Per Q5. |
| `20260426120700_drop_trials_trial_chairperson` | Drops the unused Phase 0 free-text column; replaced by the event-level FK in the previous migration. Down recreates as TEXT NULL. |
| `20260426120800_add_events_dogs_per_hour_override` | `events.dogs_per_hour_override JSONB`, nullable. Keys are `canonical_classes.code` strings; values are minutes-per-dog. App-layer validation enforces key resolution (CHECK cannot reference other tables); migration header carries the TODO. |
| `20260426120900_extend_armband_scheme_per_series` | `ALTER TYPE armband_scheme ADD VALUE IF NOT EXISTS 'per_series'`. The `IF NOT EXISTS` guard satisfies the round-trip contract under the project's no-op-down ENUM policy (see the 2026-04-25 Decisions-log "Postgres ENUM additions are one-way" entry). |
| `20260426121000_add_clubs_officers_json` | `clubs.officers_json JSONB`, nullable. Array-of-records shape; serde-typed at the app layer; no DDL CHECK. Per Q6. |
| `20260426121100_add_trial_class_offerings_judges_book_columns` | `pre_trial_blank_pdf_object_key` and `signed_scan_pdf_object_key`, both nullable TEXT. Two-column shape per the 2026-04-26 Decisions-log entry "PR 2d: judges-book PDF storage uses two columns, not one overwriting column" (revising the 2026-04-24 working assumption). |
| `20260426121200_create_combined_award_groups` | Creates `combined_award_groups` (parent) and `combined_award_group_classes` (junction). Reference data, registry-scoped. RLS lands in the next migration. |
| `20260426121300_enable_rls_on_combined_award_groups` | Permissive-read RLS plus SELECT-only grant to qtrial_tenant for both new tables. Pattern matches the PR 2a reference-table RLS migration. |

Non-migration deliverables landed alongside:

- `db/seed/akc/regulations/akc_rally_regulations_1217.pdf` and
  `db/seed/akc/regulations/akc_obedience_regulations_2025_03.pdf` -
  frozen citation sources for migration headers and the seed CSV
  citations. Verified URLs at `images.akc.org` on 2026-04-25.
- `db/seed/akc/combined_award_groups.csv` (5 rows) and
  `db/seed/akc/combined_award_group_classes.csv` (12 rows) - loaded
  by the seed loader extension in workers/src/seed_loader/. Citations
  reference the regulation PDFs above.
- DATA_MODEL.md bumped to v0.4.
