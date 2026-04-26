-- Drop clubs.officers_json. Any officer data stored in the column
-- is lost on rollback; production data (none yet) would need to be
-- re-entered through the admin tooling.

ALTER TABLE clubs DROP COLUMN officers_json;
