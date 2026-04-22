-- Teams for Obedience Team and the Rally Team variants. A team row
-- is created at the event level; entry_lines associate with a team
-- via entry_lines.team_id (added in the entry_lines migration).
--
-- The team type enum covers the current AKC team offerings per
-- DOMAIN_GLOSSARY. Rally T Challenge Team is included because the
-- data model ought to carry it even if it is not offered by the
-- first clubs we onboard; adding an enum value later is cheaper
-- than adding it to existing production data.

CREATE TYPE team_type AS ENUM (
    'obedience_team',
    'rally_team_novice',
    'rally_team_advanced',
    'rally_team_excellent',
    'rally_t_challenge_team'
);

CREATE TABLE teams (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its team records; soft delete via deleted_at does not cascade.
    club_id      UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE: hard delete of the parent event wipes the
    -- team record. entry_lines that reference it cascade from their
    -- own parent chain.
    event_id     UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    team_name    TEXT NOT NULL,
    team_type    team_type NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at   TIMESTAMPTZ,
    created_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by   UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX teams_club_id_ix ON teams (club_id) WHERE deleted_at IS NULL;
CREATE INDEX teams_event_id_ix ON teams (event_id) WHERE deleted_at IS NULL;

-- Team names are unique within an event so secretaries cannot
-- create two teams with the same label at the same event.
CREATE UNIQUE INDEX teams_event_name_uk
    ON teams (event_id, team_name)
    WHERE deleted_at IS NULL;
