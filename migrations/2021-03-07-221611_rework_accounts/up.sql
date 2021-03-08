DROP TABLE user_account CASCADE;

CREATE TABLE user_account(
    id uuid DEFAULT uuid_generate_v4() NOT NULL,
    display_name character varying,
    email character varying,
    username character varying NOT NULL,
    pw character varying,
    salt character varying,
    token character varying,
    created timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone DEFAULT now() NOT NULL,
    email_verified boolean DEFAULT false NOT NULL
);

ALTER TABLE ONLY user_account
    ADD CONSTRAINT username_is_unique UNIQUE (username);

ALTER TABLE ONLY user_account
    ADD CONSTRAINT email_is_unique UNIQUE (email);

ALTER TABLE ONLY user_account
    ADD CONSTRAINT user_account_pkey PRIMARY KEY (id);

CREATE TRIGGER user_account_updated AFTER UPDATE ON user_account FOR EACH ROW EXECUTE PROCEDURE set_updated_col();
