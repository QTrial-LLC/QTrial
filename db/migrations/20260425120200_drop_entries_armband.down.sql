-- Recreate entries.armband, its CHECK, and the partial unique index
-- byte-identically to the original Phase 0 definitions
-- (20260420120400_create_entries.up.sql lines 64, 78, 104-106).

ALTER TABLE entries ADD COLUMN armband INT;

ALTER TABLE entries
    ADD CONSTRAINT entries_armband_positive CHECK (armband >= 1);

CREATE UNIQUE INDEX entries_event_armband_uk
    ON entries (event_id, armband)
    WHERE armband IS NOT NULL AND deleted_at IS NULL;
