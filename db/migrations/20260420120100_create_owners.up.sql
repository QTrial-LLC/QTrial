-- Owners are the registered names on a dog's AKC registration, held
-- per-club in MVP. Cross-tenant dedup (one canonical owner shared
-- across clubs) is a P2 decision per DATA_MODEL §3; for now each
-- club maintains its own owner directory, with an optional link to
-- the global users table when the owner also has an QTrial
-- account.
--
-- Owner is distinct from exhibitor: the exhibitor is the user who
-- actually submits an entry (tracked on entries.exhibitor_user_id);
-- the owner is who AKC has on file. Usually the same person, not
-- always. The catalog prints the registered owners, not the
-- exhibitor.

CREATE TABLE owners (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its owner directory; soft delete via deleted_at does not
    -- cascade.
    club_id                  UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- Optional link to the global users table. ON DELETE SET NULL so
    -- the owner directory entry survives if the user account is
    -- purged; the row remains a contact record for AKC catalog
    -- purposes.
    user_id                  UUID REFERENCES users(id) ON DELETE SET NULL,
    last_name                TEXT NOT NULL,
    first_name               TEXT NOT NULL,
    address_line1            TEXT,
    city                     TEXT,
    state                    TEXT,
    postal_code              TEXT,
    country_code             TEXT,
    phone                    TEXT,
    -- CITEXT so dedup-by-email is case insensitive across the club.
    email                    CITEXT,
    mailing_list_optin       BOOL NOT NULL DEFAULT FALSE,
    prefers_email_contact    BOOL NOT NULL DEFAULT TRUE,
    is_club_member           BOOL NOT NULL DEFAULT FALSE,
    active                   BOOL NOT NULL DEFAULT TRUE,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at               TIMESTAMPTZ,
    created_by               UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by               UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX owners_club_id_ix
    ON owners (club_id) WHERE deleted_at IS NULL;

-- Fast owner-by-email-and-club lookup for the "do we already have
-- this owner?" dedup path at entry time.
CREATE UNIQUE INDEX owners_club_email_uk
    ON owners (club_id, email)
    WHERE email IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX owners_user_id_ix
    ON owners (user_id) WHERE user_id IS NOT NULL AND deleted_at IS NULL;
