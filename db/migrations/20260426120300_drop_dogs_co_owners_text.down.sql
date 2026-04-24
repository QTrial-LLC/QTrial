-- Recreate dogs.co_owners_text byte-identically to the Phase 0
-- definition (20260420120200_create_dogs_titles_and_participation
-- .up.sql line 60): nullable TEXT, no default.

ALTER TABLE dogs ADD COLUMN co_owners_text TEXT;
