DROP INDEX IF EXISTS trial_awards_trial_type_uk;
DROP INDEX IF EXISTS trial_awards_contributing_entry_lines_gin;
DROP INDEX IF EXISTS trial_awards_trial_id_ix;
DROP INDEX IF EXISTS trial_awards_club_id_ix;
DROP TABLE IF EXISTS trial_awards;
DROP TYPE IF EXISTS award_type;

DROP INDEX IF EXISTS judge_assignments_offering_judge_uk;
DROP INDEX IF EXISTS judge_assignments_judge_id_ix;
DROP INDEX IF EXISTS judge_assignments_class_offering_ix;
DROP INDEX IF EXISTS judge_assignments_club_id_ix;
DROP TABLE IF EXISTS judge_assignments;
