-- Drop the indexes first (they reference the columns), then drop
-- the columns. Reverse of the up migration.

DROP INDEX IF EXISTS events_event_secretary_user_id_ix;
DROP INDEX IF EXISTS events_trial_chair_user_id_ix;

ALTER TABLE events
    DROP COLUMN event_secretary_user_id,
    DROP COLUMN trial_chair_user_id;
