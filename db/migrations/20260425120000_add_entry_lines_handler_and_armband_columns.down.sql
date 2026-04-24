DROP INDEX IF EXISTS entry_lines_handler_contact_id_ix;
DROP INDEX IF EXISTS entry_lines_armband_assignment_id_ix;

ALTER TABLE entry_lines DROP COLUMN IF EXISTS junior_handler_akc_number;
ALTER TABLE entry_lines DROP COLUMN IF EXISTS handler_contact_id;
ALTER TABLE entry_lines DROP COLUMN IF EXISTS armband_assignment_id;
