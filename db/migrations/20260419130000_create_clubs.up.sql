-- Clubs: the tenant root.
--
-- A row in `clubs` is one AKC-affiliated or AKC-licensed dog club. Every
-- tenant-scoped table that lands in later migrations (entries, dogs,
-- owners, judges, events, ...) FKs here on `club_id`, and RLS policies
-- on those tables restrict rows to the club in the current session's
-- `app.current_club_id` setting.
--
-- `primary_contact_user_id` is intentionally nullable: the bootstrap
-- insert for a new club predates any user at that club. App-layer flow
-- backfills the contact once the first user_club_role is granted.
--
-- `created_by` and `updated_by` FK into users(id) once that table is
-- created, via a deferred FK added in the next migration. They are
-- nullable to permit bootstrap rows, with app-layer discipline (not a
-- NOT NULL constraint) enforcing non-null for normal creates. See
-- comments on those columns.

CREATE TYPE akc_club_status AS ENUM (
    'member',
    'licensed',
    'none'
);

CREATE TYPE billing_status AS ENUM (
    'active',
    'comped',
    'suspended'
);

CREATE TABLE clubs (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name             TEXT NOT NULL,
    abbreviation             TEXT,
    akc_club_number          TEXT,
    ukc_club_number          TEXT,
    akc_status               akc_club_status NOT NULL DEFAULT 'none',
    logo_object_key          TEXT,
    primary_contact_user_id  UUID,
    billing_status           billing_status NOT NULL DEFAULT 'active',
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at               TIMESTAMPTZ,
    -- NULL indicates either the primordial bootstrap row or a row
    -- created via a path that pre-dates user-attribution tracking.
    -- Normal app creates MUST set this; enforced at app layer, not by
    -- NOT NULL constraint, because of the bootstrap problem. FK to
    -- users(id) is added in 20260419130100_create_users after the
    -- users table exists.
    created_by               UUID,
    updated_by               UUID
);

CREATE UNIQUE INDEX clubs_akc_club_number_uk
    ON clubs (akc_club_number)
    WHERE akc_club_number IS NOT NULL AND deleted_at IS NULL;
