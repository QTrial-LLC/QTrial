-- Add dogs.registration_type ENUM and column.
--
-- DATA_MODEL.md §4 has listed this column since v0.2; the Phase 0
-- migration at 20260420120200 didn't include it, and PR 2c-beta
-- closes that gap.
--
-- The five enum values cover AKC's registration categories:
--   * akc_purebred     - standard AKC-registered purebred
--   * pal              - Purebred Alternative Listing (formerly ILP):
--                        unregistered but identifiable purebred,
--                        eligible for performance events
--   * canine_partners  - AKC Canine Partners program for mixed-breed
--                        and unregistered dogs; also called the
--                        "All-American Dog" program
--   * fss              - Foundation Stock Service: rare breeds in
--                        the early-recognition pipeline
--   * misc             - Miscellaneous class: breeds moving toward
--                        full recognition (e.g. Bracco Italiano was
--                        here before 2022)
--
-- Nullable, no default. NULL means "registration type not yet known."
-- The Access import path (tblDogData -> dogs, per DATA_MODEL.md §11)
-- will produce rows where type is genuinely unrecoverable; forcing a
-- default would invent truth. The UI treats NULL as a first-class
-- "Unknown" state with a correction affordance. New dogs entered
-- through the app layer are NOT NULL at the form layer; the schema
-- permits NULL specifically for the import path.

CREATE TYPE dog_registration_type AS ENUM (
    'akc_purebred',
    'pal',
    'canine_partners',
    'fss',
    'misc'
);

ALTER TABLE dogs
    ADD COLUMN registration_type dog_registration_type;
