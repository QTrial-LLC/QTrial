-- Registry seed data.
--
-- Code format convention (applies to every seeded reference row):
--   {registry}_{sport}_{class_identifier}
-- examples: akc_rally_novice_a, akc_obed_utility_b
-- Registry codes themselves are bare uppercase tokens: "AKC", "UKC".
-- Display names are the registry's own preferred English rendering.
--
-- Idempotent via ON CONFLICT on the natural key `code`.

INSERT INTO registries (code, name)
VALUES
    -- American Kennel Club. Seeded for MVP (Obedience + Rally).
    ('AKC', 'American Kennel Club')
ON CONFLICT (code) DO NOTHING;
