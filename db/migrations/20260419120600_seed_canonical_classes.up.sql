-- Canonical class catalog seed: AKC Obedience and AKC Rally.
--
-- Sources of truth:
--   Obedience: AKC Obedience Regulations (current as of 2026-04-19,
--     most recent amendment 2025-03-01; ref RO2999 on akc.org).
--   Rally: AKC Rally Regulations (current as of 2026-04-19; includes
--     the 2023-06-29 Rally Choice amendment).
--
-- Classes in scope this session (23 total):
--   Obedience Regular (6): Novice A/B, Open A/B, Utility A/B
--   Obedience Optional Titling (5): Beginner Novice A/B, Graduate
--     Novice, Graduate Open, Versatility
--   Obedience Preferred (3): Preferred Novice, Preferred Open,
--     Preferred Utility
--   Rally Regular (9): Novice A/B, Intermediate, Advanced A/B,
--     Excellent A/B, Master, Choice
--
-- Non-regular classes (Obedience Veterans/Sub-Novice/Wildcard/Brace/
-- Team, Rally Pairs/Team/T-Challenge) are deliberately out of scope
-- and land in a later seed migration when non-regular scoring and fee
-- handling are implemented as a unit.
--
-- Title earning rule JSONB is left NULL this session. The JSONB shape
-- will be designed when title-progression tracking lands; today's seed
-- populates the cheap scalar fields (legs_required_for_title,
-- qualifies_for_title_code, min_qualifying_score).
--
-- `legacy_class_code` is NULL for every row. Backfill from Deborah's
-- Access tblkAKCObedClassInfo when Phase 7 migration tooling reads
-- the .mde file.
--
-- `dogs_per_hour_default` values for Obedience are reasonable
-- estimates pending verification against the Access schema's
-- `tblTrialTimeCalculation` and against current AKC guidance. Rows
-- flagged TODO: verify with Deborah where the value is a judgment
-- call beyond what the regulations pin down.
--
-- Idempotent via ON CONFLICT on (registry_id, sport, code).

INSERT INTO canonical_classes (
    registry_id, sport, code, display_name, class_type,
    has_jumps, has_broad_jump, total_score, min_qualifying_score,
    dogs_per_hour_default, has_walkthrough, default_walkthrough_minutes,
    qualifies_for_title_code, legs_required_for_title,
    ab_eligibility_rule, ab_eligibility_title_code
)
SELECT r.id, v.sport::sport, v.code, v.display_name, v.class_type::canonical_class_type,
       v.has_jumps, v.has_broad_jump, v.total_score, v.min_qualifying_score,
       v.dogs_per_hour_default, v.has_walkthrough, v.default_walkthrough_minutes,
       v.qualifies_for_title_code, v.legs_required_for_title,
       v.ab_eligibility_rule::ab_eligibility_rule, v.ab_eligibility_title_code
FROM registries r
CROSS JOIN (VALUES
    -- ====================================================================
    -- OBEDIENCE - Regular classes
    -- AKC Obedience Regulations Ch 2 §§1-3 (Novice, Open, Utility).
    -- 200 max score, 170 minimum qualifying, three legs for the title.
    -- Novice A is restricted to handlers who have never earned a CD on
    -- any dog. Open A and Utility A mirror the handler-based restriction
    -- for CDX and UD respectively.
    -- ====================================================================
    -- TODO: verify with Deborah - Obedience dogs_per_hour defaults are
    -- estimates. Obedience Solution's tblTrialTimeCalculation likely
    -- has per-class values; confirm during Phase 7 migration tooling.
    ('obedience', 'akc_obed_novice_a', 'Novice A', 'regular',
     FALSE, FALSE, 200, 170, 10, FALSE, NULL::numeric,
     'CD', 3, 'handler_based', 'CD'),
    ('obedience', 'akc_obed_novice_b', 'Novice B', 'regular',
     FALSE, FALSE, 200, 170, 10, FALSE, NULL::numeric,
     'CD', 3, 'none', NULL::text),
    ('obedience', 'akc_obed_open_a', 'Open A', 'regular',
     TRUE, TRUE, 200, 170, 6, FALSE, NULL::numeric,
     'CDX', 3, 'handler_based', 'CDX'),
    ('obedience', 'akc_obed_open_b', 'Open B', 'regular',
     TRUE, TRUE, 200, 170, 6, FALSE, NULL::numeric,
     'CDX', 3, 'none', NULL::text),
    ('obedience', 'akc_obed_utility_a', 'Utility A', 'regular',
     TRUE, FALSE, 200, 170, 5, FALSE, NULL::numeric,
     'UD', 3, 'handler_based', 'UD'),
    ('obedience', 'akc_obed_utility_b', 'Utility B', 'regular',
     TRUE, FALSE, 200, 170, 5, FALSE, NULL::numeric,
     'UD', 3, 'none', NULL::text),

    -- ====================================================================
    -- OBEDIENCE - Optional Titling classes
    -- AKC Obedience Regulations Ch 3 (Beginner Novice, Graduate Novice,
    -- Graduate Open, Versatility). Same 170/200 scoring system as
    -- Regular classes; each class awards a distinct suffix title after
    -- three qualifying scores.
    -- ====================================================================
    -- Beginner Novice (BN) introduced as an Optional Titling class; the
    -- A/B split follows the same handler-based rule as Novice A.
    -- TODO: verify with Deborah - dogs_per_hour for BN/Graduate/Versatility.
    ('obedience', 'akc_obed_beginner_novice_a', 'Beginner Novice A', 'optional_titling',
     FALSE, FALSE, 200, 170, 12, FALSE, NULL::numeric,
     'BN', 3, 'handler_based', 'BN'),
    ('obedience', 'akc_obed_beginner_novice_b', 'Beginner Novice B', 'optional_titling',
     FALSE, FALSE, 200, 170, 12, FALSE, NULL::numeric,
     'BN', 3, 'none', NULL::text),
    -- Graduate Novice sits between Novice and Open; has no broad jump
    -- but introduces some off-leash work. No A/B split.
    ('obedience', 'akc_obed_graduate_novice', 'Graduate Novice', 'optional_titling',
     FALSE, FALSE, 200, 170, 8, FALSE, NULL::numeric,
     'GN', 3, 'none', NULL::text),
    -- Graduate Open sits between Open and Utility; includes jumps.
    ('obedience', 'akc_obed_graduate_open', 'Graduate Open', 'optional_titling',
     TRUE, FALSE, 200, 170, 5, FALSE, NULL::numeric,
     'GO', 3, 'none', NULL::text),
    -- Versatility combines exercises from Novice, Open, and Utility.
    -- Includes jumps.
    ('obedience', 'akc_obed_versatility', 'Versatility', 'optional_titling',
     TRUE, FALSE, 200, 170, 5, FALSE, NULL::numeric,
     'VER', 3, 'none', NULL::text),

    -- ====================================================================
    -- OBEDIENCE - Preferred classes
    -- AKC Obedience Regulations Ch 4. Modified versions of the Regular
    -- classes with reduced jump heights and some exercise modifications
    -- for dogs that benefit from them. Distinct titles (PCD/PCDX/PUTD).
    -- No A/B split at the Preferred level.
    -- ====================================================================
    ('obedience', 'akc_obed_preferred_novice', 'Preferred Novice', 'preferred',
     FALSE, FALSE, 200, 170, 10, FALSE, NULL::numeric,
     'PCD', 3, 'none', NULL::text),
    ('obedience', 'akc_obed_preferred_open', 'Preferred Open', 'preferred',
     TRUE, TRUE, 200, 170, 6, FALSE, NULL::numeric,
     'PCDX', 3, 'none', NULL::text),
    ('obedience', 'akc_obed_preferred_utility', 'Preferred Utility', 'preferred',
     TRUE, FALSE, 200, 170, 5, FALSE, NULL::numeric,
     'PUTD', 3, 'none', NULL::text),

    -- ====================================================================
    -- RALLY - Regular classes
    -- AKC Rally Regulations. 100 max score, 70 minimum qualifying. Three
    -- legs for titles RN/RI/RA/RE; TEN legs for RM (Ch 1 §26) and RC
    -- (Ch 3 §18). Rally Novice A eligibility is dog-based. Rally
    -- Advanced A and Excellent A carry additional handler restrictions
    -- (handler has never titled in Obedience, has never earned the
    -- corresponding Rally title) that require handler-title-history
    -- lookups; those additional restrictions are not fully encoded in
    -- this scalar schema and are flagged TODO for the title_earning_rule
    -- JSONB design.
    -- Every Rally class has a mandatory walkthrough per Ch 2 §25,
    -- defaulting to 10 minutes here; secretaries can override per trial.
    -- Rally judging rate is "up to 20 dogs per hour" (Ch 2 §4).
    -- ====================================================================
    ('rally', 'akc_rally_novice_a', 'Rally Novice A', 'regular',
     FALSE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RN', 3, 'dog_based', 'RN'),
    ('rally', 'akc_rally_novice_b', 'Rally Novice B', 'regular',
     FALSE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RN', 3, 'none', NULL::text),
    -- Rally Intermediate: stepping-stone class between Novice and
    -- Advanced. Off-leash, no jumps, no A/B split.
    ('rally', 'akc_rally_intermediate', 'Rally Intermediate', 'regular',
     FALSE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RI', 3, 'none', NULL::text),
    -- TODO: verify with Deborah - Advanced A/Excellent A require the
    -- handler-has-never-titled-in-Obedience check in addition to the
    -- dog-based eligibility. That compound rule lands in the
    -- title_earning_rule JSONB design; for now ab_eligibility_rule
    -- captures that the A-class rule is dog_and_handler_based, and the
    -- title_code captures the shared title the A class is closed to.
    ('rally', 'akc_rally_advanced_a', 'Rally Advanced A', 'regular',
     TRUE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RA', 3, 'dog_and_handler_based', 'RA'),
    ('rally', 'akc_rally_advanced_b', 'Rally Advanced B', 'regular',
     TRUE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RA', 3, 'none', NULL::text),
    ('rally', 'akc_rally_excellent_a', 'Rally Excellent A', 'regular',
     TRUE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RE', 3, 'dog_and_handler_based', 'RE'),
    ('rally', 'akc_rally_excellent_b', 'Rally Excellent B', 'regular',
     TRUE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RE', 3, 'none', NULL::text),
    -- Rally Master: off-leash, jumps, 10 qualifying scores for RM
    -- title (Ch 1 §26). Single class, no A/B split. Available as one
    -- of the HC and RHTQ combined-award contributors.
    ('rally', 'akc_rally_master', 'Rally Master', 'regular',
     TRUE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RM', 10, 'none', NULL::text),
    -- AKC Rally Regulations Ch 3 §18 (Rally Choice Class), effective
    -- 2023-06-29. All dogs eligible regardless of prior titling; may
    -- continue competing indefinitely. No jumps; all signs judged
    -- off-leash. Ten qualifying scores earn RC; RC2 at 20, RC3 at 30.
    ('rally', 'akc_rally_choice', 'Rally Choice', 'regular',
     FALSE, FALSE, 100, 70, 20, TRUE, 10.0::numeric,
     'RC', 10, 'none', NULL::text)
) AS v(sport, code, display_name, class_type,
       has_jumps, has_broad_jump, total_score, min_qualifying_score,
       dogs_per_hour_default, has_walkthrough, default_walkthrough_minutes,
       qualifies_for_title_code, legs_required_for_title,
       ab_eligibility_rule, ab_eligibility_title_code)
WHERE r.code = 'AKC'
ON CONFLICT (registry_id, sport, code) DO NOTHING;
