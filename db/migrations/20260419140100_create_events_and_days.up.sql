-- Events are the top of the event hierarchy (event -> day -> trial).
-- An event is one AKC-approved gathering over one or more consecutive
-- days at a single venue, hosted by one club. RLS uses the direct
-- `club_id` column per the project convention; every child table in
-- the event subtree carries its own `club_id` so policy checks are
-- single-column comparisons, not FK walks.
--
-- Fees, armband scheme, waitlist config, and scheduling windows live
-- on the event rather than the trial because they are set once per
-- event and apply across every trial in that event.

CREATE TYPE event_status AS ENUM (
    'draft',
    'open',
    'closed',
    'in_progress',
    'complete',
    'archived'
);

CREATE TYPE armband_scheme AS ENUM (
    'per_trial',
    'per_event',
    'per_day',
    'per_class'
);

CREATE TABLE events (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of parent wipes the
    -- subtree; soft delete via deleted_at does not cascade.
    club_id                 UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    registry_id             UUID NOT NULL REFERENCES registries(id),
    name                    TEXT NOT NULL,
    cluster_name            TEXT,
    venue_name              TEXT,
    venue_address_line1     TEXT,
    venue_address_line2     TEXT,
    venue_city              TEXT,
    venue_state             TEXT,
    venue_postal_code       TEXT,
    venue_country_code      TEXT,
    entry_opens_at          TIMESTAMPTZ,
    entry_closes_at         TIMESTAMPTZ,
    moveup_deadline_at      TIMESTAMPTZ,
    armband_scheme          armband_scheme NOT NULL DEFAULT 'per_event',
    armband_start_number    INT NOT NULL DEFAULT 1,
    armband_interval        INT NOT NULL DEFAULT 1,
    catalog_fee             NUMERIC(10, 2),
    waitlist_accepted       BOOL NOT NULL DEFAULT TRUE,
    status                  event_status NOT NULL DEFAULT 'draft',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at              TIMESTAMPTZ,
    created_by              UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by              UUID REFERENCES users(id) ON DELETE SET NULL,

    -- CHECKs tolerate NULL arguments (any comparison with NULL is
    -- NULL, which passes the constraint), so columns that are
    -- nullable during draft can remain optional here without
    -- duplicating the CHECK for each NOT NULL combination.
    CONSTRAINT events_entry_window_ordered
        CHECK (entry_closes_at > entry_opens_at),
    CONSTRAINT events_armband_start_nonneg
        CHECK (armband_start_number >= 0),
    CONSTRAINT events_armband_interval_positive
        CHECK (armband_interval > 0),
    CONSTRAINT events_catalog_fee_nonneg
        CHECK (catalog_fee >= 0)
);

-- RLS-hot path: every tenant-scoped query filters by club_id.
CREATE INDEX events_club_id_ix ON events (club_id) WHERE deleted_at IS NULL;

-- AKC numbers for events are issued per-trial, not per-event (see the
-- trials table). No event-level unique constraint here.

CREATE TABLE event_days (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    club_id      UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of the parent event wipes this
    -- day row. Soft delete on events does not cascade; app layer
    -- handles related soft-delete semantics.
    event_id     UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    day_number   INT NOT NULL,
    date         DATE NOT NULL,
    -- Official published start time on this day. Local time-of-day
    -- only; the date column carries the calendar date. Secretary
    -- arrival and informal setup times are not tracked here; see
    -- docs/research/2026-04-19 finding 11.
    start_time   TIME,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at   TIMESTAMPTZ,
    created_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by   UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT event_days_day_number_positive CHECK (day_number >= 1)
);

CREATE INDEX event_days_club_id_ix ON event_days (club_id) WHERE deleted_at IS NULL;
CREATE INDEX event_days_event_id_ix ON event_days (event_id) WHERE deleted_at IS NULL;

-- One day_number per event, one date per event. Both enforced as
-- partial unique so soft-deleted rows don't block re-insertion.
CREATE UNIQUE INDEX event_days_event_day_number_uk
    ON event_days (event_id, day_number)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX event_days_event_date_uk
    ON event_days (event_id, date)
    WHERE deleted_at IS NULL;
