-- entry_line_results holds the scoring record for a single class
-- run, one row per entry_line after the class is judged. The
-- judge_annotation_text field carries the free-text reason the
-- marked catalog prints next to NQ, Exc, or DQ entries ("left
-- ring", "unmanageable", "lack of teamwork"). It is distinct from
-- entry_lines.status_reason, which is the status-level reason code
-- attached to the state transition; the annotation text is what a
-- reader sees in the catalog.
--
-- Score ranges are deliberately NOT enforced by CHECK: the maximum
-- varies by class (200 for Obedience, 100 for Rally), which would
-- require joining canonical_classes via trial_class_offerings
-- inside the CHECK, and Postgres does not allow subqueries in
-- CHECK constraints. App layer validates score range against the
-- class total.
--
-- The placement-requires-qualifying CHECK enforces the rule that
-- only Q dogs get placements. Nonregular classes have rare
-- exceptions to this that are explicitly allowed at the app layer
-- via specific overrides, not via a permissive CHECK here.

CREATE TYPE qualifying_status AS ENUM (
    'q',
    'nq',
    'na'
);

CREATE TABLE entry_line_results (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its results; soft delete via deleted_at does not cascade.
    club_id                UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of the entry line wipes the
    -- scoring record. A result cannot exist without its line.
    entry_line_id          UUID NOT NULL REFERENCES entry_lines(id) ON DELETE CASCADE,
    score                  NUMERIC(5, 1),
    time_seconds           NUMERIC(7, 2),
    qualifying             qualifying_status NOT NULL,
    placement              INT,
    otch_points            INT,
    om_points              NUMERIC(5, 1),
    judge_annotation_text  TEXT,
    entered_at             TIMESTAMPTZ,
    -- ON DELETE SET NULL: preserve the score record if the scoring
    -- user is purged. Audit trail remains.
    entered_by_user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at             TIMESTAMPTZ,
    created_by             UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by             UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT entry_line_results_score_nonneg
        CHECK (score >= 0),
    CONSTRAINT entry_line_results_time_seconds_nonneg
        CHECK (time_seconds >= 0),
    CONSTRAINT entry_line_results_otch_points_nonneg
        CHECK (otch_points >= 0),
    CONSTRAINT entry_line_results_om_points_nonneg
        CHECK (om_points >= 0),
    CONSTRAINT entry_line_results_placement_range
        CHECK (placement IS NULL OR placement BETWEEN 1 AND 4),
    -- Only qualifying dogs get placements. Nonregular class
    -- exceptions are handled at the app layer via explicit
    -- overrides, not by weakening this CHECK.
    CONSTRAINT entry_line_results_placement_requires_q
        CHECK (placement IS NULL OR qualifying = 'q'),
    -- entered_at and entered_by_user_id travel together; either
    -- both set or both null. A result row that records "when"
    -- without "who" is an audit gap.
    CONSTRAINT entry_line_results_entered_by_and_at_together CHECK (
        (entered_at IS NULL AND entered_by_user_id IS NULL)
        OR
        (entered_at IS NOT NULL AND entered_by_user_id IS NOT NULL)
    )
);

CREATE INDEX entry_line_results_club_id_ix
    ON entry_line_results (club_id) WHERE deleted_at IS NULL;

-- One result per line. Partial unique excludes soft-deleted rows
-- so a miss-scored result can be soft-deleted and a new one
-- recorded.
CREATE UNIQUE INDEX entry_line_results_entry_line_uk
    ON entry_line_results (entry_line_id)
    WHERE deleted_at IS NULL;
