ALTER TABLE clubs
    DROP CONSTRAINT IF EXISTS clubs_updated_by_fk,
    DROP CONSTRAINT IF EXISTS clubs_created_by_fk,
    DROP CONSTRAINT IF EXISTS clubs_primary_contact_fk;

DROP INDEX IF EXISTS users_keycloak_sub_uk;
DROP INDEX IF EXISTS users_email_uk;
DROP TABLE IF EXISTS users;
-- citext extension left in place; other tables may still use CITEXT.
