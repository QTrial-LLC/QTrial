-- Recreate trials.trial_chairperson as nullable TEXT, matching the
-- Phase 0 definition (20260419140200_create_trials_and_class_offerings.up.sql
-- line 40). Data dropped on the up migration is not restored; rollback
-- past this point is a development-time mechanic, not a production
-- recovery path.

ALTER TABLE trials
    ADD COLUMN trial_chairperson TEXT;
