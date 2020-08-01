ALTER TABLE user_account DROP COLUMN auth0_id;
ALTER TABLE user_account DROP COLUMN email_verified;
ALTER TABLE user_account DROP CONSTRAINT email_is_unique;

CREATE OR REPLACE FUNCTION set_updated_col()   
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated = now();
    RETURN NEW;   
END;
$$ language 'plpgsql';

CREATE TABLE user_credential (
    user_id UUID PRIMARY KEY REFERENCES user_account(id),
    token VARCHAR NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE TRIGGER user_credential_updated AFTER UPDATE ON user_credential 
    FOR EACH ROW EXECUTE PROCEDURE set_updated_col();

CREATE TABLE user_google_token (
    google_id VARCHAR(30) NOT NULL,
    user_id UUID REFERENCES user_account(id),
    access_token VARCHAR NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),

    PRIMARY KEY (google_id, user_id)
);

CREATE TRIGGER user_google_token_updated AFTER UPDATE ON user_google_token 
    FOR EACH ROW EXECUTE PROCEDURE set_updated_col();
