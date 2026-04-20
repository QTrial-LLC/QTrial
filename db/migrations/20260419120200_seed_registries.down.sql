-- Seed-data down-migration: intentionally empty.
--
-- Reverting reference-data seeds is not a DELETE operation. If a seed
-- migration has to be undone in practice, the correct action is to
-- restore the database from a backup taken before the seed was
-- applied, not to issue DELETE statements that may cascade unexpectedly
-- into downstream tenant data. This file exists so the migration
-- directory stays in reversible mode; it is a deliberate no-op.

SELECT 1;
