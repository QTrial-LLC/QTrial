DROP POLICY IF EXISTS audit_log_tenant              ON audit_log;
DROP POLICY IF EXISTS refunds_tenant                 ON refunds;
DROP POLICY IF EXISTS payments_tenant                ON payments;
DROP POLICY IF EXISTS submission_records_tenant      ON submission_records;
DROP POLICY IF EXISTS email_templates_tenant         ON email_templates;
DROP POLICY IF EXISTS armband_assignments_tenant     ON armband_assignments;
DROP POLICY IF EXISTS dog_trial_jump_heights_tenant  ON dog_trial_jump_heights;
DROP POLICY IF EXISTS dog_ownerships_tenant          ON dog_ownerships;

ALTER TABLE audit_log               DISABLE ROW LEVEL SECURITY;
ALTER TABLE refunds                 DISABLE ROW LEVEL SECURITY;
ALTER TABLE payments                DISABLE ROW LEVEL SECURITY;
ALTER TABLE submission_records      DISABLE ROW LEVEL SECURITY;
ALTER TABLE email_templates         DISABLE ROW LEVEL SECURITY;
ALTER TABLE armband_assignments     DISABLE ROW LEVEL SECURITY;
ALTER TABLE dog_trial_jump_heights  DISABLE ROW LEVEL SECURITY;
ALTER TABLE dog_ownerships          DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON audit_log               FROM qtrial_tenant;
REVOKE ALL ON refunds                 FROM qtrial_tenant;
REVOKE ALL ON payments                FROM qtrial_tenant;
REVOKE ALL ON submission_records      FROM qtrial_tenant;
REVOKE ALL ON email_templates         FROM qtrial_tenant;
REVOKE ALL ON armband_assignments     FROM qtrial_tenant;
REVOKE ALL ON dog_trial_jump_heights  FROM qtrial_tenant;
REVOKE ALL ON dog_ownerships          FROM qtrial_tenant;
