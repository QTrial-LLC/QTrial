-- Drop entry_lines.jump_height_inches. Jump height moves to
-- dog_trial_jump_heights (PR 2b), keyed by (dog, trial).
--
-- Per Deborah's 2026-04-20 Q1, jump height never changes between
-- classes on the same day for the same dog; the rare in-ring judge
-- override (approximately once per trial-secretary career) must
-- update all of the dog's remaining entries at the trial, which is
-- cleanest with one row per (dog, trial) rather than one per entry
-- line. Rally Choice entries do not consult this table at all.
--
-- Render path: entry_lines joins dog_trial_jump_heights via
-- (entries.dog_id, trials.id) rather than carrying a denormalized
-- local copy. The per-line copy made the judge-override path
-- require N UPDATEs; the per-(dog, trial) model makes it one.
--
-- Drop order: CHECK constraint first (it references the column),
-- then the column.

ALTER TABLE entry_lines
    DROP CONSTRAINT IF EXISTS entry_lines_jump_height_nonneg;

ALTER TABLE entry_lines DROP COLUMN jump_height_inches;
