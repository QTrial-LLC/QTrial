-- Change trial_class_offerings.running_order_reference_day_id from
-- ON DELETE SET NULL to ON DELETE RESTRICT.
--
-- The session-4 FK used SET NULL, which looked innocuous but created
-- a stale-strategy corner case: if the referenced event_day was hard
-- deleted, Postgres would null the reference column without re-
-- running CHECK constraints, leaving the offering in a state where
-- running_order_strategy = 'reverse_previous_day' pointed at NULL.
-- A future reload of that row would then fail the CHECK that couples
-- the strategy to a non-null reference.
--
-- RESTRICT forces the app layer to explicitly pick a replacement
-- running_order_strategy (and update the offering) before deleting
-- the reference day. That is the right place for the domain
-- knowledge anyway: only the secretary knows whether to fall back to
-- short_to_tall, random, or a manual ordering. A future contributor
-- looking at this constraint and tempted to "fix" it back to SET
-- NULL should leave it alone.

ALTER TABLE trial_class_offerings
    DROP CONSTRAINT IF EXISTS trial_class_offerings_running_order_reference_day_id_fkey;

ALTER TABLE trial_class_offerings
    ADD CONSTRAINT trial_class_offerings_running_order_reference_day_id_fkey
        FOREIGN KEY (running_order_reference_day_id)
        REFERENCES event_days(id)
        ON DELETE RESTRICT;
