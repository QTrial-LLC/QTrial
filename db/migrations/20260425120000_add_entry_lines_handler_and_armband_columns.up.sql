-- Move handler identity and armband linkage onto entry_lines.
--
-- This is the first of the PR 2c-surgery adds-before-drops pair.
-- Three columns land here so the sibling drop migrations on entries
-- (handler_name / junior_handler_number / is_senior_handler) and the
-- existing entries.armband column have a new home to point at.
--
-- handler_contact_id is NOT NULL per DATA_MODEL.md §5: every
-- entry_line has a handler, which defaults to the dog's primary
-- owner at the app layer on INSERT. A dog running multiple classes
-- at the same event may have different handlers per class (e.g.,
-- one class handled by a junior, another by the owner), which is why
-- the column lives on entry_lines and not on entries. ON DELETE
-- RESTRICT matches the existing entries.owner_id policy: an owner
-- referenced by a live entry_line cannot be hard-deleted without
-- soft-deleting the line first.
--
-- armband_assignment_id is nullable for two reasons:
--   1. Rally Choice entries carry no armband assignment (Choice is
--      not part of any numbered series; the series-based armband
--      model does not apply).
--   2. Pre-assignment state: a line exists from submission time
--      onward, but the armband is assigned later in the workflow
--      (REQUIREMENTS §6).
-- ON DELETE SET NULL so that deleting an assignment (for whatever
-- operational reason) does not invalidate the line; the app layer
-- can reassign.
--
-- junior_handler_akc_number is free TEXT with no FK: AKC's junior
-- handler number is an external identifier managed by AKC, not a
-- QTrial row reference. Populated only when a junior handles the
-- dog (REQ-ENTRY-016).
--
-- Partial indexes on the two FK columns follow the repo convention
-- of WHERE deleted_at IS NULL so soft-deleted rows do not bloat
-- lookup plans.

ALTER TABLE entry_lines
    ADD COLUMN armband_assignment_id UUID
        REFERENCES armband_assignments(id) ON DELETE SET NULL;

ALTER TABLE entry_lines
    ADD COLUMN handler_contact_id UUID NOT NULL
        REFERENCES owners(id) ON DELETE RESTRICT;

ALTER TABLE entry_lines
    ADD COLUMN junior_handler_akc_number TEXT;

CREATE INDEX entry_lines_armband_assignment_id_ix
    ON entry_lines (armband_assignment_id)
    WHERE armband_assignment_id IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX entry_lines_handler_contact_id_ix
    ON entry_lines (handler_contact_id) WHERE deleted_at IS NULL;
