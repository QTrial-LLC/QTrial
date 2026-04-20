-- Baseline down-migration.
--
-- The baseline up-migration is `SELECT 1;` and defines no schema. There
-- is nothing to reverse. This file exists so the directory is in
-- reversible mode (sqlx requires a consistent format across all
-- migrations in a directory).

SELECT 1;
