-- Recreate the four sire/dam prefix/suffix columns byte-identically
-- to the Phase 0 definitions
-- (20260420120200_create_dogs_titles_and_participation.up.sql
-- lines 51, 53, 54, 56): nullable TEXT, no default. Column add
-- order mirrors the up's drop order for symmetry, though ALTER
-- TABLE does not depend on column order semantically.

ALTER TABLE dogs
    ADD COLUMN sire_prefix_titles TEXT,
    ADD COLUMN sire_suffix_titles TEXT,
    ADD COLUMN dam_prefix_titles TEXT,
    ADD COLUMN dam_suffix_titles TEXT;
