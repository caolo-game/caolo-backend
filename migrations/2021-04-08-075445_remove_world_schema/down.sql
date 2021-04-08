--
-- Name: scripting_schema; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.scripting_schema (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    payload jsonb NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    queen_tag character varying NOT NULL
);


ALTER TABLE public.scripting_schema OWNER TO postgres;

--
-- Name: world_const; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.world_const (
    queen_tag character varying NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    payload jsonb NOT NULL
);


ALTER TABLE public.world_const OWNER TO postgres;

--
-- Name: scripting_schema scripting_schema_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.scripting_schema
    ADD CONSTRAINT scripting_schema_pkey PRIMARY KEY (id);


--
-- Name: scripting_schema scripting_schema_queen_tag_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.scripting_schema
    ADD CONSTRAINT scripting_schema_queen_tag_key UNIQUE (queen_tag);


--
-- Name: world_const world_const_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.world_const
    ADD CONSTRAINT world_const_pkey PRIMARY KEY (queen_tag);


--
-- PostgreSQL database dump complete
--

