-- Recreate the three entries handler columns byte-identically to
-- the Phase 0 definitions (20260420120400_create_entries.up.sql
-- lines 53-55). Re-add order does not matter for correctness, but
-- reverse of the drop order for symmetry.

ALTER TABLE entries ADD COLUMN is_senior_handler BOOL NOT NULL DEFAULT FALSE;
ALTER TABLE entries ADD COLUMN junior_handler_number TEXT;
ALTER TABLE entries ADD COLUMN handler_name TEXT;
