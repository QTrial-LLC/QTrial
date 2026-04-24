-- Drop dogs.co_owners_text. Co-ownership is now modeled via the
-- dog_ownerships junction (PR 2b) with is_primary enforcing exactly
-- one primary owner per dog via partial unique index.
--
-- Per the 2026-04-20 Decisions-log entry "Co-ownership via
-- dog_ownerships junction, not co_owners_text field" (PROJECT
-- STATUS.md), the free-text column was a Phase 0 placeholder
-- pending the structured co-owner model. dog_ownerships shipped in
-- PR 2b; PR 2c-beta physically realizes the retirement by dropping
-- the column.
--
-- DATA_MODEL.md §4 (dogs section preamble) and §3 (dog_ownerships
-- rationale note) already describe co-ownership through the
-- junction; the §4 column table update that removes the
-- co_owners_text row lands in this PR's DATA_MODEL commit
-- (CHECKPOINT 2).

ALTER TABLE dogs DROP COLUMN co_owners_text;
