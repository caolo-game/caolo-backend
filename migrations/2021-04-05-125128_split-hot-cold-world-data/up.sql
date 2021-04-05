DROP TABLE world_output;

CREATE TABLE world_hot (
    id uuid DEFAULT uuid_generate_v4() NOT NULL,
    queen_tag character varying NOT NULL,
    world_time bigint NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    payload jsonb NOT NULL,

    PRIMARY KEY (queen_tag, world_time)
);

CREATE OR REPLACE FUNCTION on_world_ouput_insert () RETURNS TRIGGER
AS $$
BEGIN
    DELETE FROM world_hot
    WHERE
        id NOT IN (
            SELECT foo.id
            FROM (
                SELECT id
                FROM world_hot
                ORDER BY created DESC
                -- TODO this should consider the queen_tag as well...
                LIMIT 200
            ) foo
        );

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER world_cleanup AFTER INSERT ON world_hot FOR EACH STATEMENT EXECUTE PROCEDURE on_world_ouput_insert();

CREATE TABLE world_const (
    queen_tag character varying PRIMARY KEY NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    payload jsonb NOT NULL
);

