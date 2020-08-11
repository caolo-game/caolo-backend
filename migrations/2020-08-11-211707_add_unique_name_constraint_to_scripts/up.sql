DELETE FROM user_script AS u1
USING user_script AS u2
WHERE u1.name=u2.name AND u1.owner_id=u2.owner_id;

ALTER TABLE user_script ADD CONSTRAINT name_owner_id_unique UNIQUE (name, owner_id);
