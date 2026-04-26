-- Reverse the RLS enable migration. Drop policies, disable RLS,
-- revoke the SELECT grant. Order is the reverse of the up
-- migration: policies first (they reference the tables; ENABLE/
-- DISABLE manages the table flag; revoking the grant comes last).

DROP POLICY IF EXISTS combined_award_group_classes_read_all
    ON combined_award_group_classes;
DROP POLICY IF EXISTS combined_award_groups_read_all
    ON combined_award_groups;

ALTER TABLE combined_award_group_classes DISABLE ROW LEVEL SECURITY;
ALTER TABLE combined_award_groups        DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON combined_award_group_classes FROM qtrial_tenant;
REVOKE ALL ON combined_award_groups        FROM qtrial_tenant;
