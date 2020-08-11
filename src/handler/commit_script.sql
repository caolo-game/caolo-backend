INSERT INTO user_script
(program, name, owner_id)
VALUES
(
    $1
    , $2
    , (
        SELECT id AS owner_id
        FROM user_account
        WHERE auth0_id=$3
        LIMIT 1
    )
)
ON CONFLICT ON CONSTRAINT name_owner_id_unique 
DO UPDATE
    SET program=$1
RETURNING id, owner_id
