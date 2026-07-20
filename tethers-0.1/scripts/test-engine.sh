#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root/engine-ocaml"
dune build

actual=$(mktemp)
trap 'rm -f "$actual"' EXIT

jq -c . "$root/protocol/request.json" \
  | "$root/engine-ocaml/_build/default/bin/main.exe" \
  | jq -S . >"$actual"
jq -S . "$root/protocol/expected-response.json" | diff -u - "$actual"

echo "Engine response matches the frozen fixture"
