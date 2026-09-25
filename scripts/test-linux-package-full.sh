#!/usr/bin/env bash
# Archive-level proof for a Tethers Linux runtime: extraction paths,
# permissions, path independence, native dependency audit, provider
# supervision with orphan detection, and restart/recovery behaviour.
#
# Usage: scripts/test-linux-package-full.sh <archive-path>
#
# The provider lifecycle test uses a provider binary built from the same
# source (CI builds --bin agent_workspace_provider and exports
# TETHERS_TEST_PROVIDER_EXE); the shipped runtime itself stays minimal.
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/test-linux-package-full.sh <archive-path>

Run path, permission, supervision, dependency, and recovery tests on a Linux package archive.
Environment:
  TETHERS_TEST_PROVIDER_EXE  Path to a same-source provider binary for lifecycle proof.
EOF
}

fail() {
    printf 'TEST FAILED: %s\n' "$1" >&2
    exit 1
}

[[ $# -eq 1 ]] || { usage; exit 1; }
archive_path=$(realpath "$1")
[[ -f "$archive_path" ]] || fail "Archive does not exist: $archive_path"
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
unset TETHERS_OCAML_SWITCH 2>/dev/null || true

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "Required command is unavailable: $1"
}
require_command tar
require_command ldd
require_command python3

test_root=$(mktemp -d "${TMPDIR:-/tmp}/tethers-linux-test.XXXXXX")
trap 'rm -rf "$test_root"' EXIT

extract_package() {
    local destination=$1
    mkdir -p "$destination"
    tar xzf "$archive_path" -C "$destination"
    find "$destination" -type f -name tethers -executable -print -quit
}

printf '=== TETHERS LINUX PACKAGE FULL TESTS ===\nArchive: %s\n\n' "$archive_path"

printf 'Test 1: ordinary temp path... '
normal_bin=$(extract_package "$test_root/normal")
[[ -n "$normal_bin" ]] || fail "normal extraction did not contain executable"
"$normal_bin" --help >/dev/null || fail "normal path executable failed"
printf 'PASS\n'

printf 'Test 2: path with spaces... '
spaces_bin=$(extract_package "$test_root/path with spaces")
[[ -n "$spaces_bin" ]] || fail "space-path extraction did not contain executable"
"$spaces_bin" --help >/dev/null || fail "space-path executable failed"
printf 'PASS\n'

printf 'Test 3: nested and Unicode paths... '
nested_bin=$(extract_package "$test_root/a/b/c/テスト")
[[ -n "$nested_bin" ]] || fail "nested-path extraction did not contain executable"
"$nested_bin" --help >/dev/null || fail "nested-path executable failed"
printf 'PASS\n'

printf 'Test 4: executable permissions... '
perm_dir="$test_root/permissions"
extract_package "$perm_dir" >/dev/null
for bin_name in tethers tethers-engine; do
    bin_path=$(find "$perm_dir" -type f -name "$bin_name" -print -quit)
    [[ -n "$bin_path" && -x "$bin_path" ]] || fail "$bin_name is missing or not executable"
done
printf 'PASS\n'

printf 'Test 5: no developer-checkout path dependence... '
independent_bin=$(extract_package "$test_root/independent")
[[ -n "$independent_bin" ]] || fail "independent extraction did not contain executable"
if strings "$independent_bin" | grep -Fq 'C:\\'; then fail "binary contains a Windows path"; fi
if strings "$independent_bin" | grep -Fq '/home/'; then fail "binary contains a Linux home path"; fi
printf 'PASS\n'

printf 'Test 6: runtime dependency audit... '
deps_dir="$test_root/deps"
extract_package "$deps_dir" >/dev/null
audit_failed=false
while IFS= read -r native; do
    printf '  auditing %s\n' "$native"
    ldd_output=$(ldd "$native" 2>&1) || { printf '  ldd failed for %s:\n%s\n' "$native" "$ldd_output"; audit_failed=true; continue; }
    printf '%s\n' "$ldd_output"
    if printf '%s\n' "$ldd_output" | grep -Fq 'not found'; then
        printf '  missing library for %s\n' "$native"
        audit_failed=true
    fi
done < <(find "$deps_dir" -type f \( -name tethers -o -name tethers-engine \) -executable)
[[ "$audit_failed" == false ]] || fail "dependency audit found missing libraries"
printf 'PASS\n'

printf 'Test 7: provider supervision with no orphan... '
[[ -n "${TETHERS_TEST_PROVIDER_EXE:-}" ]] || fail "TETHERS_TEST_PROVIDER_EXE is not set; build --bin agent_workspace_provider from the same source first."
[[ -x "$TETHERS_TEST_PROVIDER_EXE" ]] || fail "provider binary is not executable: $TETHERS_TEST_PROVIDER_EXE"
supervisor_dir="$test_root/supervisor"
mkdir -p "$supervisor_dir"
provider_fifo="$test_root/supervisor.fifo"
mkfifo "$provider_fifo"
exec {provider_fd}<>"$provider_fifo"
scope_json=$(printf '{"query_root":"%s","move_source_root":"%s","move_destination_root":"%s","max_content_bytes":65536}' "$supervisor_dir" "$supervisor_dir" "$supervisor_dir")
provider_log="$test_root/supervisor.log"
TETHERS_OPERATIONAL_SCOPE_JSON="$scope_json" "$TETHERS_TEST_PROVIDER_EXE" <&$provider_fd >"$provider_log" 2>&1 &
provider_pid=$!
sleep 1
kill -0 "$provider_pid" 2>/dev/null || fail "provider did not remain alive after startup: $(cat "$provider_log")"
kill -TERM "$provider_pid" || fail "provider termination request failed"
set +e
wait "$provider_pid"
provider_status=$?
set -e
exec {provider_fd}>&-
[[ "$provider_status" -eq 0 || "$provider_status" -eq 143 ]] || fail "provider did not stop cleanly: $provider_status"
if command -v pgrep >/dev/null 2>&1; then
    pgrep -f -- "$TETHERS_TEST_PROVIDER_EXE" >/dev/null 2>&1 &&
        fail "provider orphan remains after shutdown" || true
fi
printf 'PASS\n'

printf 'Test 8: restart and recovery after normal operation... '
replay_bin=$(extract_package "$test_root/replay")
[[ -n "$replay_bin" ]] || fail "replay extraction did not contain executable"
export XDG_STATE_HOME="$test_root/state"
mkdir -p "$XDG_STATE_HOME" "$test_root/replay-work"
(
    cd "$test_root/replay-work"
    "$replay_bin" init >/dev/null
    first=$("$replay_bin" describe --json)
    second=$("$replay_bin" describe --json)
    [[ "$first" == "$second" ]]
    "$replay_bin" --help >/dev/null
    "$replay_bin" doctor >/dev/null
) || fail "restart/recovery sequence failed"
printf 'PASS\n'

printf '\n=== ALL 8 FULL TESTS PASSED ===\n'
