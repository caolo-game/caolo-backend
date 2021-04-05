DROP TABLE world_hot;
DROP TABLE world_const;

CREATE TABLE world_output (
    id uuid DEFAULT uuid_generate_v4() NOT NULL,
    world_time bigint NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    payload jsonb NOT NULL,
    queen_tag character varying NOT NULL
);

ALTER TABLE world_output OWNER TO postgres;

--
-- Name: world_output world_output_pkey; Type: CONSTRAINT; Schema:  Owner: postgres
--

ALTER TABLE ONLY world_output
    ADD CONSTRAINT world_output_pkey PRIMARY KEY (id);

CREATE OR REPLACE FUNCTION on_world_ouput_insert () RETURNS TRIGGER
AS $$
BEGIN
    DELETE FROM world_output
    WHERE
        id NOT IN (
            SELECT foo.id
            FROM (
                SELECT id
                FROM world_output
                ORDER BY created DESC
                -- TODO this should consider the queen_tag as well...
                LIMIT 200
            ) foo
        );

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;



--
-- Name: world_output world_cleanup; Type: TRIGGER; Schema:  Owner: postgres
--

CREATE TRIGGER world_cleanup AFTER INSERT ON world_output FOR EACH STATEMENT EXECUTE PROCEDURE on_world_ouput_insert();
