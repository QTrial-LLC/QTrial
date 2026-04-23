-- Generic audit trail for sensitive operations.
--
-- audit_log captures before/after state for actions that touch
-- money, entry-state transitions, AKC submission artifacts, and
-- platform-admin operations. Written by the backend at the point
-- the action completes; rows are immutable once written.
--
-- action is TEXT, not an ENUM. The set of audited actions will
-- grow over time (status_changed, payment_recorded,
-- refund_issued, submission_generated, armband_reassigned, and
-- more); an ENUM would force an ALTER TYPE migration every time a
-- new action is added, and ENUM ALTERs are disruptive in
-- production. TEXT with an index on (club_id, entity_type,
-- occurred_at DESC) serves lookup queries without locking the
-- schema for every new action vocabulary entry.
--
-- actor_user_id is nullable because system actions (cron jobs,
-- webhook handlers, cleanup workers) have no human actor; the
-- entity_type + action + diff tell the story without needing a
-- synthetic "system" user row.
--
-- Rows are immutable: no updated_at, no deleted_at, no
-- created_by / updated_by columns. occurred_at replaces the
-- standard created_at to make the intent explicit. Retention is a
-- separate operational concern (cold-storage or partition drop),
-- not modeled in this schema.

CREATE TABLE audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its audit trail. That is deliberate: a club's audit rows are
    -- about its own entities, and if the club is removed the rows
    -- have no residual value.
    club_id         UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE SET NULL on actor: a historical audit row survives
    -- the actor being purged. The entity_type / action / diff still
    -- tell the story.
    actor_user_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    -- entity_type is free text paired with entity_id. Examples:
    -- "entry_line", "payment", "submission_record", "platform_admin".
    -- No FK: entity_id points into whichever table entity_type
    -- names. Rigid FK would force one column per auditable table
    -- or prevent auditing deletes.
    entity_type     TEXT NOT NULL,
    entity_id       UUID NOT NULL,
    -- action is free text for the reasons in the header comment.
    action          TEXT NOT NULL,
    -- Before/after snapshot shaped by the app layer. Shape varies
    -- per action; readers treat this as opaque JSON for display and
    -- structured-query-on-demand via JSONB operators.
    diff            JSONB,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Primary access pattern: "what happened to entities of type X in
-- my club, newest first". DESC on occurred_at matches the query
-- order so the index scan yields rows in display order without an
-- extra sort.
CREATE INDEX audit_log_club_entity_type_occurred_at_ix
    ON audit_log (club_id, entity_type, occurred_at DESC);
-- Actor-scoped access: "show me every auditable thing user X did
-- in my club". Filtered on actor_user_id IS NOT NULL because
-- system actions have no actor and would otherwise bloat this
-- index.
CREATE INDEX audit_log_club_actor_occurred_at_ix
    ON audit_log (club_id, actor_user_id, occurred_at DESC)
    WHERE actor_user_id IS NOT NULL;
-- Entity-specific drill-down: "show me every audit row for this
-- specific entity_id in my club".
CREATE INDEX audit_log_club_entity_id_ix
    ON audit_log (club_id, entity_id);
