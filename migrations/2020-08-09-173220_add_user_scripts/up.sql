CREATE TABLE user_script (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID REFERENCES user_account(id),
    program JSON NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE TRIGGER user_script_updated AFTER UPDATE ON user_script 
    FOR EACH ROW EXECUTE PROCEDURE set_updated_col();
