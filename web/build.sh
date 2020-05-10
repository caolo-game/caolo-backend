#/usr/bin/bash

set -e

service postgresql start &

timer="1"
until runuser -l postgres -c 'pg_isready' 2>/dev/null; do
  >&2 echo "Postgres is unavailable - sleeping for $timer seconds"
  sleep $timer
done

sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"

bin/diesel database setup

cargo check

cargo install --path . --root . --no-default-features --features="with-serde"
