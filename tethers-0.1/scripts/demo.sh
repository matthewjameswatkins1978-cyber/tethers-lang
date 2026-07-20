#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

cd "$root/engine-ocaml"
dune build

cd "$root/host-rust"
cargo run -- \
  "$root/engine-ocaml/_build/default/bin/main.exe" \
  "$root/protocol/request.json"
