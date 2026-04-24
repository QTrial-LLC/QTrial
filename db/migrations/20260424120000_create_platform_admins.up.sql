-- Platform administrator grants.
--
-- platform_admins records users who hold cross-tenant authority:
-- support engineers, billing operators, the founder. It is
-- deliberately separate from user_club_roles, which is the
-- tenant-scoped role table, so platform grants are never visible to
-- club-scoped queries and are never affected by tenant RLS.
--
-- Not tenant-scoped: no club_id, no RLS on this table. The `qtrial`
-- role (owner, bypasses RLS) is the only principal that reads or
-- writes this table. qtrial_tenant does NOT get any grant; a tenant
-- connection cannot see who the platform admins are. Access from
-- the API layer runs as qtrial via a deliberate platform-admin
-- authorization path and MUST log every access (per DATA_MODEL.md
-- multi-tenancy enforcement §5).
--
-- Revocation is soft: the row carries revoked_at instead of being
-- deleted. This preserves grant history for audit, which is why
-- "active" uniqueness on user_id is enforced by a partial unique
-- index on revoked_at IS NULL rather than by the natural
-- UNIQUE (user_id). A user can be granted, revoked, and re-granted
-- over time; only one active grant at a time.

CREATE TABLE platform_admins (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ON DELETE RESTRICT: a user with an active platform_admins
    -- row cannot be hard-deleted without explicit revocation first.
    -- App layer must set revoked_at before attempting user deletion.
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    -- granted_by is nullable because the first platform admin is
    -- self-granted at system bootstrap; every subsequent grant
    -- records the granting admin. ON DELETE SET NULL preserves the
    -- grant row if the granting admin is later hard-deleted.
    granted_by_user_id  UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at          TIMESTAMPTZ
);

-- Exactly one active grant per user. Partial index: a user can have
-- any number of revoked rows plus at most one non-revoked row.
CREATE UNIQUE INDEX platform_admins_user_id_active_uk
    ON platform_admins (user_id) WHERE revoked_at IS NULL;

CREATE INDEX platform_admins_user_id_ix ON platform_admins (user_id);
