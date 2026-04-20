-- Two coupled RLS corrections that together make the "my clubs"
-- picker flow work and force writes to respect tenant scope.
--
-- 1) Tighten clubs WITH CHECK.
--
-- The session-3 clubs policy was symmetric: both USING and WITH CHECK
-- allowed rows visible via (id = current_club_id) OR (user has active
-- role at that club). That is correct for reads because the "my
-- clubs" list view legitimately needs to enumerate every club the
-- user has a role at. For writes it is too loose: a user holding
-- both a club_admin grant at club A and an exhibitor grant at club B
-- could, with app.current_club_id = A, still UPDATE or INSERT rows
-- for club B because the role-at-B arm of the OR satisfied WITH
-- CHECK regardless of context. The asymmetric shape rejects that:
-- writes only succeed when the row's id matches the current club
-- context. Role-based permission (only club_admin can update, etc.)
-- is still enforced at the app layer on top.
--
-- 2) Widen user_club_roles USING so the clubs OR arm actually works.
--
-- The session-3 user_club_roles policy restricted SELECT to rows
-- where club_id = current_club_id. Under that policy, the subquery
-- in the clubs USING clause was dead: running under the tenant role,
-- the subquery asked "which clubs does this user have a role at?"
-- but user_club_roles RLS filtered out every row except those at the
-- current club. The OR arm then returned no rows, collapsing the
-- clubs visibility to "id = current_club_id only." The "my clubs"
-- picker (user has logged in but not yet chosen a club, so
-- current_club_id is NULL) returned zero clubs.
--
-- Widening user_club_roles USING to (user_id = current_user_id OR
-- club_id = current_club_id) fixes both cases:
--   * A user sees all their own role grants across all clubs, which
--     feeds the "my clubs" picker and makes the clubs OR arm
--     functional.
--   * A club_admin still sees all role grants at their current club
--     (for user-management screens).
-- WITH CHECK on user_club_roles stays strict (club_id =
-- current_club_id) so writes cannot create or modify grants outside
-- the current tenant scope, even if they would be readable.

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
    );

DROP POLICY IF EXISTS user_club_roles_tenant ON user_club_roles;

CREATE POLICY user_club_roles_tenant ON user_club_roles
    FOR ALL
    USING (
        user_id = NULLIF(current_setting('app.current_user_id', TRUE), '')::uuid
        OR club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
    )
    WITH CHECK (
        club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
    );
