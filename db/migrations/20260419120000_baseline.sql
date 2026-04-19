-- Phase 0 baseline migration.
--
-- This file exists only to prove the sqlx migration runner finds and
-- applies something end to end. It intentionally defines no schema.
-- Real reference-data and tenant tables land in later migrations as
-- Phase 0's DATA_MODEL.md work is implemented.
--
-- Not reversible: there is nothing to undo. Future schema-modifying
-- migrations follow CLAUDE.md's reversibility rule.

SELECT 1;
