DROP INDEX IF EXISTS breed_varieties_breed_id_ix;
DROP INDEX IF EXISTS breed_varieties_breed_name_uk;
DROP TABLE IF EXISTS breed_varieties;

DROP INDEX IF EXISTS breeds_breed_group_id_ix;
DROP INDEX IF EXISTS breeds_registry_id_ix;
DROP INDEX IF EXISTS breeds_registry_name_uk;
DROP TABLE IF EXISTS breeds;

DROP INDEX IF EXISTS breed_groups_registry_id_ix;
DROP INDEX IF EXISTS breed_groups_registry_group_uk;
DROP TABLE IF EXISTS breed_groups;
