-- Create combined_award_groups and combined_award_group_classes.
--
-- Reference data, registry-scoped, shared across tenants. Permissive-
-- read RLS lands in the next migration following the project's
-- enable_rls_on_* convention (db/migrations/README.md §RLS conventions).
--
-- Purpose: drive both the additional-entry fee discount logic and
-- the per-trial combined-award computation. Per Deborah's 2026-04-23
-- round-2 Q4, the additional-entry discount applies to ANY double or
-- triple Q in B classes at one trial, not just the GFKC-listed
-- Master / Choice / RAE / RACH path. Modeling combined-award groups
-- as reference data lets the fee engine ask "is this dog entered in
-- 2+ classes in the same combined_award_group at this trial?" and
-- apply the discount accordingly.
--
-- The parent table (combined_award_groups) describes the group:
-- registry, sport, code, display name, the per-trial award type if
-- any, discount eligibility, and the AKC regulation citation.
-- award_type is nullable because some groups exist for discount-
-- eligibility only without a corresponding per-trial award:
--   - HC (Open B + Utility B): per-trial Obedience HC award
--   - RHC (Adv B + Ex B): per-trial Rally HC award
--   - RHTQ (Adv B + Ex B + Master): per-trial Rally HTQ award
--   - RAE (Adv B + Ex B): title-progression path, NO per-trial award
--   - RACH (Adv B + Ex B + Master): title-progression path, NO
--     per-trial award (note: RACH and RHTQ share the same three-
--     class membership but RHTQ produces a per-trial ribbon while
--     RACH accumulates points across 20 separate trials per Rally
--     Regulations Chapter 4 Section 2)
--
-- The junction (combined_award_group_classes) is the
-- group-to-canonical-class mapping. is_required_for_award is TRUE
-- when the dog must Q in this class at the trial for the group's
-- per-trial award OR for the group's title-progression leg. For all
-- five seed rows planned for CHECKPOINT 2 it is TRUE on every
-- junction row, because every AKC-recognized combined entry in this
-- table requires Q's in all listed classes. The flag stays in the
-- schema for future groups whose semantics require optional
-- contributors.
--
-- Sport scoping: column on the parent (sport sport NOT NULL); a
-- junction row's canonical_class must reference a canonical_classes
-- row whose sport matches the parent's sport. No DDL trigger
-- enforces the cross-row match; the seed loader (lands in CHECKPOINT
-- 2) validates it on each row, with an error message pointing to
-- the offending CSV row.
--
-- AKC citations are tracked verbatim in regulation_citation TEXT.
-- Source PDFs are committed under db/seed/akc/regulations/:
--   * Rally Regulations: akc_rally_regulations_1217.pdf, edition 1217
--     (December 2017 base, amended through January 8, 2024).
--   * Obedience Regulations: akc_obedience_regulations_2025_03.pdf,
--     2025-03 amended edition.
-- Verified 2026-04-25; Master + Choice does NOT appear as a
-- combined-entry path or combined-award definition in either
-- rulebook. The "Master + Choice" wording on the GFKC June 2026
-- premium list is a club-side fee discount, not an AKC combined
-- award.

CREATE TABLE combined_award_groups (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id           UUID NOT NULL REFERENCES registries(id),
    sport                 sport NOT NULL,
    -- Stable code used by the seed loader for idempotent upsert and
    -- by app code for stable references. Examples: 'akc_obedience_hc',
    -- 'akc_rally_rhtq', 'akc_rally_rae'.
    code                  TEXT NOT NULL,
    display_name          TEXT NOT NULL,
    -- The per-trial award type this group produces. NULL when the
    -- group exists for discount-eligibility only (RAE, RACH).
    award_type            award_type,
    is_discount_eligible  BOOL NOT NULL DEFAULT TRUE,
    -- Verbatim AKC regulation citation, e.g. "Rally Regulations
    -- Chapter 1, Section 32" for RHTQ. Nullable when no canonical
    -- citation exists; a TODO marker in the seed CSV is preferable
    -- to a wrong citation.
    regulation_citation   TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX combined_award_groups_registry_sport_code_uk
    ON combined_award_groups (registry_id, sport, code);

CREATE INDEX combined_award_groups_registry_id_ix
    ON combined_award_groups (registry_id);

CREATE TABLE combined_award_group_classes (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ON DELETE CASCADE: junction rows are meaningless without their
    -- parent group; if a group is deleted in a future seed update,
    -- its membership rows go too.
    combined_award_group_id     UUID NOT NULL
        REFERENCES combined_award_groups(id) ON DELETE CASCADE,
    -- ON DELETE RESTRICT (default): a canonical_classes row cannot
    -- be hard-deleted while it is a member of any combined_award_
    -- group. The seed loader removes the membership row first.
    canonical_class_id          UUID NOT NULL
        REFERENCES canonical_classes(id),
    -- TRUE when the dog must Q in this class for the group's award
    -- or title-progression leg. See header comment for semantics.
    is_required_for_award       BOOL NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX combined_award_group_classes_group_class_uk
    ON combined_award_group_classes
       (combined_award_group_id, canonical_class_id);

-- Index on canonical_class_id supports the fee-engine query
-- "given the dog's entered set, which combined_award_groups are
-- in play?" which walks from canonical_class_id upward to the
-- parent group.
CREATE INDEX combined_award_group_classes_canonical_class_id_ix
    ON combined_award_group_classes (canonical_class_id);
