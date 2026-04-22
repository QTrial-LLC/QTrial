-- Enable row-level security on the seven tables introduced by the
-- event-setup phase. Every policy follows the same shape: USING and
-- WITH CHECK both compare the row's direct `club_id` column against
-- the current session's `app.current_club_id` via the NULLIF-
-- current_setting form established in the session-3 tenancy
-- migration. Unlike clubs, these tables have no cross-club "picker"
-- view; a user reading their events list always already has a
-- specific club context. Symmetric USING and WITH CHECK is the
-- simpler correct shape.
--
-- GRANTS come before policies. `qtrial_tenant` gets full CRUD on
-- every table (RLS does the row gating). Reference-data style
-- tables (canonical_classes, registries) were granted SELECT-only
-- to qtrial_tenant in session 3 and need no additional grants
-- here; they are the read-side of the event subtree, not part of
-- the event subtree itself.

------------------------------------------------------------
-- GRANTS
------------------------------------------------------------

GRANT SELECT, INSERT, UPDATE, DELETE ON events                 TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON event_days             TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON trials                 TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON trial_class_offerings  TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON judges                 TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON judge_assignments      TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON trial_awards           TO qtrial_tenant;

------------------------------------------------------------
-- RLS ON EVENT-SETUP TABLES
------------------------------------------------------------

ALTER TABLE events                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_days             ENABLE ROW LEVEL SECURITY;
ALTER TABLE trials                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE trial_class_offerings  ENABLE ROW LEVEL SECURITY;
ALTER TABLE judges                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE judge_assignments      ENABLE ROW LEVEL SECURITY;
ALTER TABLE trial_awards           ENABLE ROW LEVEL SECURITY;

CREATE POLICY events_tenant ON events
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY event_days_tenant ON event_days
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY trials_tenant ON trials
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY trial_class_offerings_tenant ON trial_class_offerings
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY judges_tenant ON judges
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY judge_assignments_tenant ON judge_assignments
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY trial_awards_tenant ON trial_awards
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);
