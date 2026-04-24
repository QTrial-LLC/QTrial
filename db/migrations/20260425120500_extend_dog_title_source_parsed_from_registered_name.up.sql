-- Extend the dog_title_source ENUM with parsed_from_registered_name.
--
-- DATA_MODEL.md §4 dog_titles lists four source values
-- (owner_entered, registry_verified, earned_in_qtrial,
-- parsed_from_registered_name); the Phase 0 migration at
-- 20260420120200_create_dogs_titles_and_participation.up.sql
-- defined only the first three. The fourth value is the source tag
-- used when the name parser (REQ-NAME-001) extracts a title from a
-- dog's registered name string rather than receiving it as a
-- discrete input.
--
-- Postgres 16 permits ALTER TYPE ADD VALUE inside a transaction
-- block as long as the new value is not USED in the same
-- transaction. Since this migration only adds the value without
-- inserting rows that reference it, a standard transactional
-- migration is sufficient; no -- no-transaction directive is
-- required.

ALTER TYPE dog_title_source ADD VALUE 'parsed_from_registered_name';
