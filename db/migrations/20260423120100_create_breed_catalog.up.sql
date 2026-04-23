-- Breed catalog reference tables: breed_groups, breeds, breed_varieties.
--
-- Breeds are registry-scoped (AKC and UKC group the same physical
-- breeds differently, and some breeds are recognized by one registry
-- and not another) so every breed row FKs to registries. Group numbers
-- 1-11 reflect AKC's variety/miscellaneous taxonomy current through 2026.
--
-- breeds.default_height_inches is Deborah's Access-era entry-form
-- autofill: the default jump height for a breed at most trials. The
-- authoritative per-dog jump height is on dog_trial_jump_heights
-- (lands in PR 2c); this column exists so entry forms can prefill
-- sensibly when a new dog is added. Nullable because post-v1 breeds
-- from Deborah's CSV do not all have a height recorded.
--
-- breed_varieties exists only for the ~19 breeds that compete by
-- variety under AKC rules (Poodles, Dachshunds, Cocker Spaniels,
-- Chihuahuas, Beagles, Bull Terriers, Collies, Manchester Terriers).
-- A breed with has_variety = TRUE has at least one breed_varieties
-- row; a breed with has_variety = FALSE has zero.

CREATE TABLE breed_groups (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id            UUID NOT NULL REFERENCES registries(id),
    -- Legacy GroupID from Deborah's Access database (tblAKCGroups).
    -- Preserved for migration continuity; new rows may leave it NULL.
    legacy_id              INT,
    group_number           INT NOT NULL,
    display_name           TEXT NOT NULL,
    -- AKC registration-number letter prefixes for breeds in this group,
    -- e.g. {"SN","SR","SS"} for Sporting. Stored as TEXT[] because a
    -- group can have multiple prefixes; the seed CSV packs them as a
    -- pipe-delimited string that the loader splits.
    registration_prefixes  TEXT[] NOT NULL DEFAULT '{}',
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX breed_groups_registry_group_uk
    ON breed_groups (registry_id, group_number);
CREATE INDEX breed_groups_registry_id_ix ON breed_groups (registry_id);

CREATE TABLE breeds (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id            UUID NOT NULL REFERENCES registries(id),
    breed_group_id         UUID NOT NULL REFERENCES breed_groups(id),
    -- Legacy BreedID from Deborah's Access database (tblBreeds). Used
    -- by the variety loader to resolve breed_id from breed_legacy_id.
    legacy_id              INT,
    display_name           TEXT NOT NULL,
    -- Short abbreviation used on catalog rows where space is tight.
    -- Most seed rows equal the display_name, but a handful differ
    -- (e.g. "Tibetan Mastiff" abbreviates as "Tibet. Mastiff").
    abbreviation           TEXT,
    -- Default jump height for entry-form autofill. The per-dog
    -- authoritative value lives on dog_trial_jump_heights (PR 2c).
    default_height_inches  INT,
    -- Giant breeds (Great Danes, Great Pyrenees, etc.) drive a
    -- different jumping bracket and show up on premium lists with
    -- specialty notes; flag is used by the schedule generator.
    is_giant               BOOL NOT NULL DEFAULT FALSE,
    -- Three-quarters flag marks a narrow set of breeds whose default
    -- jump height computes from 0.75 * wither height rather than
    -- full-height tables. Carried over from Deborah's schema.
    is_three_quarters      BOOL NOT NULL DEFAULT FALSE,
    has_variety            BOOL NOT NULL DEFAULT FALSE,
    -- has_division separates breeds that compete by a non-variety
    -- division (e.g. Dachshund Longhaired/Smooth/Wirehaired is a
    -- variety; some UKC breeds use "division" terminology instead).
    has_division           BOOL NOT NULL DEFAULT FALSE,
    display_order          INT,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX breeds_registry_name_uk
    ON breeds (registry_id, display_name);
CREATE INDEX breeds_registry_id_ix ON breeds (registry_id);
CREATE INDEX breeds_breed_group_id_ix ON breeds (breed_group_id);

CREATE TABLE breed_varieties (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    breed_id       UUID NOT NULL REFERENCES breeds(id),
    legacy_id      INT,
    display_name   TEXT NOT NULL,
    display_order  INT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX breed_varieties_breed_name_uk
    ON breed_varieties (breed_id, display_name);
CREATE INDEX breed_varieties_breed_id_ix ON breed_varieties (breed_id);
