DROP INDEX IF EXISTS trial_class_offerings_trial_class_uk;
DROP INDEX IF EXISTS trial_class_offerings_trial_id_ix;
DROP INDEX IF EXISTS trial_class_offerings_club_id_ix;
DROP TABLE IF EXISTS trial_class_offerings;

DROP INDEX IF EXISTS trials_akc_event_number_uk;
DROP INDEX IF EXISTS trials_event_day_trial_number_uk;
DROP INDEX IF EXISTS trials_event_day_id_ix;
DROP INDEX IF EXISTS trials_club_id_ix;
DROP TABLE IF EXISTS trials;

DROP TYPE IF EXISTS running_order_strategy;
DROP TYPE IF EXISTS trial_status;
