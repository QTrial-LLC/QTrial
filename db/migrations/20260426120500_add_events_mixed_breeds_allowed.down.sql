-- Drop events.mixed_breeds_allowed. Any explicit FALSE values in
-- existing rows lose their information on rollback; this is acceptable
-- because the column is being added with no production data yet, and
-- the default TRUE captures the typical case anyway.

ALTER TABLE events DROP COLUMN mixed_breeds_allowed;
