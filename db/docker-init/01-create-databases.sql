-- Local development bootstrap for the shared Postgres instance.
--
-- The Postgres container runs this exactly once, on an empty data
-- directory, as the superuser. It creates two isolated tenants on one
-- server: `qtrial` holds the application schema, `keycloak` holds the
-- identity provider's own tables. Each has its own role so a compromise
-- of one cannot touch the other.
--
-- Passwords here are fine only because this file is dev-only. Production
-- credentials live in Secrets Manager per ARCHITECTURE.md.

CREATE ROLE qtrial WITH LOGIN PASSWORD 'qtrial';
CREATE DATABASE qtrial OWNER qtrial;

CREATE ROLE keycloak WITH LOGIN PASSWORD 'keycloak';
CREATE DATABASE keycloak OWNER keycloak;
