-- AKC fee schedule reference table.
--
-- Year-scoped, per-sport per-entry fees that AKC charges the club on
-- JOVOB7 (Obedience/Conformation) and JOVRY8 (Rally) submissions. AKC
-- raises fees periodically (confirmed 2026 increase per JOVOB7 10/25
-- and JOVRY8 10/25 form text); modeling this as data, not as code,
-- means future rate changes land as a new seed migration, not a code
-- change.
--
-- The `sport` grouping here is AKC's fee grouping, not our per-sport
-- sport enum. AKC bills Obedience and Conformation together
-- ("obedience_conformation"), and Rally on its own, each with
-- different rules about first-entry vs additional-entry fees and
-- secretary-fee thresholds. Additional AKC fee groupings (Agility,
-- etc.) will be added to `akc_fee_sport` when those sports land.
--
-- Natural unique key: (registry_id, sport, effective_year). Re-running
-- the seed is idempotent via ON CONFLICT on that triple.

CREATE TYPE akc_fee_sport AS ENUM (
    'obedience_conformation',
    'rally'
);

CREATE TABLE akc_fee_schedules (
    id                             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id                    UUID NOT NULL REFERENCES registries(id),
    sport                          akc_fee_sport NOT NULL,
    effective_year                 INT NOT NULL,
    -- Obedience/Conformation charges a $0.50 recording fee on the first
    -- entry per dog and $0 on additional entries. Rally charges no
    -- separate recording fee, so both columns are NULL for Rally rows.
    recording_fee_first_entry      NUMERIC(5, 2),
    recording_fee_additional       NUMERIC(5, 2),
    -- Service fee is where the rate increases effective 2026 show up.
    -- Obedience/Conformation uses a first-vs-additional split. Rally is
    -- a flat rate and stores the same value in both columns so joiners
    -- do not need sport-specific logic.
    service_fee_first_entry        NUMERIC(5, 2) NOT NULL,
    service_fee_additional         NUMERIC(5, 2) NOT NULL,
    -- Event secretary fee is charged only above the free-event
    -- threshold. Both values are absolutes; the calling app compares
    -- the secretary's year-to-date event count against the threshold
    -- to decide whether to include the fee on a given submission.
    event_secretary_fee            NUMERIC(5, 2) NOT NULL,
    event_secretary_fee_threshold  INT NOT NULL,
    -- List of entry-type codes that are NOT counted when computing the
    -- recording-fee total on JOVOB7/JOVRY8. Stored as an array for
    -- flexibility; the app joins against entry metadata when tallying.
    excluded_from_recording_fee    TEXT[] NOT NULL DEFAULT '{}',
    created_at                     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX akc_fee_schedules_registry_sport_year_uk
    ON akc_fee_schedules (registry_id, sport, effective_year);
