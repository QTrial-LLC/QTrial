DROP POLICY IF EXISTS entry_line_results_tenant      ON entry_line_results;
DROP POLICY IF EXISTS entry_lines_tenant             ON entry_lines;
DROP POLICY IF EXISTS entries_tenant                 ON entries;
DROP POLICY IF EXISTS teams_tenant                   ON teams;
DROP POLICY IF EXISTS dog_sport_participation_tenant ON dog_sport_participation;
DROP POLICY IF EXISTS dog_titles_tenant              ON dog_titles;
DROP POLICY IF EXISTS dogs_tenant                    ON dogs;
DROP POLICY IF EXISTS owners_tenant                  ON owners;

ALTER TABLE entry_line_results      DISABLE ROW LEVEL SECURITY;
ALTER TABLE entry_lines             DISABLE ROW LEVEL SECURITY;
ALTER TABLE entries                 DISABLE ROW LEVEL SECURITY;
ALTER TABLE teams                   DISABLE ROW LEVEL SECURITY;
ALTER TABLE dog_sport_participation DISABLE ROW LEVEL SECURITY;
ALTER TABLE dog_titles              DISABLE ROW LEVEL SECURITY;
ALTER TABLE dogs                    DISABLE ROW LEVEL SECURITY;
ALTER TABLE owners                  DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON entry_line_results      FROM qtrial_tenant;
REVOKE ALL ON entry_lines             FROM qtrial_tenant;
REVOKE ALL ON entries                 FROM qtrial_tenant;
REVOKE ALL ON teams                   FROM qtrial_tenant;
REVOKE ALL ON dog_sport_participation FROM qtrial_tenant;
REVOKE ALL ON dog_titles              FROM qtrial_tenant;
REVOKE ALL ON dogs                    FROM qtrial_tenant;
REVOKE ALL ON owners                  FROM qtrial_tenant;
