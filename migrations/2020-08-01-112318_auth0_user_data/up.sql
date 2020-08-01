DROP TABLE user_credential;
DROP TABLE user_google_token;

DELETE FROM user_account WHERE 1=1;

ALTER TABLE user_account ADD CONSTRAINT email_is_unique UNIQUE (email);

ALTER TABLE user_account ADD COLUMN auth0_id VARCHAR NOT NULL;
CREATE UNIQUE INDEX auth0_id_idx ON user_account (auth0_id);
ALTER TABLE user_account ADD CONSTRAINT auth0_id_is_unique UNIQUE USING INDEX auth0_id_idx;

ALTER TABLE user_account ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT FALSE;
