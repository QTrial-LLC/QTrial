DROP POLICY IF EXISTS trial_awards_tenant          ON trial_awards;
DROP POLICY IF EXISTS judge_assignments_tenant     ON judge_assignments;
DROP POLICY IF EXISTS judges_tenant                ON judges;
DROP POLICY IF EXISTS trial_class_offerings_tenant ON trial_class_offerings;
DROP POLICY IF EXISTS trials_tenant                ON trials;
DROP POLICY IF EXISTS event_days_tenant            ON event_days;
DROP POLICY IF EXISTS events_tenant                ON events;

ALTER TABLE trial_awards           DISABLE ROW LEVEL SECURITY;
ALTER TABLE judge_assignments      DISABLE ROW LEVEL SECURITY;
ALTER TABLE judges                 DISABLE ROW LEVEL SECURITY;
ALTER TABLE trial_class_offerings  DISABLE ROW LEVEL SECURITY;
ALTER TABLE trials                 DISABLE ROW LEVEL SECURITY;
ALTER TABLE event_days             DISABLE ROW LEVEL SECURITY;
ALTER TABLE events                 DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON trial_awards           FROM qtrial_tenant;
REVOKE ALL ON judge_assignments      FROM qtrial_tenant;
REVOKE ALL ON judges                 FROM qtrial_tenant;
REVOKE ALL ON trial_class_offerings  FROM qtrial_tenant;
REVOKE ALL ON trials                 FROM qtrial_tenant;
REVOKE ALL ON event_days             FROM qtrial_tenant;
REVOKE ALL ON events                 FROM qtrial_tenant;
