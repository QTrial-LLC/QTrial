-- entry_lines is one dog's participation in one class within one
-- entry. The status enum is the state machine referenced by
-- CLAUDE.md's "Entry has a state machine, do not add parallel
-- boolean columns" rule; every lifecycle transition a dog-in-class
-- goes through (payment pending, accepted, waitlisted, scratched,
-- withdrawn, transferred, moved up, absent at check-in, excused or
-- disqualified by the judge) maps to a status value here.
--
-- Per-line detail that can vary between classes for the same entry
-- lives on this table: jump height (classes within an entry can
-- have different jump requirements when the dog is entered in both
-- jumping and non-jumping classes), team membership, brace partner,
-- pre-entry transfer intent, and the running-order slot for trial
-- day. Armband is deliberately on entries, not here, because it is
-- per-dog-per-event (see entries).
--
-- The brace_partner_entry_line_id self-FK is DEFERRABLE INITIALLY
-- DEFERRED so a brace pair can be inserted in a single transaction
-- with each row referencing the other; the FK check runs at COMMIT
-- rather than at each INSERT.

CREATE TYPE entry_line_status AS ENUM (
    'pending_payment',
    'active',
    'waitlist',
    'scratched',
    'withdrawn',
    'transferred',
    'moved_up',
    'absent',
    'excused',
    'dq'
);

CREATE TABLE entry_lines (
    id                                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its entry lines; soft delete via deleted_at does not cascade.
    club_id                             UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    entry_id                            UUID NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    trial_class_offering_id             UUID NOT NULL
        REFERENCES trial_class_offerings(id) ON DELETE RESTRICT,
    jump_height_inches                  NUMERIC(4, 1),
    -- Denormalized string listing the other classes this dog is
    -- entered in at this event, rendered verbatim into the catalog.
    -- App layer updates it as entries are added or removed; storing
    -- it here avoids joining at catalog-render time.
    also_entered_in                     TEXT,
    -- ON DELETE SET NULL: if the team record is hard deleted the
    -- entry line loses its team association but survives as an
    -- individual entry.
    team_id                             UUID REFERENCES teams(id) ON DELETE SET NULL,
    -- Self-FK for brace partner. DEFERRABLE INITIALLY DEFERRED so a
    -- brace pair can be created in a single transaction where both
    -- rows reference each other; the FK check runs at COMMIT.
    brace_partner_entry_line_id         UUID
        REFERENCES entry_lines(id) DEFERRABLE INITIALLY DEFERRED,
    status                              entry_line_status NOT NULL DEFAULT 'pending_payment',
    status_changed_at                   TIMESTAMPTZ,
    status_reason                       TEXT,
    -- Pre-entry transfer intent target references the specific
    -- trial_class_offering to move the entry to when the trigger
    -- title is earned before trial start. The dropdown in
    -- Obedience Solution pairs these at entry time; QTrial models
    -- the pairing directly rather than carrying "Transfer to X" as
    -- parallel class rows.
    transfer_intent_target_class_id     UUID
        REFERENCES trial_class_offerings(id) ON DELETE SET NULL,
    transfer_intent_trigger_title_code  TEXT,
    waitlist_position                   INT,
    is_alternate                        BOOL NOT NULL DEFAULT FALSE,
    is_veteran                          BOOL NOT NULL DEFAULT FALSE,
    running_order_position              INT,
    random_order_number                 INT,
    created_at                          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                          TIMESTAMPTZ,
    created_by                          UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                          UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT entry_lines_jump_height_nonneg
        CHECK (jump_height_inches >= 0),
    CONSTRAINT entry_lines_waitlist_position_positive
        CHECK (waitlist_position >= 1),
    -- A waitlist_position can only be attached to a waitlisted line.
    -- Going the other way (waitlist implies position) is not
    -- enforced: a row can briefly be waitlist without a position
    -- while the app layer decides where to slot it in.
    CONSTRAINT entry_lines_waitlist_position_only_on_waitlist
        CHECK (waitlist_position IS NULL OR status = 'waitlist'),
    -- Transfer intent is all-or-nothing: either both the target
    -- offering and the trigger title code are set, or both are
    -- null. A half-configured intent cannot fire correctly when
    -- the trigger title is earned.
    CONSTRAINT entry_lines_transfer_intent_coupled CHECK (
        (transfer_intent_target_class_id IS NULL
         AND transfer_intent_trigger_title_code IS NULL)
        OR
        (transfer_intent_target_class_id IS NOT NULL
         AND transfer_intent_trigger_title_code IS NOT NULL)
    ),
    CONSTRAINT entry_lines_running_order_position_positive
        CHECK (running_order_position >= 1)
);

CREATE INDEX entry_lines_club_id_ix
    ON entry_lines (club_id) WHERE deleted_at IS NULL;
CREATE INDEX entry_lines_entry_id_ix
    ON entry_lines (entry_id) WHERE deleted_at IS NULL;
CREATE INDEX entry_lines_trial_class_offering_id_ix
    ON entry_lines (trial_class_offering_id) WHERE deleted_at IS NULL;
CREATE INDEX entry_lines_team_id_ix
    ON entry_lines (team_id) WHERE team_id IS NOT NULL AND deleted_at IS NULL;

-- Supports "show me waitlisted entries" and "show me scratched
-- entries at this trial" dashboards.
CREATE INDEX entry_lines_status_ix
    ON entry_lines (status) WHERE deleted_at IS NULL;

-- A dog is entered in a given class offering at most once per
-- entry. Partial unique excludes soft-deleted rows so a cancelled
-- class registration can be re-submitted.
CREATE UNIQUE INDEX entry_lines_entry_offering_uk
    ON entry_lines (entry_id, trial_class_offering_id)
    WHERE deleted_at IS NULL;
