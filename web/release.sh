#!/bin/bash

set -e

echo "Release command starting"

cd /caolo

ls -al
./diesel migration run

echo "Release command finished"
