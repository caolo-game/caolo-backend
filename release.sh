#!/bin/bash

set -e


echo "Waiting for database connection"

echo $DATABASE_URL > dburl.txt
# curl doesn't support postgres://
sed -i 's/postgres:\/\//http:\/\//' dburl.txt

# wait until the server sends an empty response, indicating it is up and running
# I noticed that diesel tends to hang if it has to wait for the DB to start, 
# thus this hackerino
ts=0
while ! cat dburl.txt | xargs -n1 curl  2>&1 | grep 'curl: (52)'
do
    # exponential backoff
    echo Sleeping for $((1<<ts)) seconds
    sleep $((1 << ts))
    ts=$((ts+1))
done

echo "Release command starting"

/caolo/diesel migration run

echo "Release command finished"
