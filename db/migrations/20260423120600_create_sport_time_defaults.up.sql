-- Sport time defaults.
--
-- Platform-level pacing defaults per sport-or-event, used by the
-- schedule generator to estimate trial duration and publish a
-- judging schedule. Per-event overrides (dogs_per_hour_override on
-- the events table) land in PR 2c. Per Deborah's Q5 (2026-04-20),
-- Obedience and Rally default to 3.0 min/dog as a rule of thumb;
-- her actual November 2025 numbers varied class by class (Rally
-- Choice ~4.3, Rally Master ~3.5, Rally Excellent B ~3.1), which
-- is the motivation for the per-event override.
--
-- sport_or_event is a free-text column rather than an FK to the
-- sport ENUM because the seed CSV distinguishes AKC Agility types
-- (Agility-Standard, Agility-JWW, Agility-FAST, Agility-ISC) that
-- do not live in the sport ENUM. The awards and scheduling paths
-- look rows up by this string.

CREATE TABLE sport_time_defaults (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Legacy TimeCalcID from tblTrialTimeCalculation.
    legacy_id             INT,
    -- Free text, e.g. "Obedience", "Rally", "Agility-Standard",
    -- "Agility-JWW", "Agility-FAST", "Agility-ISC".
    sport_or_event        TEXT NOT NULL UNIQUE,
    minutes_per_dog       NUMERIC(3, 1) NOT NULL,
    class_change_seconds  INT NOT NULL DEFAULT 0,
    event_change_seconds  INT NOT NULL DEFAULT 0,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT sport_time_defaults_minutes_positive CHECK (minutes_per_dog > 0),
    CONSTRAINT sport_time_defaults_class_change_nonneg CHECK (class_change_seconds >= 0),
    CONSTRAINT sport_time_defaults_event_change_nonneg CHECK (event_change_seconds >= 0)
);
