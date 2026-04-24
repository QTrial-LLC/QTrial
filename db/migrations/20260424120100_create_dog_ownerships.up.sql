-- dog_ownerships models the many-to-many relationship between dogs
-- and owner contacts. Per Deborah's Q2 (2026-04-20), co-owners are
-- common in real AKC data and go on the dog record, not on the
-- entry. Replaces the legacy dogs.co_owners_text free-text column
-- (dogs.co_owners_text stays for migration-from-Access continuity
-- and retires in PR 2c).
--
-- Primary owner invariant: exactly one row per dog is flagged
-- is_primary = TRUE. Enforced by a partial unique index on dog_id
-- WHERE is_primary = TRUE AND deleted_at IS NULL. Soft-deleted rows
-- are ignored so a primary can be retired and re-assigned cleanly.
--
-- Tenant scope: denormalized club_id column per the Phase 0
-- convention ("every tenant-scoped table carries its own club_id";
-- see dog_titles and dog_sport_participation for the pattern).
-- App layer populates club_id from the parent dog at insert time
-- via qtrial_shared::tenancy::parent_club_id with
-- ParentEntity::Dog. RLS WITH CHECK enforces club_id equals the
-- current_club_id session variable.

CREATE TABLE dog_ownerships (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its dog_ownerships rows; soft delete via deleted_at does not
    -- cascade.
    club_id             UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard-deleting a dog drops its ownership
    -- rows. The app layer soft-deletes dogs (deleted_at) during
    -- normal operation, which does not cascade here.
    dog_id              UUID NOT NULL REFERENCES dogs(id) ON DELETE CASCADE,
    -- ON DELETE RESTRICT: an owner cannot be hard-deleted while
    -- referenced by a dog_ownerships row. App layer must
    -- soft-delete (set deleted_at on dog_ownerships) first.
    -- Column name preserved for a potential future contacts-table
    -- consolidation; today the FK target is owners.
    owner_contact_id    UUID NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    is_primary          BOOL NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX dog_ownerships_club_id_ix
    ON dog_ownerships (club_id) WHERE deleted_at IS NULL;
CREATE INDEX dog_ownerships_dog_id_ix
    ON dog_ownerships (dog_id) WHERE deleted_at IS NULL;
CREATE INDEX dog_ownerships_owner_contact_id_ix
    ON dog_ownerships (owner_contact_id) WHERE deleted_at IS NULL;

-- One ownership row per (dog, owner) among live rows. Soft-deleted
-- rows are excluded so a revoked owner can be re-added cleanly.
CREATE UNIQUE INDEX dog_ownerships_dog_owner_uk
    ON dog_ownerships (dog_id, owner_contact_id)
    WHERE deleted_at IS NULL;

-- Exactly one primary owner per dog. Partial unique: a dog has at
-- most one live is_primary=TRUE row; any number of non-primary
-- rows; any number of soft-deleted rows (historical primaries).
CREATE UNIQUE INDEX dog_ownerships_one_primary_per_dog_uk
    ON dog_ownerships (dog_id)
    WHERE is_primary = TRUE AND deleted_at IS NULL;
