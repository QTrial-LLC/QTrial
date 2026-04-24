DROP INDEX IF EXISTS submission_records_status_ix;
DROP INDEX IF EXISTS submission_records_trial_id_ix;
DROP INDEX IF EXISTS submission_records_club_id_ix;
DROP TABLE IF EXISTS submission_records;

DROP TYPE IF EXISTS submission_status;
DROP TYPE IF EXISTS submission_type;
