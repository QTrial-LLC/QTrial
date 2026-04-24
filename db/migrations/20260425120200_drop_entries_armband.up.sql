-- Drop entries.armband. Armband moves to armband_assignments (PR 2b)
-- via entry_lines.armband_assignment_id (added in
-- 20260425120000_add_entry_lines_handler_and_armband_columns).
--
-- The per-series modeling supports Obedience's 500-series convention
-- (Advanced B / Excellent B / Master share armbands because they
-- combine into High Triple) that the per-event entries.armband
-- column could not represent. Deborah's 2026-04-23 Q3 and Q4
-- confirmed the series-sharing behavior: one armband per (dog, trial,
-- series), not one per (dog, event).
--
-- Drop order: partial unique index first (it references the column),
-- then the CHECK constraint, then the column itself. Reverse of the
-- create order in 20260420120400_create_entries.up.sql.

DROP INDEX IF EXISTS entries_event_armband_uk;

ALTER TABLE entries
    DROP CONSTRAINT IF EXISTS entries_armband_positive;

ALTER TABLE entries DROP COLUMN armband;
