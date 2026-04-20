-- Turn on row-level security and set the role grants that make it
-- enforceable.
--
-- The pattern across the whole schema:
--   * `offleash` owns every table. Connections as `offleash` bypass
--     RLS by Postgres convention (table owner is exempt), which gives
--     us platform-admin paths and migration tooling without additional
--     wiring. No ALTER TABLE ... FORCE ROW LEVEL SECURITY anywhere.
--   * `offleash_tenant` is a NOLOGIN role. Application requests open a
--     transaction as `offleash`, then execute `SET LOCAL ROLE
--     offleash_tenant` (via `offleash_shared::tenancy::begin_as_tenant`).
--     Inside that transaction the role owner is no longer exempt from
--     RLS, and the policies below restrict what the request can see
--     and write.
--   * Session variables `app.current_user_id` and `app.current_club_id`
--     are set with `SET LOCAL` inside the same transaction. Policies
--     read them via `current_setting(name, true)` so an unset variable
--     returns NULL rather than erroring.
--
-- Reference tables (registries, akc_fee_schedules, canonical_classes)
-- are read-only for tenants. A permissive `USING (TRUE)` policy plus a
-- SELECT-only GRANT gives us three-layer defense: the grant alone
-- prevents writes, the policy alone allows reads, and the absence of
-- INSERT/UPDATE/DELETE policies means even if a future grant were
-- added by mistake those statements would be denied.

------------------------------------------------------------
-- GRANTS
------------------------------------------------------------

GRANT USAGE ON SCHEMA public TO offleash_tenant;

-- Reference tables: read-only.
GRANT SELECT ON registries         TO offleash_tenant;
GRANT SELECT ON akc_fee_schedules  TO offleash_tenant;
GRANT SELECT ON canonical_classes  TO offleash_tenant;

-- Tenant tables: full CRUD. RLS does the row-level gating.
GRANT SELECT, INSERT, UPDATE, DELETE ON clubs            TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON users            TO offleash_tenant;
GRANT SELECT, INSERT, UPDATE, DELETE ON user_club_roles  TO offleash_tenant;

------------------------------------------------------------
-- RLS ON REFERENCE TABLES
------------------------------------------------------------

ALTER TABLE registries        ENABLE ROW LEVEL SECURITY;
ALTER TABLE akc_fee_schedules ENABLE ROW LEVEL SECURITY;
ALTER TABLE canonical_classes ENABLE ROW LEVEL SECURITY;

CREATE POLICY registries_read_all ON registries
    FOR SELECT
    USING (TRUE);

CREATE POLICY akc_fee_schedules_read_all ON akc_fee_schedules
    FOR SELECT
    USING (TRUE);

CREATE POLICY canonical_classes_read_all ON canonical_classes
    FOR SELECT
    USING (TRUE);

------------------------------------------------------------
-- RLS ON USERS (self-only)
------------------------------------------------------------
--
-- A tenant-context query can only read or modify the currently
-- authenticated user's row. Invite-by-email and judge-directory style
-- lookups must run without assuming the tenant role; i.e., they run
-- as `offleash` via a deliberate bypass path.

ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE POLICY users_self ON users
    FOR ALL
    USING (
        id = NULLIF(current_setting('app.current_user_id', TRUE), '')::uuid
    )
    WITH CHECK (
        id = NULLIF(current_setting('app.current_user_id', TRUE), '')::uuid
    );

------------------------------------------------------------
-- RLS ON CLUBS
------------------------------------------------------------
--
-- A club is visible when either:
--   * the current session's club_id matches (the fast path, for
--     requests that have picked a specific club context), or
--   * the current user holds at least one active role grant at that
--     club (the "my clubs" list view, before a specific club context
--     is chosen).
--
-- Writes are gated the same way at the row level. Role-based write
-- permission (only club_admin can update, etc.) is enforced in the
-- app layer above this policy.

ALTER TABLE clubs ENABLE ROW LEVEL SECURITY;

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

------------------------------------------------------------
-- RLS ON USER_CLUB_ROLES
------------------------------------------------------------
--
-- Straight tenant isolation: role grants belong to a club and are
-- visible / mutable only in the matching club's context.

ALTER TABLE user_club_roles ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_club_roles_tenant ON user_club_roles
    FOR ALL
    USING (
        club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
    )
    WITH CHECK (
        club_id = NULLIF(current_setting('app.current_club_id', TRUE), '')::uuid
    );
