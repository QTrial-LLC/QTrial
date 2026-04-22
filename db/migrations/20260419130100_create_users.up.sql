-- Users: the global identity directory.
--
-- Users are NOT tenant-scoped. One physical person can have roles at
-- many clubs; the mapping lives in user_club_roles (created next). RLS
-- on this table is a self-only policy (see the next RLS migration) so
-- a tenant-context query that accidentally SELECTs from users can only
-- see the currently-authenticated user's own row. Cross-user reads
-- (invite-by-email, judge directory, etc.) must run without the tenant
-- role (i.e., as `qtrial`, not as `qtrial_tenant`) and enforce
-- their own access rules at the app layer.
--
-- NOTE ON DOC INVERSION: DATA_MODEL §1 Multi-tenancy text treats users
-- as app-layer-gated with no RLS. That default was revisited and
-- inverted 2026-04-19: RLS with a self-only policy is cheap insurance
-- against a tenant-context query leaking PII (email, full name,
-- address, phone). The deliberate bypass path for legitimate
-- cross-user lookups is a feature; routing them through the owner
-- role is an explicit gate rather than an accidental one.

CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- The `sub` claim from the Keycloak-issued JWT. Nullable this
    -- session because Keycloak wiring is deferred; backfilled when
    -- identity provider integration lands.
    keycloak_sub             TEXT,
    email                    CITEXT NOT NULL,
    display_name             TEXT,
    first_name               TEXT,
    last_name                TEXT,
    phone                    TEXT,
    address_line1            TEXT,
    address_line2            TEXT,
    city                     TEXT,
    state                    TEXT,
    postal_code              TEXT,
    country_code             TEXT,
    junior_handler_number    TEXT,
    birthdate                DATE,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at               TIMESTAMPTZ,
    -- Self-FK. NULL for the primordial bootstrap user; app layer sets
    -- it for every user created via the normal flow.
    created_by               UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by               UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX users_email_uk
    ON users (email)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX users_keycloak_sub_uk
    ON users (keycloak_sub)
    WHERE keycloak_sub IS NOT NULL AND deleted_at IS NULL;

-- Wire the clubs.created_by / updated_by / primary_contact_user_id
-- FKs now that users exists. ON DELETE SET NULL preserves the club row
-- if the attributing user is purged (GDPR, manual cleanup).
ALTER TABLE clubs
    ADD CONSTRAINT clubs_primary_contact_fk
        FOREIGN KEY (primary_contact_user_id) REFERENCES users(id) ON DELETE SET NULL,
    ADD CONSTRAINT clubs_created_by_fk
        FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL,
    ADD CONSTRAINT clubs_updated_by_fk
        FOREIGN KEY (updated_by) REFERENCES users(id) ON DELETE SET NULL;
