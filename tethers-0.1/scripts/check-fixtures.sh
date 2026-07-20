#!/usr/bin/env sh
set -eu

jq -e . protocol/request.json >/dev/null
jq -e . protocol/expected-response.json >/dev/null
echo "JSON fixtures are valid"
