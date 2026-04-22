DROP POLICY IF EXISTS user_club_roles_tenant ON user_club_roles;
DROP POLICY IF EXISTS clubs_visibility ON clubs;
DROP POLICY IF EXISTS users_self ON users;
DROP POLICY IF EXISTS canonical_classes_read_all ON canonical_classes;
DROP POLICY IF EXISTS akc_fee_schedules_read_all ON akc_fee_schedules;
DROP POLICY IF EXISTS registries_read_all ON registries;

ALTER TABLE user_club_roles  DISABLE ROW LEVEL SECURITY;
ALTER TABLE clubs            DISABLE ROW LEVEL SECURITY;
ALTER TABLE users            DISABLE ROW LEVEL SECURITY;
ALTER TABLE canonical_classes DISABLE ROW LEVEL SECURITY;
ALTER TABLE akc_fee_schedules DISABLE ROW LEVEL SECURITY;
ALTER TABLE registries        DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON user_club_roles  FROM qtrial_tenant;
REVOKE ALL ON users            FROM qtrial_tenant;
REVOKE ALL ON clubs            FROM qtrial_tenant;
REVOKE ALL ON canonical_classes FROM qtrial_tenant;
REVOKE ALL ON akc_fee_schedules FROM qtrial_tenant;
REVOKE ALL ON registries        FROM qtrial_tenant;
REVOKE USAGE ON SCHEMA public FROM qtrial_tenant;
