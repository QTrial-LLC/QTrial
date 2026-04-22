-- Local development bootstrap for the shared Postgres instance.
--
-- The Postgres container runs this exactly once, on an empty data
-- directory, as the superuser. It creates:
--
-- * Two isolated databases on one server: `qtrial` holds the
--   application schema, `keycloak` holds the identity provider's own
--   tables. Each has its own owning role so a compromise of one
--   cannot touch the other.
-- * A NOLOGIN role `qtrial_tenant` that the API assumes via
--   `SET LOCAL ROLE` inside a transaction. RLS policies on tenant
--   tables are written against this role, so application connections
--   only see the current-tenant slice of data. Platform admin paths
--   skip the SET LOCAL ROLE and run as the table owner (`qtrial`),
--   which bypasses RLS by Postgres convention.
--
-- Passwords here are fine only because this file is dev-only. Production
-- credentials live in Secrets Manager per ARCHITECTURE.md.

CREATE ROLE qtrial WITH LOGIN PASSWORD 'qtrial';
CREATE DATABASE qtrial OWNER qtrial;

CREATE ROLE keycloak WITH LOGIN PASSWORD 'keycloak';
CREATE DATABASE keycloak OWNER keycloak;

CREATE ROLE qtrial_tenant NOLOGIN;

-- `qtrial` owns every application table and runs migrations. To
-- downgrade into the tenant role inside a transaction (SET LOCAL ROLE
-- qtrial_tenant), `qtrial` must be a member of `qtrial_tenant`.
-- Granting membership from the superuser session is the standard way
-- to enable that.
GRANT qtrial_tenant TO qtrial;
