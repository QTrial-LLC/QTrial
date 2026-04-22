ALTER TABLE trial_awards DROP CONSTRAINT IF EXISTS trial_awards_winning_entry_fk;

DROP INDEX IF EXISTS entries_event_armband_uk;
DROP INDEX IF EXISTS entries_event_dog_uk;
DROP INDEX IF EXISTS entries_owner_id_ix;
DROP INDEX IF EXISTS entries_exhibitor_user_id_ix;
DROP INDEX IF EXISTS entries_dog_id_ix;
DROP INDEX IF EXISTS entries_event_id_ix;
DROP INDEX IF EXISTS entries_club_id_ix;
DROP TABLE IF EXISTS entries;
DROP TYPE IF EXISTS payment_method;
