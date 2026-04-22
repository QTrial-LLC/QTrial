-- Trials are the leaf competition unit: one sport at one day of an
-- event, with its own AKC event number, entry fees, judge
-- assignments, and awards. A single day can hold multiple trials
-- (AM/PM, or Obedience plus Rally concurrently).
--
-- Per DOMAIN_GLOSSARY, AKC event numbers are issued per-sport-per-
-- day-per-trial: two trials of the same sport on the same day get
-- different numbers, and concurrent Obedience and Rally trials get
-- different numbers too. AKC guarantees global uniqueness in the
-- YYYYNNNNNN format, so the unique index is global (not tenant-
-- scoped). A tenant learning that a specific number is taken is
-- not a meaningful information leak because AKC numbers appear on
-- public premium lists.

CREATE TYPE trial_status AS ENUM (
    'draft',
    'open',
    'closed',
    'running',
    'complete'
);

CREATE TYPE running_order_strategy AS ENUM (
    'short_to_tall',
    'tall_to_short',
    'random',
    'manual',
    'reverse_previous_day'
);

CREATE TABLE trials (
    id                              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id                         UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of parent wipes the subtree;
    -- soft delete via deleted_at does not cascade.
    event_day_id                    UUID NOT NULL REFERENCES event_days(id) ON DELETE CASCADE,
    trial_number                    INT NOT NULL,
    sport                           sport NOT NULL,
    akc_event_number                TEXT,
    trial_chairperson               TEXT,
    -- Official published start time on this trial. Local time-of-day;
    -- the day's date comes from event_days.date.
    start_time                      TIME,
    entry_limit                     INT,
    -- Fee columns are nullable because a draft trial may not have
    -- set prices yet. All enforce non-negative via CHECK.
    first_class_fee                 NUMERIC(10, 2),
    additional_class_fee            NUMERIC(10, 2),
    nonregular_class_fee            NUMERIC(10, 2),
    nonregular_second_class_fee     NUMERIC(10, 2),
    brace_fee                       NUMERIC(10, 2),
    team_fee                        NUMERIC(10, 2),
    rally_pairs_fee                 NUMERIC(10, 2),
    rally_team_fee                  NUMERIC(10, 2),
    first_class_fee_jr              NUMERIC(10, 2),
    additional_class_fee_jr         NUMERIC(10, 2),
    status                          trial_status NOT NULL DEFAULT 'draft',
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                      TIMESTAMPTZ,
    created_by                      UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                      UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT trials_trial_number_positive CHECK (trial_number >= 1),
    CONSTRAINT trials_entry_limit_nonneg CHECK (entry_limit >= 0),
    CONSTRAINT trials_first_class_fee_nonneg CHECK (first_class_fee >= 0),
    CONSTRAINT trials_additional_class_fee_nonneg CHECK (additional_class_fee >= 0),
    CONSTRAINT trials_nonregular_class_fee_nonneg CHECK (nonregular_class_fee >= 0),
    CONSTRAINT trials_nonregular_second_class_fee_nonneg CHECK (nonregular_second_class_fee >= 0),
    CONSTRAINT trials_brace_fee_nonneg CHECK (brace_fee >= 0),
    CONSTRAINT trials_team_fee_nonneg CHECK (team_fee >= 0),
    CONSTRAINT trials_rally_pairs_fee_nonneg CHECK (rally_pairs_fee >= 0),
    CONSTRAINT trials_rally_team_fee_nonneg CHECK (rally_team_fee >= 0),
    CONSTRAINT trials_first_class_fee_jr_nonneg CHECK (first_class_fee_jr >= 0),
    CONSTRAINT trials_additional_class_fee_jr_nonneg CHECK (additional_class_fee_jr >= 0)
);

CREATE INDEX trials_club_id_ix ON trials (club_id) WHERE deleted_at IS NULL;
CREATE INDEX trials_event_day_id_ix ON trials (event_day_id) WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX trials_event_day_trial_number_uk
    ON trials (event_day_id, trial_number)
    WHERE deleted_at IS NULL;

-- Global partial unique on AKC event number. AKC guarantees global
-- uniqueness in the YYYYNNNNNN format; this index matches that
-- reality rather than tenant-scoping the constraint.
CREATE UNIQUE INDEX trials_akc_event_number_uk
    ON trials (akc_event_number)
    WHERE akc_event_number IS NOT NULL AND deleted_at IS NULL;

-- `trial_class_offerings` binds a specific canonical class to a
-- specific trial with per-offering overrides (ring number, class
-- limit, running order, schedule component minutes). The per-trial
-- offering is where most scheduling and capacity knobs live; the
-- canonical class is just the reference definition.
CREATE TABLE trial_class_offerings (
    id                              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id                         UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of parent wipes the subtree;
    -- soft delete via deleted_at does not cascade.
    trial_id                        UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    canonical_class_id              UUID NOT NULL REFERENCES canonical_classes(id),
    -- Default to 1, reject 0. Obedience Solution's legacy "Ring 0"
    -- was a placeholder for unset and should never appear in
    -- QTrial; Phase 7 migration translates it to 1 at import time.
    ring_number                     INT NOT NULL DEFAULT 1,
    class_limit                     INT,
    scheduled_start_time            TIME,
    running_order_strategy          running_order_strategy NOT NULL DEFAULT 'short_to_tall',
    -- Required when strategy is reverse_previous_day, null otherwise.
    -- The CHECK below enforces the coupling.
    running_order_reference_day_id  UUID REFERENCES event_days(id) ON DELETE SET NULL,
    jump_start_height               INT,
    -- Schedule components. Defaults derived from the canonical class
    -- at insert time by the app layer; null permitted so secretaries
    -- can explicitly blank overrides.
    per_dog_minutes                 NUMERIC(4, 1),
    walkthrough_minutes             NUMERIC(4, 1),
    ribbon_presentation_minutes     NUMERIC(4, 1),
    class_transition_minutes        NUMERIC(4, 1),
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                      TIMESTAMPTZ,
    created_by                      UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                      UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT trial_class_offerings_ring_positive CHECK (ring_number >= 1),
    CONSTRAINT trial_class_offerings_class_limit_nonneg CHECK (class_limit >= 0),
    CONSTRAINT trial_class_offerings_per_dog_minutes_nonneg CHECK (per_dog_minutes >= 0),
    CONSTRAINT trial_class_offerings_walkthrough_minutes_nonneg CHECK (walkthrough_minutes >= 0),
    CONSTRAINT trial_class_offerings_ribbon_minutes_nonneg CHECK (ribbon_presentation_minutes >= 0),
    CONSTRAINT trial_class_offerings_transition_minutes_nonneg CHECK (class_transition_minutes >= 0),
    CONSTRAINT trial_class_offerings_reverse_day_requires_reference
        CHECK (
            (running_order_strategy = 'reverse_previous_day'
             AND running_order_reference_day_id IS NOT NULL)
            OR
            (running_order_strategy <> 'reverse_previous_day'
             AND running_order_reference_day_id IS NULL)
        )
);

CREATE INDEX trial_class_offerings_club_id_ix
    ON trial_class_offerings (club_id) WHERE deleted_at IS NULL;

CREATE INDEX trial_class_offerings_trial_id_ix
    ON trial_class_offerings (trial_id) WHERE deleted_at IS NULL;

-- One offering per canonical class per trial: a trial cannot offer
-- the same Novice A twice. Soft-deleted rows are excluded so a
-- previously-cancelled offering can be re-added.
CREATE UNIQUE INDEX trial_class_offerings_trial_class_uk
    ON trial_class_offerings (trial_id, canonical_class_id)
    WHERE deleted_at IS NULL;
