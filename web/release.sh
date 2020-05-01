#/usr/bin/bash

set -e

cd /caolo

ls -al
diesel --version
diesel database setup

echo "Release command finished"
