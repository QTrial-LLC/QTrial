-- Judges are tracked per-club (each club maintains its own judge
-- directory with contact details) with a soft-unique constraint on
-- AKC judge number. Cross-club dedup (one canonical judge identity
-- shared across tenants) is a P2 decision; for MVP we accept that
-- two clubs each carry their own row for the same judge, as
-- documented in DATA_MODEL §3. Within a single club, partial-unique
-- on (club_id, akc_judge_number) prevents duplicate directory
-- entries for the same judge.
--
-- `user_id` is nullable and FKs into the global `users` table when
-- the judge has an OffLeash account. The link lets a judge sign in
-- and see their assignments in a future session. Without an account
-- the row is a pure contact record.

CREATE TABLE judges (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its judge directory; soft delete via deleted_at does not
    -- cascade.
    club_id             UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE SET NULL: if the linked OffLeash user is purged
    -- (GDPR, manual cleanup), the judge directory entry survives as
    -- a contact record without the login link.
    user_id             UUID REFERENCES users(id) ON DELETE SET NULL,
    last_name           TEXT NOT NULL,
    first_name          TEXT NOT NULL,
    akc_judge_number    TEXT,
    address_line1       TEXT,
    city                TEXT,
    state               TEXT,
    postal_code         TEXT,
    country_code        TEXT,
    phone               TEXT,
    cell                TEXT,
    email               CITEXT,
    is_provisional      BOOL NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ,
    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by          UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX judges_club_id_ix ON judges (club_id) WHERE deleted_at IS NULL;
CREATE INDEX judges_user_id_ix ON judges (user_id) WHERE user_id IS NOT NULL AND deleted_at IS NULL;

CREATE UNIQUE INDEX judges_club_akc_number_uk
    ON judges (club_id, akc_judge_number)
    WHERE akc_judge_number IS NOT NULL AND deleted_at IS NULL;
