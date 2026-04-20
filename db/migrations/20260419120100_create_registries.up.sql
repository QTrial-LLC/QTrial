-- Registry reference table.
--
-- A registry is the sanctioning body that issues event numbers, defines
-- classes, and accepts results submissions. MVP ships with AKC as the
-- only seeded registry; UKC lands when P2 work begins. Registry-scoped
-- reference data (canonical_classes, akc_fee_schedules, title codes,
-- breeds) all FK here so we never have to backfill the "which registry
-- is this?" question after adding a second one.
--
-- Natural unique key: `code`. The UUID `id` is the stable key every
-- other table FKs to.

CREATE TABLE registries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code        TEXT NOT NULL,
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX registries_code_uk ON registries (code);
