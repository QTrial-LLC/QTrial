-- Enable row-level security on the eight entry-layer tables. Every
-- table in this cluster is tenant-scoped through a direct club_id
-- column; the policy compares club_id against
-- current_setting('app.current_club_id'). Symmetric USING and WITH
-- CHECK because none of these tables need the cross-club "picker"
-- shape that clubs uses: a user reading entries, dogs, owners, etc.
-- always has a specific club context set.
--
-- GRANTS come before policies. offleash_tenant gets full CRUD on
-- every table; RLS gates rows.

------------------------------------------------------------
-- GRANTS
------------------------------------------------------------

GRANT SELECT, INSERT, UPDATE, DELETE ON owners                   TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON dogs                     TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON dog_titles               TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON dog_sport_participation  TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON teams                    TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON entries                  TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON entry_lines              TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON entry_line_results       TO offleash_tenant;

------------------------------------------------------------
-- RLS
------------------------------------------------------------

ALTER TABLE owners                  ENABLE ROW LEVEL SECURITY;
ALTER TABLE dogs                    ENABLE ROW LEVEL SECURITY;
ALTER TABLE dog_titles              ENABLE ROW LEVEL SECURITY;
ALTER TABLE dog_sport_participation ENABLE ROW LEVEL SECURITY;
ALTER TABLE teams                   ENABLE ROW LEVEL SECURITY;
ALTER TABLE entries                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE entry_lines             ENABLE ROW LEVEL SECURITY;
ALTER TABLE entry_line_results      ENABLE ROW LEVEL SECURITY;

CREATE POLICY owners_tenant ON owners
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY dogs_tenant ON dogs
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY dog_titles_tenant ON dog_titles
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY dog_sport_participation_tenant ON dog_sport_participation
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY teams_tenant ON teams
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY entries_tenant ON entries
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY entry_lines_tenant ON entry_lines
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);

CREATE POLICY entry_line_results_tenant ON entry_line_results
    FOR ALL
    USING (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid)
    WITH CHECK (club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid);
