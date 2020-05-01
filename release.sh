#/usr/bin/bash

set -e

ls -al
diesel --version
diesel database setup

echo "Release command finished"
