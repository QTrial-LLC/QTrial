-- Enable permissive-read RLS on combined_award_groups and
-- combined_award_group_classes.
--
-- Both tables are reference data, shared across tenants. The
-- pattern matches every other reference table in the schema
-- (db/migrations/README.md §RLS conventions, established in
-- 20260419130300_enable_rls_and_grants.up.sql for registries /
-- akc_fee_schedules / canonical_classes and replicated in
-- 20260423120700_enable_rls_on_reference_data_foundation.up.sql for
-- the 14 PR 2a reference tables):
--
--   * GRANT SELECT ON <table> TO qtrial_tenant.
--   * ALTER TABLE <table> ENABLE ROW LEVEL SECURITY.
--   * CREATE POLICY <table>_read_all ON <table> FOR SELECT USING (TRUE).
--
-- No INSERT/UPDATE/DELETE policy is created for qtrial_tenant, which
-- means those operations are implicitly denied even if a future
-- grant were added by mistake. The qtrial role (table owner) still
-- bypasses RLS by Postgres convention; the seed loader runs as
-- qtrial.
--
-- GRANTS come before the ENABLE ROW LEVEL SECURITY so any query run
-- between the two steps fails closed (no grant means no access).

------------------------------------------------------------
-- GRANTS
------------------------------------------------------------

GRANT SELECT ON combined_award_groups        TO qtrial_tenant;
GRANT SELECT ON combined_award_group_classes TO qtrial_tenant;

------------------------------------------------------------
-- ENABLE RLS
------------------------------------------------------------

ALTER TABLE combined_award_groups        ENABLE ROW LEVEL SECURITY;
ALTER TABLE combined_award_group_classes ENABLE ROW LEVEL SECURITY;

------------------------------------------------------------
-- POLICIES
------------------------------------------------------------

CREATE POLICY combined_award_groups_read_all ON combined_award_groups
    FOR SELECT USING (TRUE);

CREATE POLICY combined_award_group_classes_read_all ON combined_award_group_classes
    FOR SELECT USING (TRUE);
