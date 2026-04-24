-- Drop entries.handler_name, entries.junior_handler_number,
-- entries.is_senior_handler. Handler identity moves to entry_lines
-- (handler_contact_id + junior_handler_akc_number, added in
-- 20260425120000_add_entry_lines_handler_and_armband_columns) per
-- DATA_MODEL.md §5.
--
-- Motivation: a dog running multiple classes at the same event may
-- have different handlers per class (e.g., one class handled by a
-- junior, another by the owner). The per-class granularity lives on
-- entry_lines, not on entries. Bundling all three drops in one
-- migration because they are semantically one change: "handler
-- identity moves off entries."
--
-- is_senior_handler is dropped entirely per Deborah's 2026-04-23
-- item 5 ("dropping from MVP. You've never seen it, so it doesn't
-- ship"). It was in the Phase 0 migration on speculation; the
-- round-2 Q&A confirmed the concept does not exist in AKC's current
-- trial secretary workflow.

ALTER TABLE entries DROP COLUMN handler_name;
ALTER TABLE entries DROP COLUMN junior_handler_number;
ALTER TABLE entries DROP COLUMN is_senior_handler;
