-- Jump-height reference table.
--
-- Per-registry per-sport catalog of legal jump heights. A dog entering
-- a class jumps at one of these heights (dog_trial_jump_heights, PR 2c);
-- the judges book renders the class's jump height in the page header;
-- the AKC XML submission path (post-MVP) needs the AKC secondary class
-- code ("4INCHES", "8INCHES", etc.) for each height.
--
-- height_inches is NUMERIC(4,1) per DATA_MODEL.md §8. The 2026 seed
-- values are all whole inches (4, 8, 10, 12, ..., 36), but the column
-- shape leaves room for half-inch heights without a later ALTER.
--
-- The jumps.csv source row for Jump=0 (Obedience on-leash class
-- sentinel) is intentionally skipped by the loader: on-leash classes
-- do not consult jump_heights, and representing "no jump" as a 0-inch
-- row would pollute height queries and UI dropdowns.
--
-- akc_secondary_class_code is nullable. The 2026 seed leaves every row
-- NULL because the authoritative per-height code list lives in
-- db/seed/akc/post_mvp/akc_xml_jump_heights.csv, which is deferred
-- with the rest of the XML submission workstream (Agility, post-MVP).
-- When that workstream lands, a future seed pass will fill this column
-- idempotently via ON CONFLICT on the natural key.

CREATE TABLE jump_heights (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id               UUID NOT NULL REFERENCES registries(id),
    -- Legacy JumpID from Deborah's Access database (tblAKCObedienceJumps).
    legacy_id                 INT,
    sport                     sport NOT NULL,
    height_inches             NUMERIC(4, 1) NOT NULL,
    -- AKC's internal code for XML submission payloads, e.g. "4INCHES".
    -- Post-MVP (Agility XML) will backfill; MVP rows leave it NULL.
    akc_secondary_class_code  TEXT,
    -- Catalog display order within a sport (jumps.csv NewOrder column).
    -- Full-rank ordering across both Obedience and Rally heights.
    display_order             INT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX jump_heights_registry_sport_height_uk
    ON jump_heights (registry_id, sport, height_inches);
CREATE INDEX jump_heights_registry_id_ix ON jump_heights (registry_id);
