-- Payments and refunds against entries.
--
-- payments records one row per payment event against an entry. A
-- single entry can accumulate multiple payment rows (partial
-- payments, corrections, split-tender scenarios); the accounting
-- truth for an entry is the sum of live payments rows minus the
-- sum of live refunds rows pointing at them.
--
-- The method column reuses the payment_method ENUM created in
-- 20260420120400_create_entries.up.sql. The ENUM was deliberately
-- declared there so both the advertised method on entries and the
-- actually-recorded method on payments share the same type; the
-- check below is just a reuse, not a new ENUM.
--
-- The deposit tracking columns (deposited, deposit_date) exist
-- because new exhibitors sometimes send checks that are held up by
-- confusion about the additional-entry discount: the secretary
-- needs to know which checks have actually hit the bank so refund
-- decisions can proceed safely. The deposit_consistency CHECK
-- enforces that a row flagged deposited must have a deposit_date.
--
-- refunds records one row per refund event against a payment.
-- A refund's amount may be less than the referenced payment
-- (partial refund), and multiple refunds can reference the same
-- payment. The refund_reason ENUM is created here (not on
-- payments) because it is refund-only taxonomy.
--
-- ON DELETE RESTRICT on both entry_id (payments) and payment_id
-- (refunds): a paid or refunded entry cannot be hard-deleted
-- without first clearing the payment history; the app layer
-- soft-deletes the rows during normal operation.

CREATE TABLE payments (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its payment records.
    club_id                 UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    entry_id                UUID NOT NULL REFERENCES entries(id) ON DELETE RESTRICT,
    amount                  NUMERIC(10, 2) NOT NULL,
    method                  payment_method NOT NULL,
    -- Check number, Stripe charge id, PayPal transaction id, etc.
    -- Free text because the format varies by method.
    reference               TEXT,
    -- paid_at is the exhibitor-facing date on the instrument (the
    -- date written on the check, the date of the card charge). DATE
    -- not TIMESTAMPTZ because check-writing has no clock time.
    paid_at                 DATE,
    -- recorded_at is when QTrial saw the payment; TIMESTAMPTZ for
    -- precise ordering and audit.
    recorded_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    recorded_by_user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    -- Deposit tracking for physical checks. The club's deposit
    -- workflow is out-of-band from QTrial, but the secretary needs
    -- to see at-a-glance which payments are cleared.
    deposited               BOOL NOT NULL DEFAULT FALSE,
    deposit_date            DATE,
    note                    TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at              TIMESTAMPTZ,
    created_by              UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by              UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT payments_amount_positive CHECK (amount > 0),
    CONSTRAINT payments_deposit_consistency CHECK (
        deposited = FALSE OR deposit_date IS NOT NULL
    )
);

CREATE INDEX payments_club_id_ix
    ON payments (club_id) WHERE deleted_at IS NULL;
CREATE INDEX payments_entry_id_ix
    ON payments (entry_id) WHERE deleted_at IS NULL;
CREATE INDEX payments_recorded_at_ix ON payments (recorded_at);
-- Deposit-queue view ("show undeposited checks for my club"): filter
-- on (club_id, deposited) among live rows.
CREATE INDEX payments_club_deposited_ix
    ON payments (club_id, deposited) WHERE deleted_at IS NULL;


CREATE TYPE refund_reason AS ENUM (
    'bitch_in_season',
    'scratched_before_closing',
    'withdrawn',
    'duplicate_entry',
    'other'
);

CREATE TABLE refunds (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE.
    club_id                 UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    payment_id              UUID NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    amount                  NUMERIC(10, 2) NOT NULL,
    reason                  refund_reason NOT NULL,
    -- Free text detail; required by practice for reason = 'other'
    -- but left as a soft requirement rather than a CHECK because
    -- a hard-enforced CHECK would break silent imports from legacy
    -- data.
    reason_detail           TEXT,
    refunded_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    refunded_by_user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at              TIMESTAMPTZ,
    created_by              UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by              UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT refunds_amount_positive CHECK (amount > 0)
);

CREATE INDEX refunds_club_id_ix
    ON refunds (club_id) WHERE deleted_at IS NULL;
CREATE INDEX refunds_payment_id_ix
    ON refunds (payment_id) WHERE deleted_at IS NULL;
