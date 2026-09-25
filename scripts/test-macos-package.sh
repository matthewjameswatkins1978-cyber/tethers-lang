#!/usr/bin/env bash
# Clean-package smoke for a Tethers macOS runtime (0.8 authority architecture).
#
# Usage: scripts/test-macos-package.sh <package-directory>
#
# The package directory is used exactly as an end user would use it: no source
# tree, Cargo target, or developer-home dependency.
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/test-macos-package.sh <package-directory>

Run smoke tests on an extracted macOS Tethers package directory.
EOF
}

fail() {
    printf 'TEST FAILED: %s\n' "$1" >&2
    exit 1
}

[[ $# -eq 1 ]] || { usage; exit 1; }
package_dir=$(cd "$1" && pwd -P)
[[ -d "$package_dir" ]] || fail "Package directory does not exist: $package_dir"

# Sanitise PATH: include standard system and Homebrew directories.
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
unset TETHERS_OCAML_SWITCH 2>/dev/null || true

tethers_bin="$package_dir/bin/tethers"
engine_bin="$package_dir/bin/tethers-engine"
[[ -f "$tethers_bin" && -x "$tethers_bin" ]] || fail "Missing executable: $tethers_bin"
[[ -f "$engine_bin" && -x "$engine_bin" ]] || fail "Missing executable: $engine_bin"

expected_version=$(tr -d '[:space:]' < "$package_dir/VERSION")

test_root=$(mktemp -d "${TMPDIR:-/tmp}/tethers-mac-smoke.XXXXXX")
trap 'rm -rf "$test_root"' EXIT
export XDG_STATE_HOME="$test_root/state"
mkdir -p "$XDG_STATE_HOME"

printf '=== TETHERS MACOS PACKAGE SMOKE ===\n'
printf 'Package: %s\n\n' "$package_dir"

printf 'Test 1: version matches packaged VERSION... '
version_output=$("$tethers_bin" --version 2>&1) || fail "version command failed: $version_output"
[[ "$version_output" == "tethers $expected_version" ]] || fail "unexpected version: '$version_output' (expected 'tethers $expected_version')"
printf 'PASS (%s)\n' "$version_output"

printf 'Test 2: describe reports capabilities as JSON... '
describe_output=$("$tethers_bin" describe --json 2>&1) || fail "describe failed: $describe_output"
printf '%s\n' "$describe_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value.get("status") == "ok", value' ||
    fail "describe did not return status=ok: $describe_output"
printf 'PASS\n'

printf 'Test 3: init creates a ready workspace... '
mkdir -p "$test_root/work"
init_output=$(cd "$test_root/work" && "$tethers_bin" init 2>&1) || fail "init failed: $init_output"
printf '%s\n' "$init_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value.get("status") == "ok" and value.get("data", {}).get("ready") is True, value' ||
    fail "init did not report a ready workspace: $init_output"
printf 'PASS\n'

printf 'Test 4: doctor reports healthy with packaged engine available... '
doctor_output=$(cd "$test_root/work" && "$tethers_bin" doctor 2>&1) || fail "doctor failed: $doctor_output"
printf '%s\n' "$doctor_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); data=value.get("data", {}); assert value.get("status") == "ok" and data.get("healthy") is True and data.get("engine", {}).get("available") is True, value' ||
    fail "doctor did not report healthy with engine available: $doctor_output"
printf 'PASS\n'

printf 'Test 5: capability list returns a valid contract... '
cap_output=$("$tethers_bin" capability list 2>&1) || fail "capability list failed"
printf '%s\n' "$cap_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value.get("status") == "ok", value' ||
    fail "capability list did not return status=ok"
printf 'PASS\n'

printf 'Test 6: workspace stat proves the directory... '
stat_output=$(cd "$package_dir" && "$tethers_bin" workspace stat --path . 2>&1) || fail "workspace stat failed: $stat_output"
printf '%s\n' "$stat_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); data=value.get("data", {}); assert value.get("status") == "ok" and data.get("exists") is True and data.get("kind") == "directory", value' ||
    fail "workspace stat did not prove the directory: $stat_output"
printf 'PASS\n'

printf 'Test 7: describe output is deterministic... '
first=$("$tethers_bin" describe --json) || fail "first describe failed"
second=$("$tethers_bin" describe --json) || fail "second describe failed"
[[ "$first" == "$second" ]] || fail "describe output changed between identical runs"
printf 'PASS\n'

printf 'Test 8: external authority smoke (ALLOW/ASK/DENY/hostile)... '
python3 "$package_dir/examples/external-consumer/smoke.py" \
    --tethers "$tethers_bin" --engine "$engine_bin" ||
    fail "external authority smoke failed"
printf 'PASS\n'

printf '\n=== ALL 8 MACOS PACKAGE SMOKE TESTS PASSED ===\n'
