DROP INDEX IF EXISTS event_days_event_date_uk;
DROP INDEX IF EXISTS event_days_event_day_number_uk;
DROP INDEX IF EXISTS event_days_event_id_ix;
DROP INDEX IF EXISTS event_days_club_id_ix;
DROP TABLE IF EXISTS event_days;

DROP INDEX IF EXISTS events_club_id_ix;
DROP TABLE IF EXISTS events;

DROP TYPE IF EXISTS armband_scheme;
DROP TYPE IF EXISTS event_status;
