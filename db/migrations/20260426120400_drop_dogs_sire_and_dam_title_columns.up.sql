-- Collapse the Phase 0 six-column sire/dam shape to two columns.
-- sire_registered_name and dam_registered_name STAY; the four
-- prefix/suffix sub-columns drop.
--
-- Why: the dog's own name is stored in ONE column (registered_name)
-- with companion parsed_* arrays populated by the name parser
-- (REQ-NAME-001). Storing sire and dam with a different shape
-- (prefix + name + suffix split at insert time) was philosophical
-- inconsistency. Either the parser handles all three or it handles
-- none. PR 2c-beta collapses to two columns for sire / dam so the
-- parser pipeline is uniform across the dog and its parents.
--
-- Per DATA_MODEL.md §4 "Sire and dam names are stored as free text
-- because their titles are not queried often enough to justify
-- denormalization. The catalog renderer parses them at display time
-- against the same title catalog used for the dog itself." The
-- render-time parsing cost is per-catalog-page, amortized; the six
-- columns were deadweight at the row level.
--
-- The 2026-04-26 Decisions-log entry "Sire and dam names are two
-- columns, not six" (PROJECT_STATUS.md, CHECKPOINT 2) captures the
-- rationale durably.
--
-- sire_registered_name and dam_registered_name keep their Phase 0
-- definitions (nullable TEXT). Only the four prefix/suffix columns
-- drop.

ALTER TABLE dogs
    DROP COLUMN sire_prefix_titles,
    DROP COLUMN sire_suffix_titles,
    DROP COLUMN dam_prefix_titles,
    DROP COLUMN dam_suffix_titles;
