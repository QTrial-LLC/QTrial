-- Canonical class catalog.
--
-- The master list of every competition class OffLeash supports per
-- registry per sport. Per-event class offerings (trial_class_offerings,
-- created in a later migration) FK here so a class definition change
-- lands in one place. Registry-scoped reference data; no tenant
-- isolation (every club sees the same class catalog).
--
-- The three enums below are used by other tables too. `sport` is the
-- per-sport discriminator used on `trials`, `dog_sport_participation`,
-- and anywhere else we talk about sport. `canonical_class_type` is
-- AKC's regulatory category. `ab_eligibility_rule` encodes the
-- fundamentally different A/B entry rules between Obedience
-- (handler-based) and Rally (dog-based, plus extra handler
-- restrictions at Advanced and Excellent).
--
-- MVP enum values only include what we seed this session. Adding
-- Conformation/Agility/ScentWork/etc. happens via ALTER TYPE in later
-- migrations when those sports land.
--
-- Natural unique key: (registry_id, sport, code).

CREATE TYPE sport AS ENUM (
    'obedience',
    'rally'
);

CREATE TYPE canonical_class_type AS ENUM (
    'regular',
    'optional_titling',
    'preferred',
    'nonregular'
);

CREATE TYPE ab_eligibility_rule AS ENUM (
    'handler_based',
    'dog_based',
    'dog_and_handler_based',
    'none'
);

CREATE TABLE canonical_classes (
    id                           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id                  UUID NOT NULL REFERENCES registries(id),
    sport                        sport NOT NULL,
    code                         TEXT NOT NULL,
    display_name                 TEXT NOT NULL,
    class_type                   canonical_class_type NOT NULL,
    -- legacy_class_code maps to tblkAKCObedClassInfo.ClassCode in
    -- Deborah's Access schema. Nullable for post-Access classes, most
    -- notably Rally Choice (added by AKC 2023-06-29). Backfill from
    -- the .mde when Phase 7 migration tooling lands.
    legacy_class_code            INT,
    is_sanctioned                BOOL NOT NULL DEFAULT TRUE,
    has_jumps                    BOOL NOT NULL,
    has_broad_jump               BOOL NOT NULL DEFAULT FALSE,
    has_multiple_entries_per_dog BOOL NOT NULL DEFAULT FALSE,
    total_score                  INT NOT NULL,
    min_qualifying_score         INT NOT NULL,
    dogs_per_hour_default        INT,
    has_walkthrough              BOOL NOT NULL DEFAULT FALSE,
    -- Default walkthrough minutes; overrideable per trial via
    -- trial_class_offerings.walkthrough_minutes. Null for classes
    -- without a walkthrough.
    default_walkthrough_minutes  NUMERIC(4, 1),
    -- Suffix title code this class can earn legs toward. Null for
    -- classes that do not confer a title (nonregular classes if we
    -- add them later).
    qualifies_for_title_code     TEXT,
    -- Number of qualifying scores (legs) needed to earn the title.
    -- Typically 3; 10 for Rally Master (RM) and Rally Choice (RC).
    legs_required_for_title      INT,
    -- Structured title earning rules (e.g., "three qualifying scores
    -- under at least two different judges", consecutive-Q requirements,
    -- cross-class constraints for championship titles). JSONB schema
    -- is deliberately left undesigned this migration; rows are seeded
    -- with NULL and filled when title-progression tracking lands.
    title_earning_rule           JSONB,
    -- A/B eligibility rule for this class. `handler_based` is
    -- Obedience A; `dog_based` is Rally Novice A; `dog_and_handler_based`
    -- is Rally Advanced A and Excellent A; `none` is every B class and
    -- every single-class class (Rally Intermediate, Master, Choice,
    -- Graduate Novice, etc.).
    ab_eligibility_rule          ab_eligibility_rule NOT NULL DEFAULT 'none',
    -- The title code whose presence disqualifies a dog or handler from
    -- this class's A-class entry. NULL when ab_eligibility_rule is
    -- 'none'.
    ab_eligibility_title_code    TEXT,
    -- Self-FK retained for migration compatibility with the Access
    -- schema's "Transfer to X" rows. OffLeash models transfer intent
    -- on entry_lines, not via parallel class rows, so this field stays
    -- NULL in modern seed data.
    parent_class_id              UUID REFERENCES canonical_classes(id),
    created_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX canonical_classes_registry_sport_code_uk
    ON canonical_classes (registry_id, sport, code);
