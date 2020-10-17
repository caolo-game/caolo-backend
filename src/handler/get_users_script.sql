SELECT user_script.owner_id AS user_id ,
       user_script.name ,
       user_script.id AS script_id ,
       user_script.program AS payload
FROM user_script
INNER JOIN user_account ON user_script.owner_id=user_account.id
WHERE user_account.auth0_id=$1
    AND user_script.id=$2
