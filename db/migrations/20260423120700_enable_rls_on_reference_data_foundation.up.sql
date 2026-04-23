-- Enable permissive-read RLS on the reference tables created in this
-- PR. Pattern matches the one established in
-- 20260419130300_enable_rls_and_grants for registries,
-- akc_fee_schedules, and canonical_classes:
--
--   * GRANT SELECT ON <table> TO qtrial_tenant.
--   * ALTER TABLE <table> ENABLE ROW LEVEL SECURITY.
--   * CREATE POLICY <table>_read_all ON <table> FOR SELECT USING (TRUE).
--
-- No INSERT/UPDATE/DELETE policy is created for qtrial_tenant, which
-- means those operations are implicitly denied even if a future grant
-- were added by mistake. Table owner (qtrial) still bypasses RLS by
-- Postgres convention, which is how the seed loader and future admin
-- paths populate reference data.

------------------------------------------------------------
-- GRANTS
------------------------------------------------------------

GRANT SELECT ON countries                  TO qtrial_tenant;
GRANT SELECT ON states                     TO qtrial_tenant;
GRANT SELECT ON breed_groups               TO qtrial_tenant;
GRANT SELECT ON breeds                     TO qtrial_tenant;
GRANT SELECT ON breed_varieties            TO qtrial_tenant;
GRANT SELECT ON title_prefixes             TO qtrial_tenant;
GRANT SELECT ON title_suffixes             TO qtrial_tenant;
GRANT SELECT ON jump_heights               TO qtrial_tenant;
GRANT SELECT ON obedience_exercises        TO qtrial_tenant;
GRANT SELECT ON obedience_class_exercises  TO qtrial_tenant;
GRANT SELECT ON otch_points                TO qtrial_tenant;
GRANT SELECT ON om_points                  TO qtrial_tenant;
GRANT SELECT ON rally_rach_points          TO qtrial_tenant;
GRANT SELECT ON sport_time_defaults        TO qtrial_tenant;

------------------------------------------------------------
-- ENABLE RLS
------------------------------------------------------------

ALTER TABLE countries                  ENABLE ROW LEVEL SECURITY;
ALTER TABLE states                     ENABLE ROW LEVEL SECURITY;
ALTER TABLE breed_groups               ENABLE ROW LEVEL SECURITY;
ALTER TABLE breeds                     ENABLE ROW LEVEL SECURITY;
ALTER TABLE breed_varieties            ENABLE ROW LEVEL SECURITY;
ALTER TABLE title_prefixes             ENABLE ROW LEVEL SECURITY;
ALTER TABLE title_suffixes             ENABLE ROW LEVEL SECURITY;
ALTER TABLE jump_heights               ENABLE ROW LEVEL SECURITY;
ALTER TABLE obedience_exercises        ENABLE ROW LEVEL SECURITY;
ALTER TABLE obedience_class_exercises  ENABLE ROW LEVEL SECURITY;
ALTER TABLE otch_points                ENABLE ROW LEVEL SECURITY;
ALTER TABLE om_points                  ENABLE ROW LEVEL SECURITY;
ALTER TABLE rally_rach_points          ENABLE ROW LEVEL SECURITY;
ALTER TABLE sport_time_defaults        ENABLE ROW LEVEL SECURITY;

------------------------------------------------------------
-- POLICIES
------------------------------------------------------------

CREATE POLICY countries_read_all ON countries
    FOR SELECT USING (TRUE);

CREATE POLICY states_read_all ON states
    FOR SELECT USING (TRUE);

CREATE POLICY breed_groups_read_all ON breed_groups
    FOR SELECT USING (TRUE);

CREATE POLICY breeds_read_all ON breeds
    FOR SELECT USING (TRUE);

CREATE POLICY breed_varieties_read_all ON breed_varieties
    FOR SELECT USING (TRUE);

CREATE POLICY title_prefixes_read_all ON title_prefixes
    FOR SELECT USING (TRUE);

CREATE POLICY title_suffixes_read_all ON title_suffixes
    FOR SELECT USING (TRUE);

CREATE POLICY jump_heights_read_all ON jump_heights
    FOR SELECT USING (TRUE);

CREATE POLICY obedience_exercises_read_all ON obedience_exercises
    FOR SELECT USING (TRUE);

CREATE POLICY obedience_class_exercises_read_all ON obedience_class_exercises
    FOR SELECT USING (TRUE);

CREATE POLICY otch_points_read_all ON otch_points
    FOR SELECT USING (TRUE);

CREATE POLICY om_points_read_all ON om_points
    FOR SELECT USING (TRUE);

CREATE POLICY rally_rach_points_read_all ON rally_rach_points
    FOR SELECT USING (TRUE);

CREATE POLICY sport_time_defaults_read_all ON sport_time_defaults
    FOR SELECT USING (TRUE);
