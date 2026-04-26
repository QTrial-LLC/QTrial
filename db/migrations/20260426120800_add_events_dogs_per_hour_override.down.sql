-- Drop events.dogs_per_hour_override. Any per-class pacing overrides
-- stored in this column are lost on rollback; existing rows fall back
-- to the schedule-generator chain levels 2-4 (per-offering override,
-- canonical_classes default, sport_time_defaults).

ALTER TABLE events DROP COLUMN dogs_per_hour_override;
