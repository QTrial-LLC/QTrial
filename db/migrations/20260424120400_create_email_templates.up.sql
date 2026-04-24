-- Per-club email templates.
--
-- Per Deborah's Q4 (2026-04-20), template voices vary by club: some
-- clubs prefer a terse confirmation, others want a full reminder
-- checklist. Hardcoding one voice would force every club to accept
-- it; per-club template rows let each club tailor its outgoing
-- email without code changes.
--
-- template_key is TEXT rather than an ENUM because new keys will be
-- added over time (post_closing_reminder, cancellation_notice,
-- refund_confirmation, judge_change_notice, and more). TEXT with a
-- UNIQUE constraint on (club_id, template_key) WHERE
-- deleted_at IS NULL gives the right shape and avoids an ALTER TYPE
-- migration every time a new template surfaces.
--
-- Variable substitution is simple {{variable_name}} for MVP;
-- Jinja-style conditionals are post-MVP. The list of variables
-- available per template_key is documented in WORKFLOWS.md §10.
-- Default templates are seeded on club creation (a later PR);
-- clubs override via the settings UI.

CREATE TABLE email_templates (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its template overrides. Soft delete via deleted_at does not
    -- cascade.
    club_id             UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    template_key        TEXT NOT NULL,
    subject_template    TEXT NOT NULL,
    body_template       TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ,
    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by          UUID REFERENCES users(id) ON DELETE SET NULL
);

-- One live template per (club, key). Soft-deleted rows are excluded
-- so a deleted custom template can be restored or replaced cleanly.
CREATE UNIQUE INDEX email_templates_club_key_uk
    ON email_templates (club_id, template_key)
    WHERE deleted_at IS NULL;

CREATE INDEX email_templates_club_id_ix
    ON email_templates (club_id) WHERE deleted_at IS NULL;
