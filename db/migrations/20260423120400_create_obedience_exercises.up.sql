-- Obedience exercise catalog and per-class exercise layout.
--
-- obedience_exercises is the 20-row master list of scored exercises
-- (Heel on Leash, Figure Eight, Drop on Recall, ...). These are the
-- atomic building blocks of an Obedience judges book page.
--
-- obedience_class_exercises is the normalized junction that tells the
-- judges-book generator "for class X (pattern_variant V), box N is
-- exercise E worth P points, or it is a non-scored rollup row labeled
-- L". A judges book page has 13 boxes in a grid; each box is either a
-- scored exercise (columns 1..K) or a rollup row near the bottom
-- (Subtotal of Points Off, Maximum Score (200), POINTS OFF (Subtract),
-- Total Score, FINAL QUALIFYING SCORE).
--
-- Why not a wide Box1..Box13 layout on canonical_classes?
--
-- (1) The box count varies by class (Novice has 12, Utility has 13,
--     Beginner Novice has 11). A wide layout wastes columns or needs
--     per-class special casing.
-- (2) Some classes have pattern variants: Open B and Utility B are
--     scored in six randomized orderings (Open B I through VI, Utility
--     B I through VI). Normalization lets one class carry six rows
--     per box slot without schema gymnastics.
-- (3) The judges-book generator already iterates; a junction table
--     maps directly to "for each row in (class, pattern, order)".
--
-- Cells vs exercises:
--
-- The seed CSV packs each row's 13 boxes as free-text cells. The
-- loader parses each cell:
--   * If the cell text matches "(...) (<N> pts)" (DOTALL regex), it is
--     a scored exercise, max_points = N.
--   * If the scored cell's name matches an obedience_exercises row
--     exactly, the junction row carries the FK.
--   * If the scored cell name does not match (compound cells like
--     "Heel on Leash & Figure Eight", sub-numbered variants like
--     "Scent Discrimination #1", or Random-Reward placeholders like
--     "#1"), the junction row carries max_points plus box_label = raw
--     cell text. obedience_exercise_id is NULL.
--   * If the cell text does not match the "(N pts)" pattern, it is a
--     rollup / header label. obedience_exercise_id and max_points are
--     both NULL; box_label holds the cell text.
--
-- Two CHECKs enforce the invariants:
--   * At least one of obedience_exercise_id, max_points, box_label is
--     NOT NULL (no empty rows).
--   * If obedience_exercise_id is set, max_points must also be set
--     (a linked exercise is always a scored exercise).
--
-- pattern_variant defaults to 1 for classes with a single judges book
-- layout. Open B and Utility B load six variants each (1 through 6),
-- corresponding to the randomly assigned exercise-order patterns that
-- the trial secretary selects at run time. Random-Reward placeholder
-- rows in the CSV (Open B, Utility B, Preferred Open, Preferred
-- Utility) are NOT loaded: their box cells are "#1..#6" placeholders
-- with no concrete exercise content; the variant rows above carry the
-- real data for Open B and Utility B. Preferred Open and Preferred
-- Utility have published AKC exercise lists that the seed CSV does
-- not yet contain (source:
-- https://www.akc.org/sports/obedience/getting-started/classes/); a
-- later PR will extend obedience_exercises with "Command
-- Discrimination" and "Stand Stay - Get Your Leash" and add the
-- missing Preferred class layouts. GFKC does not offer Preferred
-- classes, so MVP ships without judges book support for them.

CREATE TABLE obedience_exercises (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Legacy ExerciseID from Deborah's Access database
    -- (tblAKCObedienceExercises). Preserved for migration continuity.
    legacy_id     INT,
    display_name  TEXT NOT NULL,
    -- Long-form description is reserved for the exercise help surface
    -- (tooltip on the scoring UI, premium list footnote). Seed rows
    -- leave it NULL; later content work will populate.
    description   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX obedience_exercises_display_name_uk
    ON obedience_exercises (display_name);

CREATE TABLE obedience_class_exercises (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canonical_class_id     UUID NOT NULL REFERENCES canonical_classes(id),
    -- Nullable because compound exercise cells, sub-numbered variants,
    -- and rollup rows do not map to a single obedience_exercises row.
    obedience_exercise_id  UUID REFERENCES obedience_exercises(id),
    -- 1 for single-pattern classes. Open B I..VI and Utility B I..VI
    -- load as variants 1..6 of their respective base canonical class.
    pattern_variant        INT NOT NULL DEFAULT 1,
    -- Position in the judges book page, 1-13. Combined with
    -- canonical_class_id and pattern_variant, this is the natural key.
    display_order          INT NOT NULL,
    -- Points the exercise is worth when scored. Rollup/header rows
    -- leave this NULL.
    max_points             INT,
    -- Raw cell text for unmatched scored cells and rollup rows. NULL
    -- when the cell maps cleanly to an obedience_exercises entry.
    box_label              TEXT,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT obedience_class_exercises_has_content CHECK (
        obedience_exercise_id IS NOT NULL
        OR max_points IS NOT NULL
        OR box_label IS NOT NULL
    ),
    CONSTRAINT obedience_class_exercises_exercise_has_points CHECK (
        obedience_exercise_id IS NULL
        OR max_points IS NOT NULL
    )
);

CREATE UNIQUE INDEX obedience_class_exercises_class_variant_order_uk
    ON obedience_class_exercises (canonical_class_id, pattern_variant, display_order);
CREATE INDEX obedience_class_exercises_canonical_class_id_ix
    ON obedience_class_exercises (canonical_class_id);
CREATE INDEX obedience_class_exercises_exercise_id_ix
    ON obedience_class_exercises (obedience_exercise_id) WHERE obedience_exercise_id IS NOT NULL;
