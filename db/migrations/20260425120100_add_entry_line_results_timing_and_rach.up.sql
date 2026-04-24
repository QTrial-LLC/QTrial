-- Add timing fields and Rally (RACH) points to entry_line_results.
--
-- DATA_MODEL.md §5 specifies three timing columns and a third
-- points column that were missing from the Phase 0 migration. The
-- four additions together close the results-row gap between what
-- Phase 0 produced and what the Nov 2025 marked catalog plus AKC's
-- RACH point rules require.
--
-- time_started / time_finished are the judge's TIMESTAMPTZ readings
-- from the judges book cover (the cover has "Time Started" and
-- "Time Finished" fields that the judge fills in during the run).
-- time_on_course is either computed from the difference (app layer)
-- or entered directly when the judge recorded only total time.
-- Tie-breaking rule is ORDER BY score DESC, time_on_course ASC
-- (Rally Excellent B, Nov 2025 Glens Falls catalog: armbands 512
-- and 524 both scored 100, placed 1st and 2nd by time).
--
-- rach_points parallels otch_points (Obedience) and om_points
-- (also Obedience): the Rally Championship point table mapping
-- qualifying score to championship points. Populated only for
-- qualifying Rally dogs. INT plus CHECK rach_points >= 0 mirrors
-- the existing otch_points pattern on this same table.
--
-- All four columns are nullable: a result row exists as soon as the
-- class is scored, but timing and points may arrive piecemeal
-- (timing written by the judge in the book, points computed later
-- by the awards engine).

ALTER TABLE entry_line_results
    ADD COLUMN time_started TIMESTAMPTZ;

ALTER TABLE entry_line_results
    ADD COLUMN time_finished TIMESTAMPTZ;

ALTER TABLE entry_line_results
    ADD COLUMN time_on_course INTERVAL;

ALTER TABLE entry_line_results
    ADD COLUMN rach_points INT;

ALTER TABLE entry_line_results
    ADD CONSTRAINT entry_line_results_rach_points_nonneg
        CHECK (rach_points >= 0);
