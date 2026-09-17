#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/test-linux-package.sh <package-directory>

Run smoke tests on a packaged Linux Tethers release.
EOF
}

fail() {
    printf 'TEST FAILED: %s\n' "$1" >&2
    exit 1
}

if [[ $# -ne 1 ]]; then
    usage
    exit 1
fi

package_dir=$(realpath "$1")
[[ -d "$package_dir" ]] || fail "Package directory does not exist: $package_dir"
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
unset TETHERS_OCAML_SWITCH 2>/dev/null || true

tethers_bin=""
for candidate in "$package_dir/tethers" "$package_dir/bin/tethers"; do
    if [[ -f "$candidate" && -x "$candidate" ]]; then
        tethers_bin="$candidate"
        break
    fi
done
[[ -n "$tethers_bin" ]] || fail "Could not find tethers binary in package"

engine_bin=""
for candidate in "$package_dir/tethers-engine" "$package_dir/bin/tethers-engine"; do
    if [[ -f "$candidate" && -x "$candidate" ]]; then
        engine_bin="$candidate"
        break
    fi
done
[[ -n "$engine_bin" ]] || fail "Could not find tethers-engine binary in package"

test_root=$(mktemp -d)
trap 'rm -rf "$test_root"' EXIT
export XDG_STATE_HOME="$test_root/state"
mkdir -p "$XDG_STATE_HOME"

run_json_ok() {
    local label=$1
    shift
    local output parsed
    if ! output=$("$@" 2>&1); then
        printf 'FAIL\n'
        fail "$label exited non-zero: $output"
    fi
    if ! parsed=$(printf '%s\n' "$output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value.get("status") == "ok"; print(value.get("schema", ""))'); then
        printf 'FAIL\n'
        fail "$label did not return a JSON status=ok envelope: $output"
    fi
    [[ -n "$parsed" ]] || fail "$label returned no schema"
    printf 'PASS\n'
}

printf '=== TETHERS LINUX PACKAGE SMOKE TESTS ===\n'
printf 'Package directory: %s\n' "$package_dir"
printf 'Tethers binary: %s\n' "$tethers_bin"
printf 'Engine binary: %s\n\n' "$engine_bin"

cd "$package_dir"

printf 'Test 1: executable starts and identifies product... '
version_output=$("$tethers_bin" --version 2>&1) || fail "version command failed: $version_output"
[[ "$version_output" == "tethers 0.7.1" ]] || fail "unexpected version: $version_output"
printf 'PASS\n'

printf 'Test 2: describe --json... '
run_json_ok "describe" "$tethers_bin" describe --json

printf 'Test 3: init creates a ready workspace... '
init_output=$("$tethers_bin" init 2>&1) || fail "init failed: $init_output"
printf '%s\n' "$init_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["status"] == "ok" and value["data"]["ready"] is True and value["data"]["engine"]["available"] is True' || fail "init did not report a ready workspace: $init_output"
printf 'PASS\n'

printf 'Test 4: doctor reports healthy package... '
doctor_output=$("$tethers_bin" doctor 2>&1) || fail "doctor failed: $doctor_output"
printf '%s\n' "$doctor_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["status"] == "ok" and value["data"]["healthy"] is True and value["data"]["engine"]["available"] is True' || fail "doctor did not report healthy: $doctor_output"
printf 'PASS\n'

printf 'Test 5: capability discovery returns a valid contract... '
run_json_ok "capability list" "$tethers_bin" capability list

printf 'Test 6: workspace stat returns the package directory... '
stat_output=$("$tethers_bin" workspace stat --path . 2>&1) || fail "workspace stat failed: $stat_output"
printf '%s\n' "$stat_output" | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["status"] == "ok" and value["data"]["exists"] is True and value["data"]["kind"] == "directory"' || fail "workspace stat did not prove the directory: $stat_output"
printf 'PASS\n'

printf 'Test 7: deterministic discovery output... '
describe_one=$("$tethers_bin" describe --json) || fail "first describe failed"
describe_two=$("$tethers_bin" describe --json) || fail "second describe failed"
[[ "$describe_one" == "$describe_two" ]] || fail "describe output changed between identical runs"
printf 'PASS\n'

printf 'Test 8: provider startup and shutdown... '
provider_fifo="$test_root/provider.fifo"
mkfifo "$provider_fifo"
exec {provider_fd}<>"$provider_fifo"
scope_json=$(printf '{"query_root":"%s","move_source_root":"%s","move_destination_root":"%s","max_content_bytes":65536}' "$package_dir" "$package_dir" "$package_dir")
provider_log="$test_root/provider.log"
TETHERS_OPERATIONAL_SCOPE_JSON="$scope_json" "$package_dir/agent_workspace_provider" <&$provider_fd >"$provider_log" 2>&1 &
provider_pid=$!
sleep 1
if ! kill -0 "$provider_pid" 2>/dev/null; then
    set +e
    wait "$provider_pid"
    provider_status=$?
    set -e
    fail "provider exited before startup (status $provider_status): $(cat "$provider_log")"
fi
kill -TERM "$provider_pid" 2>/dev/null || fail "could not request provider shutdown"
set +e
wait "$provider_pid"
provider_status=$?
set -e
exec {provider_fd}>&-
[[ "$provider_status" -eq 0 || "$provider_status" -eq 143 ]] || fail "provider shutdown returned status $provider_status: $(cat "$provider_log")"
if kill -0 "$provider_pid" 2>/dev/null; then
    fail "provider remained alive after shutdown"
fi
printf 'PASS\n'

printf 'Test 9: restart after normal operation... '
"$tethers_bin" --help >/dev/null || fail "first restart invocation failed"
"$tethers_bin" --help >/dev/null || fail "second restart invocation failed"
printf 'PASS\n'

printf '\n=== ALL 9 SMOKE TESTS PASSED ===\n'
