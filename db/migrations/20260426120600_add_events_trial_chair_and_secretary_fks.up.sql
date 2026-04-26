-- Add events.trial_chair_user_id and events.event_secretary_user_id.
--
-- Per Deborah's 2026-04-23 round-2 Q5, the Trial Chair and Trial
-- Secretary roles are distinct and event-level, not trial-level.
-- The chair handles pre-trial arrangements (acquiring judges, AKC
-- approval, judge accommodations, recruiting stewards / hospitality
-- crew / timekeeper / course builder, expense payments). The
-- secretary handles on-the-day operations (paperwork, scores,
-- entries). The two roles are usually held by two different people.
--
-- Granularity is per-event, not per-trial: the GFKC June 2026 Rally
-- premium list (db/seed/akc/sample_artifacts/gfkc_rally_premium_2026_06.pdf)
-- prints "Rally Trial Chair: Chris Argento" and "Trial Secretary:
-- Debbie Pruyn" once for both Saturday and Sunday trials. Multiple
-- trials within an event share one chair and one secretary; a future
-- per-trial override column can be added if a real-world case for
-- it surfaces.
--
-- ON DELETE SET NULL matches the existing created_by / updated_by
-- pattern on the events table
-- (20260419140100_create_events_and_days.up.sql lines 55-56). RESTRICT
-- would block user hard-deletion any time the user has been assigned
-- as chair or secretary, which is too aggressive.
--
-- Both columns are nullable. A draft event may be saved without
-- either assigned; per WORKFLOWS.md §1.3 the secretary is invited
-- early in the workflow and the chair is identified later. The API
-- layer enforces "both must be set before status transitions to
-- 'open'"; the database layer permits NULL for the draft state.
--
-- The free-text trials.trial_chairperson column is being dropped in
-- the next migration, replaced by this event-level FK.

ALTER TABLE events
    ADD COLUMN trial_chair_user_id      UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN event_secretary_user_id  UUID REFERENCES users(id) ON DELETE SET NULL;

-- Partial indexes on (column) WHERE deleted_at IS NULL match every
-- other event-subtree FK index in the schema. Useful for lookups
-- like "show me all events I am the chair for" without scanning
-- soft-deleted rows.
CREATE INDEX events_trial_chair_user_id_ix
    ON events (trial_chair_user_id) WHERE deleted_at IS NULL;
CREATE INDEX events_event_secretary_user_id_ix
    ON events (event_secretary_user_id) WHERE deleted_at IS NULL;
