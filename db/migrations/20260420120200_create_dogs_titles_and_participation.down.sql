DROP INDEX IF EXISTS dog_sport_participation_club_id_ix;
DROP TABLE IF EXISTS dog_sport_participation;

DROP INDEX IF EXISTS dog_titles_dog_code_uk;
DROP INDEX IF EXISTS dog_titles_dog_id_ix;
DROP INDEX IF EXISTS dog_titles_club_id_ix;
DROP TABLE IF EXISTS dog_titles;
DROP TYPE IF EXISTS dog_title_source;
DROP TYPE IF EXISTS dog_title_category;

DROP INDEX IF EXISTS dogs_club_registry_number_uk;
DROP INDEX IF EXISTS dogs_owner_id_ix;
DROP INDEX IF EXISTS dogs_club_id_ix;
DROP TABLE IF EXISTS dogs;
DROP TYPE IF EXISTS dog_sex;
