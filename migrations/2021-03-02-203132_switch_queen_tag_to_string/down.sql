DELETE FROM world_output;
ALTER TABLE world_output DROP COLUMN queen_tag;
ALTER TABLE world_output ADD COLUMN queen_tag UUID NOT NULL;

DELETE FROM scripting_schema;
ALTER TABLE scripting_schema DROP COLUMN queen_tag;
ALTER TABLE scripting_schema ADD COLUMN queen_tag UUID NOT NULL UNIQUE;
