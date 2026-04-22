-- AKC fee schedule seed data for 2025 and 2026.
--
-- Source of truth: AKC forms JOVOB7 (Report of Dog Show or Obedience
-- Trial, revision 10/25) and JOVRY8 (Report of Rally Trial, revision
-- 10/25). Both forms print the 2025 and 2026 fee tables directly.
--
-- Exclusion lists pulled from the same forms' "Recording Fee does NOT
-- apply to" boilerplate plus DOMAIN_GLOSSARY §Money §AKC recording fee
-- exclusions.
--
-- Idempotent via ON CONFLICT on (registry_id, sport, effective_year).
-- Re-running against a populated DB is a no-op.

INSERT INTO akc_fee_schedules (
    registry_id, sport, effective_year,
    recording_fee_first_entry, recording_fee_additional,
    service_fee_first_entry, service_fee_additional,
    event_secretary_fee, event_secretary_fee_threshold,
    excluded_from_recording_fee
)
SELECT r.id, v.sport::akc_fee_sport, v.effective_year,
       v.recording_fee_first_entry, v.recording_fee_additional,
       v.service_fee_first_entry, v.service_fee_additional,
       v.event_secretary_fee, v.event_secretary_fee_threshold,
       v.excluded_from_recording_fee
FROM registries r
CROSS JOIN (VALUES
    -- JOVOB7 (10/25) Obedience/Conformation 2025 rates.
    ('obedience_conformation', 2025,
     0.50::numeric, 0.00::numeric,
     3.00::numeric, 3.50::numeric,
     10.00::numeric, 8,
     ARRAY[
         'junior_showmanship', 'sweepstakes', 'futurities', 'maturities',
         'brace', 'team', 'nonregular', 'special_attractions',
         'bitch_in_season_withdrawn', 'judge_change_withdrawn'
     ]),
    -- JOVOB7 (10/25) Obedience/Conformation 2026 rates. Service fee
    -- rises from $3.00/$3.50 to $4.00/$4.50 effective 2026.
    ('obedience_conformation', 2026,
     0.50::numeric, 0.00::numeric,
     4.00::numeric, 4.50::numeric,
     10.00::numeric, 8,
     ARRAY[
         'junior_showmanship', 'sweepstakes', 'futurities', 'maturities',
         'brace', 'team', 'nonregular', 'special_attractions',
         'bitch_in_season_withdrawn', 'judge_change_withdrawn'
     ]),
    -- JOVRY8 (10/25) Rally 2025 rates. Flat $3.50 per entry, no
    -- separate recording fee, secretary-fee threshold is 12 events.
    ('rally', 2025,
     NULL::numeric, NULL::numeric,
     3.50::numeric, 3.50::numeric,
     10.00::numeric, 12,
     ARRAY[
         'nonregular', 'special_attractions',
         'bitch_in_season_withdrawn', 'judge_change_withdrawn'
     ]),
    -- JOVRY8 (10/25) Rally 2026 rates. Flat rate rises to $4.50.
    ('rally', 2026,
     NULL::numeric, NULL::numeric,
     4.50::numeric, 4.50::numeric,
     10.00::numeric, 12,
     ARRAY[
         'nonregular', 'special_attractions',
         'bitch_in_season_withdrawn', 'judge_change_withdrawn'
     ])
) AS v(sport, effective_year,
       recording_fee_first_entry, recording_fee_additional,
       service_fee_first_entry, service_fee_additional,
       event_secretary_fee, event_secretary_fee_threshold,
       excluded_from_recording_fee)
WHERE r.code = 'AKC'
ON CONFLICT (registry_id, sport, effective_year) DO NOTHING;
