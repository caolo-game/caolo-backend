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
