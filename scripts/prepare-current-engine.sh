#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/prepare-current-engine.sh [--release]

Build the current OCaml engine in the current checkout and write verified
provenance to verification/current-engine-provenance.json.

The OCaml switch is selected from TETHERS_OCAML_SWITCH, or from the prepared
workshop switch bl-tethers-5.5.0 when no override is supplied.
EOF
}

fail() {
    printf 'VERIFICATION PREREQUISITE FAILED\n%s\n' "$1" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "Required command is unavailable: $1"
}

release_mode=false
while (($# > 0)); do
    case "$1" in
        --release)
            release_mode=true
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            fail "Unknown argument: $1"
            ;;
    esac
    shift
done

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_dir/.." && pwd -P)
engine_root="$repository_root/tethers-0.1/engine-ocaml"
manifest_path="$repository_root/verification/current-engine-provenance.json"

require_command git
require_command opam
require_command jq
require_command sha256sum

git_root=$(git -C "$repository_root" rev-parse --show-toplevel 2>/dev/null) ||
    fail 'Could not identify the current Git worktree.'
[[ "$git_root" == "$repository_root" ]] ||
    fail 'The engine preparation script is not running inside the intended Git worktree.'

source_commit=$(git -C "$repository_root" rev-parse HEAD 2>/dev/null) ||
    fail 'Could not identify the current source commit.'
source_tree=$(git -C "$repository_root" rev-parse 'HEAD^{tree}' 2>/dev/null) ||
    fail 'Could not identify the current source tree.'

if [[ "$release_mode" == true ]] && [[ -n "$(git -C "$repository_root" status --porcelain=v1 --untracked-files=all)" ]]; then
    fail 'Release verification requires a clean Git worktree.'
fi

ocaml_switch=${TETHERS_OCAML_SWITCH:-bl-tethers-5.5.0}
ocaml_version=$(opam exec --switch="$ocaml_switch" -- ocamlc -version 2>/dev/null) ||
    fail "Could not run OCaml from switch: $ocaml_switch"
dune_version=$(opam exec --switch="$ocaml_switch" -- dune --version 2>/dev/null) ||
    fail "Could not run Dune from switch: $ocaml_switch"

[[ -d "$engine_root" ]] || fail "OCaml engine source directory is missing: tethers-0.1/engine-ocaml"

if ! (cd -- "$engine_root" && opam exec --switch="$ocaml_switch" -- dune build '@all'); then
    fail "The current OCaml engine could not be built from source commit $source_commit. Cross-language host tests were not attempted."
fi

engine_path=''
for candidate in \
    "$engine_root/_build/default/bin/tethers_mcp_main" \
    "$engine_root/_build/default/bin/tethers_mcp_main.exe"; do
    if [[ -f "$candidate" && -x "$candidate" ]]; then
        engine_path=$(cd -- "$(dirname -- "$candidate")" && pwd -P)/$(basename -- "$candidate")
        break
    fi
done
[[ -n "$engine_path" ]] ||
    fail 'The current OCaml build completed without producing the tethers_mcp_main executable.'

binary_relative_path=${engine_path#"$repository_root/"}
[[ "$binary_relative_path" != "$engine_path" ]] ||
    fail 'The built engine was not produced inside the current repository.'
binary_sha256=$(sha256sum "$engine_path" | awk '{print $1}')

mkdir -p -- "$(dirname -- "$manifest_path")"
temporary_manifest="$manifest_path.tmp.$$"
jq -n \
    --arg schema 'tethers.engine/1' \
    --arg status 'pass' \
    --arg source_commit "$source_commit" \
    --arg source_tree "$source_tree" \
    --arg binary_relative_path "$binary_relative_path" \
    --arg binary_sha256 "$binary_sha256" \
    --arg ocaml_version "$ocaml_version" \
    --arg dune_version "$dune_version" \
    '{schema: $schema, status: $status, source_commit: $source_commit,
      source_tree: $source_tree, binary_relative_path: $binary_relative_path,
      binary_sha256: $binary_sha256, ocaml_version: $ocaml_version,
      dune_version: $dune_version}' > "$temporary_manifest"
mv -- "$temporary_manifest" "$manifest_path"

printf 'PASS current OCaml engine: %s\n' "$binary_relative_path"
printf 'PASS engine source commit: %s\n' "$source_commit"
printf 'PASS engine source tree: %s\n' "$source_tree"
printf 'PASS engine SHA-256: %s\n' "$binary_sha256"
printf 'PASS OCaml version: %s\n' "$ocaml_version"
printf 'PASS Dune version: %s\n' "$dune_version"
printf 'PASS provenance manifest: verification/current-engine-provenance.json\n'
