-- Add events.mixed_breeds_allowed BOOL NOT NULL DEFAULT TRUE.
--
-- Per Deborah's 2026-04-23 round-2 Q3, conformation events sometimes
-- exclude mixed-breed dogs (the AKC Canine Partners "All-American
-- Dog" program). The flag is set per-event so a Specialty conformation
-- event can opt out, while the typical Obedience or Rally trial keeps
-- the permissive default. The default of TRUE matches the most common
-- real-world case (Obedience and Rally trials accept All-American
-- Dog) and means existing event rows take the permissive value
-- without an explicit backfill.
--
-- Scope-lock for PR 2d, established by the 2026-04-25 design note
-- (docs/research/2026-04-25-pr-2d-checkpoint-0-design-note.md §B6):
-- this BOOL ships alone, with the breed/breed-group/breed-variety
-- list model deferred to a future PR. The two pieces are
-- structurally separate work - the BOOL handles the All-American Dog
-- exclusion case; the list model handles Specialty single-breed
-- restrictions and breed-group filters. Designing breed-list on
-- speculation produces a worse model than waiting until a real
-- Specialty or Group show artifact is in hand.
--
-- The Decisions-log entry locking this scope and the supersession
-- note pointing the 2026-04-23 research note and REQUIREMENTS.md:87
-- at the deferral land in CHECKPOINT 3.

ALTER TABLE events
    ADD COLUMN mixed_breeds_allowed BOOL NOT NULL DEFAULT TRUE;
