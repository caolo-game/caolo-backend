CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE OR REPLACE FUNCTION set_updated_col()   
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated = now();
    RETURN NEW;   
END;
$$ language 'plpgsql';

CREATE TABLE user_account (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    display_name VARCHAR NULL,
    email VARCHAR NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE TRIGGER user_account_updated AFTER UPDATE ON user_account 
    FOR EACH ROW EXECUTE PROCEDURE set_updated_col();

CREATE TABLE user_credential (
    user_id UUID PRIMARY KEY REFERENCES user_account(id),
    token VARCHAR NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE TRIGGER user_credential_updated AFTER UPDATE ON user_credential 
    FOR EACH ROW EXECUTE PROCEDURE set_updated_col();
