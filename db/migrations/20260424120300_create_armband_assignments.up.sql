-- Armband assignments per (dog, trial, armband series).
--
-- Per Deborah's 2026-04-23 feedback, armband series groupings are
-- driven by which classes qualify for combined awards (High in
-- Trial, High Combined, High Triple). Classes that share eligibility
-- for the same combined award share an armband series. Rally Choice
-- is excluded from High Triple, so at a Rally trial that runs
-- Novice/Advanced/Excellent/Master plus Choice, Choice has its own
-- series and the other four share one.
--
-- The per-trial series-to-classes mapping is trial configuration
-- (lands in PR 2c). This table stores only the assignment result:
-- "at trial T, dog D in series S carries armband number N". Entry
-- lines then FK to the appropriate assignment row via
-- entry_lines.armband_assignment_id (that column lands in PR 2c as
-- well).
--
-- Two uniqueness invariants, both strict:
--   (trial_id, armband_series, armband_number): no two dogs share
--     an armband in the same series at the same trial
--   (dog_id, trial_id, armband_series): one assignment per
--     (dog, trial, series)
--
-- No deleted_at on this table. Armband assignments are ephemeral
-- per-trial state. If a dog scratches before the trial, the app
-- layer deletes the row; there is no history to preserve here. A
-- historical trace belongs in audit_log instead.

CREATE TABLE armband_assignments (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its armband assignments.
    club_id          UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE RESTRICT: a dog cannot be hard-deleted while
    -- referenced by an active assignment. App layer removes the
    -- assignment first.
    dog_id           UUID NOT NULL REFERENCES dogs(id) ON DELETE RESTRICT,
    -- ON DELETE CASCADE on trial: assignments are meaningless
    -- outside their trial, so hard-deleting a trial drops them.
    trial_id         UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    -- Free text for the series label because series schemes vary by
    -- trial ("Advanced B / Excellent B / Master", "500", etc.). The
    -- app-layer logic that maps class offerings to series produces
    -- this string.
    armband_series   TEXT NOT NULL,
    armband_number   INT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by       UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT armband_assignments_number_positive CHECK (
        armband_number >= 1
    )
);

-- No two dogs share an armband number within the same series at
-- the same trial.
CREATE UNIQUE INDEX armband_assignments_trial_series_number_uk
    ON armband_assignments (trial_id, armband_series, armband_number);

-- One assignment row per (dog, trial, series). A dog may hold
-- assignments in multiple series at the same trial (e.g. Choice
-- plus Master) via distinct rows.
CREATE UNIQUE INDEX armband_assignments_dog_trial_series_uk
    ON armband_assignments (dog_id, trial_id, armband_series);

CREATE INDEX armband_assignments_club_id_ix ON armband_assignments (club_id);
CREATE INDEX armband_assignments_dog_id_ix ON armband_assignments (dog_id);
CREATE INDEX armband_assignments_trial_id_ix ON armband_assignments (trial_id);
