-- Recreate entry_lines.jump_height_inches and its CHECK byte-
-- identically to the Phase 0 definitions
-- (20260420120500_create_entry_lines.up.sql lines 43, 81-82).

ALTER TABLE entry_lines ADD COLUMN jump_height_inches NUMERIC(4, 1);

ALTER TABLE entry_lines
    ADD CONSTRAINT entry_lines_jump_height_nonneg
        CHECK (jump_height_inches >= 0);
