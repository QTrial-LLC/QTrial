-- Scoring lookup tables: otch_points, om_points, rally_rach_points.
--
-- Three class-x-score lookup tables that the awards engine consults
-- when computing OTCH points (placement-based), OM points (score-based,
-- Obedience Master), and RACH points (score-based, Rally
-- Championship).
--
-- Per Deborah's Q3 (2026-04-20), QTrial stores only the current AKC
-- rules. Historical pre-2019 tables are out of scope: exhibitors claim
-- titles based on AKC's authoritative records, and QTrial trusts the
-- handler's claim when it comes to cross-system title history.
-- Meaning: we recompute points for trials we run, we don't backdate.
--
-- Natural keys:
--   otch_points: (class_name, entries_min, entries_max) - each class
--       has a non-overlapping set of entry-count brackets.
--   om_points and rally_rach_points: (score) alone - each qualifying
--       score maps to one point value.

CREATE TABLE otch_points (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    legacy_id            INT,
    -- Free text rather than an FK to canonical_classes because AKC's
    -- OTCH rules apply only to Open B and Utility B at MVP, and the
    -- lookup is performed by class name in the awards-compute path.
    -- A future ALTER can swap this for canonical_class_id when the
    -- OTCH calculator is written.
    class_name           TEXT NOT NULL,
    entries_min          INT NOT NULL,
    entries_max          INT NOT NULL,
    first_place_points   INT NOT NULL DEFAULT 0,
    second_place_points  INT NOT NULL DEFAULT 0,
    third_place_points   INT NOT NULL DEFAULT 0,
    fourth_place_points  INT NOT NULL DEFAULT 0,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT otch_points_bracket_ordered CHECK (entries_min <= entries_max),
    CONSTRAINT otch_points_first_nonneg   CHECK (first_place_points >= 0),
    CONSTRAINT otch_points_second_nonneg  CHECK (second_place_points >= 0),
    CONSTRAINT otch_points_third_nonneg   CHECK (third_place_points >= 0),
    CONSTRAINT otch_points_fourth_nonneg  CHECK (fourth_place_points >= 0)
);

CREATE UNIQUE INDEX otch_points_class_bracket_uk
    ON otch_points (class_name, entries_min, entries_max);

CREATE TABLE om_points (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    legacy_id   INT,
    -- 190.0 to 200.0 in 0.5-point increments; NUMERIC(4,1) matches
    -- DATA_MODEL.md §8 and the score grid used on the scoring UI.
    score       NUMERIC(4, 1) NOT NULL,
    om_points   INT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT om_points_score_nonneg  CHECK (score >= 0),
    CONSTRAINT om_points_points_nonneg CHECK (om_points >= 0)
);

CREATE UNIQUE INDEX om_points_score_uk ON om_points (score);

CREATE TABLE rally_rach_points (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    legacy_id    INT,
    -- NUMERIC(4,1) despite the MVP values being whole scores (91.0
    -- through 100.0). Rally has not historically used fractional
    -- scores, but the CSV emits "91.0" rather than "91"; storing as
    -- NUMERIC avoids lossy CSV-to-INT coercion.
    score        NUMERIC(4, 1) NOT NULL,
    rach_points  INT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT rally_rach_points_score_nonneg  CHECK (score >= 0),
    CONSTRAINT rally_rach_points_points_nonneg CHECK (rach_points >= 0)
);

CREATE UNIQUE INDEX rally_rach_points_score_uk ON rally_rach_points (score);
