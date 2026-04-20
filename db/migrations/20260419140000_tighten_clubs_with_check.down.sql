-- Revert to the session-3 shape: symmetric clubs policy (USING and
-- WITH CHECK both use the OR construction) plus strict-by-club_id
-- user_club_roles policy. The down exists only so the reversible-mode
-- invariant holds; reverting in practice would reintroduce the
-- cross-tenant write escape the up-migration closed.

DROP POLICY IF EXISTS user_club_roles_tenant ON user_club_roles;

CREATE POLICY user_club_roles_tenant ON user_club_roles
    FOR ALL
    USING (
        club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
    )
    WITH CHECK (
        club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
    );

DROP POLICY IF EXISTS clubs_visibility ON clubs;

CREATE POLICY clubs_visibility ON clubs
    FOR ALL
    USING (
        id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
        OR id IN (
            SELECT ucr.club_id
            FROM user_club_roles ucr
            WHERE ucr.user_id = NULLIF(current_setting('app.current_user_id', TRUE), '')::uuid
              AND ucr.revoked_at IS NULL
              AND ucr.deleted_at IS NULL
        )
    )
    WITH CHECK (
        id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
        OR id IN (
            SELECT ucr.club_id
            FROM user_club_roles ucr
            WHERE ucr.user_id = NULLIF(current_setting('app.current_user_id', TRUE), '')::uuid
              AND ucr.revoked_at IS NULL
              AND ucr.deleted_at IS NULL
        )
    );
