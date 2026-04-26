-- Drop trials.trial_chairperson. Free-text Phase 0 column with no
-- consumers in api/, workers/, shared/, or web/ (verified by grep
-- 2026-04-25). The chair role moves to events.trial_chair_user_id
-- (added in the previous migration) per Deborah's 2026-04-23 Q5
-- answer that the chair role is event-level, not trial-level.
--
-- A per-trial chair override column may be added back in a future
-- PR if a documented use case surfaces (e.g., a cluster event with
-- separate chairs for Obedience trials and Rally trials), but no
-- such case exists today. Drop is cleaner than carrying an unused
-- column with a deprecation marker.
--
-- Down recreates the column as TEXT NULL. Any previously-stored
-- chairperson strings are lost on the up migration; this is
-- acceptable because the column is unused legacy from the Phase 0
-- events/trials scaffold and no production data exists yet.

ALTER TABLE trials DROP COLUMN trial_chairperson;
