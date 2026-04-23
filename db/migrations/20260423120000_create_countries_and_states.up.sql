-- Countries and states reference tables.
--
-- Geographic reference data shared across all tenants. An owner's
-- mailing address (owners.country_code, owners.state) today stores
-- plain text codes; once the reference data is in place, later PRs can
-- migrate those columns to FKs and enforce that an owner's state
-- belongs to its country.
--
-- Neither table is tenant-scoped; permissive-read RLS is enabled in
-- a later migration alongside the other reference tables.
--
-- Natural keys: countries.alpha2_code and (alpha3_code); states
-- (country_id, code). The UUID primary key is the stable FK target;
-- natural keys give the seed loader a stable ON CONFLICT target that
-- does not depend on row order.

CREATE TABLE countries (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ISO 3166-1 alpha-2 code, e.g. "US", "CA", "GB". The loader's
    -- ON CONFLICT natural key.
    alpha2_code    TEXT NOT NULL,
    -- ISO 3166-1 alpha-3 code, e.g. "USA", "CAN", "GBR". Also unique so
    -- consumers that prefer the longer code can FK against it via a
    -- lookup without ambiguity.
    alpha3_code    TEXT NOT NULL,
    display_name   TEXT NOT NULL,
    -- Catalog ordering hint from Deborah's Access database. Nullable
    -- because a future ISO-based re-seed may not populate it.
    display_order  INT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX countries_alpha2_code_uk ON countries (alpha2_code);
CREATE UNIQUE INDEX countries_alpha3_code_uk ON countries (alpha3_code);

CREATE TABLE states (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Legacy row ID from Deborah's Access database (tblStates.StateID).
    -- Preserved as a non-key column for migration continuity; new rows
    -- created in QTrial may leave it NULL.
    legacy_id   INT,
    country_id  UUID NOT NULL REFERENCES countries(id),
    -- Two-letter state or province code, e.g. "NY", "CA", "ON". No
    -- human-readable display_name column yet: the current seed CSV only
    -- contains the code. A later migration will add display_name from
    -- a hardcoded US + CA lookup; until then, catalogs render the code.
    code        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX states_country_code_uk ON states (country_id, code);
CREATE INDEX states_country_id_ix ON states (country_id);
