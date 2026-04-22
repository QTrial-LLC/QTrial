-- The dog subtree: the dog record, the titles it has earned, and
-- the sports it currently competes in. All three tables are
-- tenant-scoped through direct club_id per the DATA_MODEL convention;
-- the dog's club_id is denormalized onto dog_titles and
-- dog_sport_participation so RLS on those tables is a single-column
-- check, not a join-to-parent.
--
-- breed_id is deliberately left nullable with no FK constraint this
-- session. The breeds reference table lands next session; a
-- follow-up migration will add the FK and backfill. Same pattern as
-- the winning_entry_id-to-entries FK deferral we've used before.
-- Until that lands, breed_id is treated as an opaque UUID placeholder
-- the app layer does not enforce.
--
-- Sire and dam information is stored as six denormalized text fields
-- (prefix titles, registered name, suffix titles for each parent)
-- rather than a separate pedigree table. AKC registered names are
-- rendered verbatim in catalogs, not walked as a graph; normalizing
-- sire and dam would add joins for zero correctness benefit.

CREATE TYPE dog_sex AS ENUM (
    'male',
    'female',
    'male_neutered',
    'female_spayed'
);

CREATE TABLE dogs (
    id                            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its dog directory; soft delete via deleted_at does not cascade.
    club_id                       UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE RESTRICT: a dog row cannot be orphaned if its owner
    -- is hard deleted. App layer soft-deletes dogs (inactive = true
    -- or deleted_at) before cleaning up owners.
    owner_id                      UUID NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    registry_id                   UUID NOT NULL REFERENCES registries(id),
    -- breed_id: FK to the breeds reference table lands next session.
    -- Nullable + unconstrained UUID placeholder until then. A
    -- follow-up migration will add the constraint and backfill.
    breed_id                      UUID,
    breed_variety_id              UUID,
    breed_division                TEXT,
    call_name                     TEXT,
    sex                           dog_sex,
    registered_name               TEXT,
    registration_number           TEXT,
    registration_country_code     TEXT,
    birthdate                     DATE,
    breeder                       TEXT,
    sire_prefix_titles            TEXT,
    sire_registered_name          TEXT,
    sire_suffix_titles            TEXT,
    dam_prefix_titles             TEXT,
    dam_registered_name           TEXT,
    dam_suffix_titles             TEXT,
    -- Free-text co-owners for catalog rendering. Structured
    -- co-owner records are a P2 consideration per DATA_MODEL §5
    -- pending decision.
    co_owners_text                TEXT,
    jump_height_measured          NUMERIC(4, 1),
    has_jump_height_card          BOOL NOT NULL DEFAULT FALSE,
    -- AKC disciplinary ineligibility. Distinct from per-class
    -- eligibility rules: this blocks every trial entry, not just
    -- certain classes. REQUIREMENTS §16a.1.
    is_akc_ineligible             BOOL NOT NULL DEFAULT FALSE,
    akc_ineligible_reason         TEXT,
    akc_ineligible_recorded_at    TIMESTAMPTZ,
    inactive                      BOOL NOT NULL DEFAULT FALSE,
    created_at                    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                    TIMESTAMPTZ,
    created_by                    UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                    UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT dogs_jump_height_nonneg CHECK (jump_height_measured >= 0),
    -- If a dog is flagged ineligible, there must be a recorded
    -- timestamp. reason is nullable because AKC may not disclose it.
    CONSTRAINT dogs_ineligible_has_recorded_at CHECK (
        (is_akc_ineligible = FALSE AND akc_ineligible_recorded_at IS NULL)
        OR
        (is_akc_ineligible = TRUE AND akc_ineligible_recorded_at IS NOT NULL)
    )
);

CREATE INDEX dogs_club_id_ix ON dogs (club_id) WHERE deleted_at IS NULL;
CREATE INDEX dogs_owner_id_ix ON dogs (owner_id) WHERE deleted_at IS NULL;

-- AKC registration number uniqueness is club-scoped for MVP (cross-
-- tenant dedup deferred). Partial unique excludes null numbers (not
-- yet registered) and soft-deleted rows.
CREATE UNIQUE INDEX dogs_club_registry_number_uk
    ON dogs (club_id, registry_id, registration_number)
    WHERE registration_number IS NOT NULL AND deleted_at IS NULL;


CREATE TYPE dog_title_category AS ENUM (
    'prefix',
    'suffix'
);

CREATE TYPE dog_title_source AS ENUM (
    'owner_entered',
    'registry_verified',
    'earned_in_qtrial'
);

CREATE TABLE dog_titles (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Denormalized club_id for RLS. Per the tenancy convention,
    -- every tenant-scoped table carries its own club_id; the app
    -- layer sets this from the parent dog at insert time. RLS WITH
    -- CHECK enforces it matches current_club_id.
    club_id          UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    dog_id           UUID NOT NULL REFERENCES dogs(id) ON DELETE CASCADE,
    -- title_code is free text until the title_prefixes /
    -- title_suffixes reference data lands next session. A follow-up
    -- migration will convert this to an FK and backfill. Examples:
    -- 'CD', 'CDX', 'OTCH', 'RN', 'RAE2', 'MACH 3'.
    title_code       TEXT NOT NULL,
    title_category   dog_title_category NOT NULL,
    earned_at        DATE,
    source           dog_title_source NOT NULL DEFAULT 'owner_entered',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ,
    created_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by       UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX dog_titles_club_id_ix ON dog_titles (club_id) WHERE deleted_at IS NULL;
CREATE INDEX dog_titles_dog_id_ix ON dog_titles (dog_id) WHERE deleted_at IS NULL;

-- One instance of each title per dog. Repeated titles with numeric
-- increments (RAE2, RAE3, MACH 2, MACH 3) are distinct title_codes
-- and get their own rows.
CREATE UNIQUE INDEX dog_titles_dog_code_uk
    ON dog_titles (dog_id, title_code)
    WHERE deleted_at IS NULL;


CREATE TABLE dog_sport_participation (
    club_id          UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    dog_id           UUID NOT NULL REFERENCES dogs(id) ON DELETE CASCADE,
    sport            sport NOT NULL,
    active           BOOL NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by       UUID REFERENCES users(id) ON DELETE SET NULL,

    PRIMARY KEY (dog_id, sport)
);

-- No deleted_at on dog_sport_participation; participation is a
-- small lookup-style row that gets toggled via `active`. If a dog
-- stops competing in a sport, `active = false` covers the state
-- without needing soft-delete. This is the only table in the entry
-- layer without deleted_at and is called out explicitly.

CREATE INDEX dog_sport_participation_club_id_ix
    ON dog_sport_participation (club_id);
