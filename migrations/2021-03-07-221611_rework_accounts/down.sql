DROP TABLE user_account CASCADE;

CREATE TABLE user_account (
    id uuid DEFAULT uuid_generate_v4() NOT NULL,
    display_name character varying,
    email character varying,
    created timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone DEFAULT now() NOT NULL,
    auth0_id character varying NOT NULL,
    email_verified boolean DEFAULT false NOT NULL
);


ALTER TABLE user_account OWNER TO postgres;

--
-- Name: user_account auth0_id_is_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY user_account
    ADD CONSTRAINT auth0_id_is_unique UNIQUE (auth0_id);


--
-- Name: user_account email_is_unique; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY user_account
    ADD CONSTRAINT email_is_unique UNIQUE (email);


--
-- Name: user_account user_account_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY user_account
    ADD CONSTRAINT user_account_pkey PRIMARY KEY (id);


--
-- Name: user_account user_account_updated; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER user_account_updated AFTER UPDATE ON user_account FOR EACH ROW EXECUTE PROCEDURE set_updated_col();

