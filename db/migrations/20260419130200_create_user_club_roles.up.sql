-- user_club_roles: the junction between users and clubs.
--
-- A role grant is tenant-scoped: it belongs to a club, and RLS filters
-- it by current_club_id. A user can have multiple role grants at a
-- single club (e.g., both `club_admin` and `judge`) but only one active
-- grant of any given role type at a time, enforced by a partial unique
-- index keyed on active rows.
--
-- `granted_at` / `revoked_at` capture the domain lifecycle of the grant
-- and are distinct from the audit `created_at` / `updated_at`. The
-- distinction matters when a grant is backdated (migrated from a
-- prior system) or when the grant row is edited for reasons other than
-- revocation.

CREATE TYPE user_club_role_type AS ENUM (
    'club_admin',
    'trial_secretary',
    'judge',
    'exhibitor'
);

CREATE TABLE user_club_roles (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id             UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role                user_club_role_type NOT NULL,
    granted_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by_user_id  UUID REFERENCES users(id) ON DELETE SET NULL,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ,
    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by          UUID REFERENCES users(id) ON DELETE SET NULL
);

-- Each user can hold any given role at a club at most once concurrently.
-- Revoked grants (revoked_at IS NOT NULL) are excluded so history can
-- accumulate (grant, revoke, re-grant).
CREATE UNIQUE INDEX user_club_roles_active_uk
    ON user_club_roles (user_id, club_id, role)
    WHERE revoked_at IS NULL AND deleted_at IS NULL;

-- Fast "what clubs is this user associated with" lookup, used by the
-- clubs RLS policy's subquery in the next migration.
CREATE INDEX user_club_roles_user_active_ix
    ON user_club_roles (user_id, club_id)
    WHERE revoked_at IS NULL AND deleted_at IS NULL;

-- Fast "who are the users at this club" lookup for admin screens.
CREATE INDEX user_club_roles_club_active_ix
    ON user_club_roles (club_id)
    WHERE revoked_at IS NULL AND deleted_at IS NULL;
