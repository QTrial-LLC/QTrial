-- Enable row-level security on the eight tenant-scoped tables added
-- in PR 2b. Matches the pattern established in
-- 20260420120700_enable_rls_on_entry_layer.up.sql: each table has a
-- direct club_id column, policy compares against
-- current_setting('app.current_club_id'), symmetric USING and WITH
-- CHECK (no cross-club "picker" shape needed because the app layer
-- always has a concrete club context when these tables are
-- addressed).
--
-- platform_admins is intentionally excluded. It carries no club_id,
-- has no RLS, and qtrial_tenant receives no grant. Tenant sessions
-- cannot read or write it; access runs as the qtrial role via a
-- platform-admin authorization path at the API layer, which MUST
-- log every access (per DATA_MODEL.md §5).
--
-- Grants come before the ENABLE ROW LEVEL SECURITY so that any
-- query run between the two steps fails closed (no grant means no
-- access), never the other way around.

------------------------------------------------------------
-- GRANTS
------------------------------------------------------------

GRANT SELECT, INSERT, UPDATE, DELETE ON dog_ownerships          TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON dog_trial_jump_heights  TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON armband_assignments     TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON email_templates         TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON submission_records      TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON payments                TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON refunds                 TO qtrial_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON audit_log               TO qtrial_tenant;

------------------------------------------------------------
-- RLS
------------------------------------------------------------

ALTER TABLE dog_ownerships          ENABLE ROW LEVEL SECURITY;
ALTER TABLE dog_trial_jump_heights  ENABLE ROW LEVEL SECURITY;
ALTER TABLE armband_assignments     ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_templates         ENABLE ROW LEVEL SECURITY;
ALTER TABLE submission_records      ENABLE ROW LEVEL SECURITY;
ALTER TABLE payments                ENABLE ROW LEVEL SECURITY;
ALTER TABLE refunds                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log               ENABLE ROW LEVEL SECURITY;

CREATE POLICY dog_ownerships_tenant ON dog_ownerships
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY dog_trial_jump_heights_tenant ON dog_trial_jump_heights
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY armband_assignments_tenant ON armband_assignments
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY email_templates_tenant ON email_templates
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY submission_records_tenant ON submission_records
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY payments_tenant ON payments
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY refunds_tenant ON refunds
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY audit_log_tenant ON audit_log
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);
