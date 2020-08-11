ALTER TABLE user_script ADD COLUMN name VARCHAR;

UPDATE user_script
SET name='no name'
WHERE 1=1;

ALTER TABLE user_script ALTER COLUMN name SET NOT NULL;
