-- Judge assignments bind a judge to a specific class offering (a
-- canonical class at a specific trial). A single class offering may
-- have multiple judge_assignments rows when a co-judge is present;
-- `is_co_judge` distinguishes the primary from the co-assignment.
--
-- Trial awards are the per-trial recognitions (HIT, HC, RHIT, RHTQ,
-- etc.). Multi-class combined awards (HC across Open B + Utility B;
-- RHTQ across Rally Advanced B + Excellent B + Master) carry an
-- array of `contributing_entry_line_ids` so a marked catalog can
-- render which component scores combined. The FK from
-- `winning_entry_id` into a future `entries` table is deferred until
-- that table lands; the column is a bare UUID here.

CREATE TABLE judge_assignments (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id                     UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of the class offering wipes
    -- its assignments; soft delete via deleted_at does not cascade.
    trial_class_offering_id     UUID NOT NULL REFERENCES trial_class_offerings(id) ON DELETE CASCADE,
    judge_id                    UUID NOT NULL REFERENCES judges(id) ON DELETE RESTRICT,
    is_co_judge                 BOOL NOT NULL DEFAULT FALSE,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                  TIMESTAMPTZ,
    created_by                  UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                  UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX judge_assignments_club_id_ix
    ON judge_assignments (club_id) WHERE deleted_at IS NULL;

CREATE INDEX judge_assignments_class_offering_ix
    ON judge_assignments (trial_class_offering_id) WHERE deleted_at IS NULL;

CREATE INDEX judge_assignments_judge_id_ix
    ON judge_assignments (judge_id) WHERE deleted_at IS NULL;

-- A given judge can be assigned to a class offering at most once,
-- either as primary or as co-judge. Prevents duplicate rows from
-- multiple save attempts.
CREATE UNIQUE INDEX judge_assignments_offering_judge_uk
    ON judge_assignments (trial_class_offering_id, judge_id)
    WHERE deleted_at IS NULL;

CREATE TYPE award_type AS ENUM (
    'hit',
    'hc',
    'phit',
    'phc',
    'rhit',
    'rhc',
    'rhtq',
    'htq'
);

CREATE TABLE trial_awards (
    id                           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id                      UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of the trial wipes its awards;
    -- soft delete via deleted_at does not cascade.
    trial_id                     UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    award_type                   award_type NOT NULL,
    -- FK into `entries` is deferred until that table exists in a
    -- later migration. Column is a bare UUID today; when entries
    -- lands the FK constraint is added as a separate migration.
    winning_entry_id             UUID,
    -- Denormalized armband for catalog printing without a join.
    winning_armband              INT,
    -- Single score for HIT/RHIT; combined score for HC/RHC/RHTQ.
    winning_score                NUMERIC(5, 1),
    -- Entry lines that contributed to a combined-score award. For
    -- HIT/RHIT this is typically a single element; for HC it's two
    -- (Open B + Utility B) and for RHTQ it's three (Advanced B +
    -- Excellent B + Master). No element-level FK is possible with an
    -- array column; app layer validates membership.
    contributing_entry_line_ids  UUID[] NOT NULL DEFAULT '{}',
    notes                        TEXT,
    created_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                   TIMESTAMPTZ,
    created_by                   UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                   UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT trial_awards_armband_positive CHECK (winning_armband >= 1),
    CONSTRAINT trial_awards_score_nonneg CHECK (winning_score >= 0)
);

CREATE INDEX trial_awards_club_id_ix ON trial_awards (club_id) WHERE deleted_at IS NULL;
CREATE INDEX trial_awards_trial_id_ix ON trial_awards (trial_id) WHERE deleted_at IS NULL;

-- GIN on the array supports the "is entry line X a contributor to
-- any award?" query pattern, which scoring and marked-catalog
-- generation will hit frequently. Cheaper to land the index with the
-- table than to add it under load later.
CREATE INDEX trial_awards_contributing_entry_lines_gin
    ON trial_awards USING GIN (contributing_entry_line_ids);

-- One award of a given type per trial. A trial has at most one HIT,
-- one HC, one RHTQ, etc. Partial unique so soft-deleted (corrected
-- or retracted) awards don't block reassignment.
CREATE UNIQUE INDEX trial_awards_trial_type_uk
    ON trial_awards (trial_id, award_type)
    WHERE deleted_at IS NULL;
