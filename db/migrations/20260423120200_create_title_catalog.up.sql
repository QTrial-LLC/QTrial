-- Title catalog: title_prefixes and title_suffixes.
--
-- A dog's registered name is rendered as:
--     <prefix titles space-separated> <registered name>, <suffix titles comma-separated>
-- e.g. "CH OTCH Rocky's Ruby Slippers CDX GO RA CGC".
--
-- The two tables share an identical schema; the distinction is
-- positional. Every row is identified by (source_organization, code)
-- rather than by registry, because non-AKC titles (Barn Hunt, and in
-- future others) may not map to any AKC-sanctioned registry.
--
-- MVP seed scope per Deborah's Q2 (2026-04-20):
--   * 49 AKC prefix titles
--   * 244 AKC suffix titles + 5 legacy compound suffix titles + 10
--     Barn Hunt suffix titles (259 rows total)
-- The 81 non-AKC suffix titles present in the seed package are
-- preserved on disk but not loaded; they return to scope post-MVP
-- when other non-AKC organizations are added.
--
-- earning_rules is a structured JSONB column reserved for rules the
-- title-automation path (REQ-NAME-001 / future auto-title-award) will
-- consume. The schema is deliberately undesigned in this PR: rows are
-- seeded with NULL and the column is filled when title progression
-- lands.
--
-- display_order is nullable. title_suffixes.csv has it; title_prefixes
-- does not. Leaving the column NULL for prefix rows is intentional
-- and preserves room for future ordering without a schema change.

CREATE TABLE title_prefixes (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Nullable because Barn Hunt titles and future non-AKC titles have
    -- no QTrial-managed registry row. AKC titles set this to the AKC
    -- registry UUID; the loader resolves that once at startup.
    registry_id               UUID REFERENCES registries(id),
    -- Legacy PrefixID from Deborah's Access database. Preserved for
    -- migration continuity when importing from existing Obedience
    -- Solution databases.
    legacy_id                 INT,
    code                      TEXT NOT NULL,
    long_name                 TEXT,
    sport_scope_code          TEXT,
    sport_scope_description   TEXT,
    -- Free-text issuing body. Defaults to 'AKC' because 49 of 49 seeded
    -- prefix rows are AKC-sanctioned; Barn Hunt rows override to
    -- 'Barn Hunt Association'.
    source_organization       TEXT NOT NULL DEFAULT 'AKC',
    display_order             INT,
    earning_rules             JSONB,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Natural unique key: code is unique within a given issuing body. Two
-- organizations may legitimately use the same code for different
-- titles, so (source_organization, code) is the right grain.
CREATE UNIQUE INDEX title_prefixes_source_code_uk
    ON title_prefixes (source_organization, code);
CREATE INDEX title_prefixes_registry_id_ix
    ON title_prefixes (registry_id) WHERE registry_id IS NOT NULL;

CREATE TABLE title_suffixes (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_id               UUID REFERENCES registries(id),
    legacy_id                 INT,
    code                      TEXT NOT NULL,
    long_name                 TEXT,
    sport_scope_code          TEXT,
    sport_scope_description   TEXT,
    source_organization       TEXT NOT NULL DEFAULT 'AKC',
    display_order             INT,
    earning_rules             JSONB,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX title_suffixes_source_code_uk
    ON title_suffixes (source_organization, code);
CREATE INDEX title_suffixes_registry_id_ix
    ON title_suffixes (registry_id) WHERE registry_id IS NOT NULL;
