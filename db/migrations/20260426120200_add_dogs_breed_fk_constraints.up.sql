-- Close the Phase 0 deferred-FK loop on dogs.breed_id and
-- dogs.breed_variety_id. The original Phase 0 migration
-- (20260420120200_create_dogs_titles_and_participation.up.sql
-- header lines 8-13) deliberately left both columns nullable and
-- unconstrained because the breeds reference table hadn't shipped
-- yet. PR 2a landed breeds and breed_varieties; PR 2c-beta adds
-- the constraints.
--
-- No ON DELETE clause so the default RESTRICT applies. A breed row
-- referenced by even one dog cannot be deleted; matches the
-- "reference-data-shouldn't-be-deletable-while-referenced" posture
-- the rest of the schema uses for registry-scoped reference tables.
-- breeds and breed_varieties are seed-loader-populated reference
-- tables; hard-deleting a row is an admin-level action that should
-- require a deliberate unlinking of every referring dog first.
--
-- Explicit constraint names match Postgres's default auto-naming
-- convention (<table>_<column>_fkey) so future tooling that expects
-- the auto-named form finds them.
--
-- Both columns are nullable and may carry NULL values. Postgres does
-- not validate FK on NULL, so existing rows that left the columns
-- NULL (including every test fixture) continue to work. The
-- constraint kicks in only when the app layer supplies a value.

ALTER TABLE dogs
    ADD CONSTRAINT dogs_breed_id_fkey
        FOREIGN KEY (breed_id) REFERENCES breeds(id),
    ADD CONSTRAINT dogs_breed_variety_id_fkey
        FOREIGN KEY (breed_variety_id) REFERENCES breed_varieties(id);
