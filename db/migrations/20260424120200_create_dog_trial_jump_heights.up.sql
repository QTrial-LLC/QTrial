-- Jump height per (dog, trial).
--
-- Per Deborah's Q1 (2026-04-20), jump height is determined per dog
-- per trial: a dog running multiple jumping classes at the same
-- trial jumps the same height in all of them. The rare
-- judge-measurement override (approximately once per trial-secretary
-- career) must update the height for all of that dog's remaining
-- classes at the current trial, which is cleanest if the height
-- lives on one row per (dog, trial) rather than per entry line.
-- Rally Choice entries do not consult this table.
--
-- jump_height_inches is INT. Elected jump height is an AKC-defined
-- integer bucket set (Obedience: 4, 8, 10, 12, 14, 16, 18, 20, 22,
-- 24, 26, 28, 30, 32, 34, 36; Rally: 4, 8, 12, 16). This is
-- semantically distinct from dogs.jump_height_measured NUMERIC(4,1),
-- which is a physical measurement at the withers and can be
-- fractional (13.5"). The two columns share a name root but not a
-- type or meaning. If AKC ever adds half-inch elected buckets, that
-- change deserves its own design conversation; until then, INT plus
-- the enumerated CHECK is the honest shape.
--
-- The CHECK constraint enumerates the legitimate AKC heights.
-- On-leash classes do not consult this table at all (they have no
-- jump), so zero is intentionally NOT in the legal set here; a
-- row with zero would be meaningless and is better rejected
-- outright than silently stored.
--
-- No deleted_at on this table: a jump height is an immutable fact
-- about a dog at a trial. Correction happens via UPDATE of the same
-- row (often by the judge-measure path), not soft-delete. If the
-- app layer ever needs a history surface here, add audit_log
-- entries for changes; the current row is always the authoritative
-- value.

CREATE TABLE dog_trial_jump_heights (
    id                              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its jump-height rows.
    club_id                         UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE RESTRICT: a dog cannot be hard-deleted while
    -- referenced by a jump-height row. App layer soft-deletes dogs
    -- (deleted_at) during normal operation.
    dog_id                          UUID NOT NULL REFERENCES dogs(id) ON DELETE RESTRICT,
    -- ON DELETE CASCADE: if a trial is hard-deleted (unusual), its
    -- jump-height rows go with it; they are meaningless outside
    -- that trial.
    trial_id                        UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    jump_height_inches              INT NOT NULL,
    -- TRUE when a judge doubted the submitted height and measured
    -- the dog in-ring. The override applies to every remaining
    -- entry for this dog at this trial; the app layer UPDATEs the
    -- same row rather than inserting a new one.
    was_judge_measured              BOOL NOT NULL DEFAULT FALSE,
    judge_measured_at               TIMESTAMPTZ,
    -- Nullable contact reference to the measuring judge. Column
    -- name preserved for a potential future contacts-table
    -- consolidation; today the target is owners (the contact
    -- record pattern used across the schema).
    judge_measured_by_contact_id    UUID REFERENCES owners(id) ON DELETE SET NULL,
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by                      UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                      UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Enumerated AKC jump heights. 0 is not valid here; on-leash
    -- classes do not have a jump and therefore do not insert rows.
    CONSTRAINT dog_trial_jump_heights_valid_height CHECK (
        jump_height_inches IN (4, 8, 10, 12, 14, 16, 18, 20, 22,
                               24, 26, 28, 30, 32, 34, 36)
    ),
    -- If a measurement was done by a judge, we must know when.
    CONSTRAINT dog_trial_jump_heights_measured_consistency CHECK (
        was_judge_measured = FALSE
        OR judge_measured_at IS NOT NULL
    )
);

CREATE UNIQUE INDEX dog_trial_jump_heights_dog_trial_uk
    ON dog_trial_jump_heights (dog_id, trial_id);
CREATE INDEX dog_trial_jump_heights_club_id_ix
    ON dog_trial_jump_heights (club_id);
CREATE INDEX dog_trial_jump_heights_dog_id_ix
    ON dog_trial_jump_heights (dog_id);
CREATE INDEX dog_trial_jump_heights_trial_id_ix
    ON dog_trial_jump_heights (trial_id);
