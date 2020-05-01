#/usr/bin/bash

set -e

echo "Release command starting"

cd /caolo

ls -al
./diesel --version
./diesel database setup

echo "Release command finished"
