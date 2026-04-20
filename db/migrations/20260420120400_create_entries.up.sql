-- entries is the top-level record for one dog competing at one
-- event under one exhibitor. It carries the per-event attributes
-- that do not vary per class: armband (assigned once, used across
-- every class the dog enters), handler info, junior handler number,
-- payment totals, and the confirmation and results email timestamps
-- Deborah confirmed 2026-04-20 are both real.
--
-- Armband placement (moved here from entry_lines per the 2026-04-19
-- scope decision): a dog has one armband per event, used across all
-- its classes at that event. The Glens Falls November 2025 catalog
-- is the ground-truth for this pattern: armband 509 appears in
-- Advanced B, Excellent B, Master, and Choice for the same dog.
--
-- The payment_method enum is created here because entries.payment_method
-- references it. The future payments table (a later Stripe-
-- integration session) will reuse the same enum type so the values
-- stay consistent between the advertised method on the entry and
-- the actual recorded payments.
--
-- This migration also closes the session-4 TODO that left
-- trial_awards.winning_entry_id without a FK constraint: now that
-- entries exists, the FK is added atomically in the same transaction
-- as the table creation. ON DELETE SET NULL because a hard-deleted
-- entry should not wipe the award history; the award row remains
-- for audit purposes with a null entry pointer, and the app layer
-- can reconstruct context from trial_awards.contributing_entry_line_ids.

CREATE TYPE payment_method AS ENUM (
    'card',
    'check',
    'money_order',
    'cash',
    'paypal',
    'coupon',
    'discount'
);

CREATE TABLE entries (
    id                           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its entries; soft delete via deleted_at does not cascade.
    club_id                      UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    event_id                     UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    -- ON DELETE RESTRICT on dog and exhibitor: an entry cannot be
    -- orphaned if its dog or exhibitor user is hard deleted. App
    -- layer cancels or soft-deletes entries first.
    dog_id                       UUID NOT NULL REFERENCES dogs(id) ON DELETE RESTRICT,
    exhibitor_user_id            UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    owner_id                     UUID NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    -- Handler on trial day may differ from the exhibitor (pro
    -- handler, family member, junior handler). Free text because
    -- the handler may not have an OffLeash account.
    handler_name                 TEXT,
    junior_handler_number        TEXT,
    is_senior_handler            BOOL NOT NULL DEFAULT FALSE,
    submitted_at                 TIMESTAMPTZ,
    payment_method               payment_method,
    total_owed                   NUMERIC(10, 2),
    total_paid                   NUMERIC(10, 2),
    -- Armband is per-dog-per-event. Assigned once during the
    -- armband assignment step (REQUIREMENTS §6); nullable before
    -- assignment. Partial unique index below ensures no two live
    -- entries in the same event share a number.
    armband                      INT,
    catalog_number               INT,
    -- Both confirmation and results email timestamps are tracked
    -- per Deborah's 2026-04-20 feedback confirming that both emails
    -- are distinct parts of the exhibitor communication lifecycle.
    confirmation_email_sent_at   TIMESTAMPTZ,
    results_email_sent_at        TIMESTAMPTZ,
    notes                        TEXT,
    created_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                   TIMESTAMPTZ,
    created_by                   UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                   UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT entries_armband_positive CHECK (armband >= 1),
    CONSTRAINT entries_catalog_number_positive CHECK (catalog_number >= 1),
    CONSTRAINT entries_total_owed_nonneg CHECK (total_owed >= 0),
    CONSTRAINT entries_total_paid_nonneg CHECK (total_paid >= 0)
);

CREATE INDEX entries_club_id_ix ON entries (club_id) WHERE deleted_at IS NULL;
CREATE INDEX entries_event_id_ix ON entries (event_id) WHERE deleted_at IS NULL;
CREATE INDEX entries_dog_id_ix ON entries (dog_id) WHERE deleted_at IS NULL;
CREATE INDEX entries_exhibitor_user_id_ix
    ON entries (exhibitor_user_id) WHERE deleted_at IS NULL;
CREATE INDEX entries_owner_id_ix ON entries (owner_id) WHERE deleted_at IS NULL;

-- One dog per event: a dog is entered once in an event, with a set
-- of classes layered on via entry_lines. Partial unique excludes
-- soft-deleted rows so a cancelled entry can be re-submitted.
CREATE UNIQUE INDEX entries_event_dog_uk
    ON entries (event_id, dog_id)
    WHERE deleted_at IS NULL;

-- Armband uniqueness within an event for live entries. The
-- "status_is_live" framing from the session-5 task maps to "not
-- soft-deleted" at the entries level: entries have no status
-- column (status lives on entry_lines), and scratching an entire
-- entry is modeled by setting deleted_at. The partial index is
-- therefore keyed on deleted_at IS NULL plus armband IS NOT NULL.
CREATE UNIQUE INDEX entries_event_armband_uk
    ON entries (event_id, armband)
    WHERE armband IS NOT NULL AND deleted_at IS NULL;

-- Close the session-4 TODO: trial_awards.winning_entry_id now has
-- an FK into entries.id. Atomic with entries creation so there is
-- never a moment where the FK target exists but the constraint is
-- missing.
ALTER TABLE trial_awards
    ADD CONSTRAINT trial_awards_winning_entry_fk
        FOREIGN KEY (winning_entry_id) REFERENCES entries(id) ON DELETE SET NULL;
