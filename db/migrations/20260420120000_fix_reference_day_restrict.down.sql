-- Revert to the SET NULL behavior from session 4. Keeping the down
-- so the reversible-mode invariant holds; doing this in practice
-- reintroduces the stale-strategy corner case and should be avoided.

ALTER TABLE trial_class_offerings
    DROP CONSTRAINT IF EXISTS trial_class_offerings_running_order_reference_day_id_fkey;

ALTER TABLE trial_class_offerings
    ADD CONSTRAINT trial_class_offerings_running_order_reference_day_id_fkey
        FOREIGN KEY (running_order_reference_day_id)
        REFERENCES event_days(id)
        ON DELETE SET NULL;
