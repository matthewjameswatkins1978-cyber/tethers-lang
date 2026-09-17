#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/test-linux-package-full.sh <archive-path>

Run path, permission, supervision, and replay/recovery tests on a Linux package.
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

archive_path=$(realpath "$1")
[[ -f "$archive_path" ]] || fail "Archive does not exist: $archive_path"
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
unset TETHERS_OCAML_SWITCH 2>/dev/null || true

test_root=$(mktemp -d "${TMPDIR:-/tmp}/tethers-linux-test.XXXXXX")
trap 'rm -rf "$test_root"' EXIT

extract_package() {
    local destination=$1
    mkdir -p "$destination"
    tar xzf "$archive_path" -C "$destination"
    find "$destination" -type f -name tethers -executable -print -quit
}

printf '=== TETHERS LINUX PACKAGE FULL TEST SUITE ===\nArchive: %s\n\n' "$archive_path"

printf 'Test 1: normal path... '
normal_dir="$test_root/normal"
normal_bin=$(extract_package "$normal_dir")
[[ -n "$normal_bin" ]] || fail "normal extraction did not contain executable"
"$normal_bin" --help >/dev/null || fail "normal path executable failed"
printf 'PASS\n'

printf 'Test 2: path with spaces... '
spaces_dir="$test_root/path with spaces"
spaces_bin=$(extract_package "$spaces_dir")
[[ -n "$spaces_bin" ]] || fail "space-path extraction did not contain executable"
"$spaces_bin" --help >/dev/null || fail "space-path executable failed"
printf 'PASS\n'

printf 'Test 3: nested and Unicode paths... '
nested_dir="$test_root/a/b/c/テスト"
nested_bin=$(extract_package "$nested_dir")
[[ -n "$nested_bin" ]] || fail "nested-path extraction did not contain executable"
"$nested_bin" --help >/dev/null || fail "nested-path executable failed"
printf 'PASS\n'

printf 'Test 4: executable permissions... '
perm_dir="$test_root/permissions"
extract_package "$perm_dir" >/dev/null
for bin_name in tethers tethers-engine agent_workspace_provider agent_coding_provider; do
    bin_path=$(find "$perm_dir" -type f -name "$bin_name" -print -quit)
    [[ -n "$bin_path" && -x "$bin_path" ]] || fail "$bin_name is missing or not executable"
done
printf 'PASS\n'

printf 'Test 5: package path independence... '
independent_dir="$test_root/independent"
independent_bin=$(extract_package "$independent_dir")
[[ -n "$independent_bin" ]] || fail "independent extraction did not contain executable"
if strings "$independent_bin" | grep -Fq 'C:\\'; then fail "binary contains a Windows path"; fi
if strings "$independent_bin" | grep -Fq '/home/matmus/'; then fail "binary contains developer home path"; fi
printf 'PASS\n'

printf 'Test 6: provider supervision and no orphan... '
supervisor_dir="$test_root/supervisor"
extract_package "$supervisor_dir" >/dev/null
provider_bin=$(find "$supervisor_dir" -type f -name agent_workspace_provider -executable -print -quit)
[[ -n "$provider_bin" ]] || fail "provider binary missing"
provider_fifo="$test_root/supervisor.fifo"
mkfifo "$provider_fifo"
exec {provider_fd}<>"$provider_fifo"
scope_json=$(printf '{"query_root":"%s","move_source_root":"%s","move_destination_root":"%s","max_content_bytes":65536}' "$supervisor_dir" "$supervisor_dir" "$supervisor_dir")
provider_log="$test_root/supervisor.log"
TETHERS_OPERATIONAL_SCOPE_JSON="$scope_json" "$provider_bin" <&$provider_fd >"$provider_log" 2>&1 &
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
if pgrep -f -- "$provider_bin" >/dev/null 2>&1; then fail "provider orphan remains"; fi
printf 'PASS\n'

printf 'Test 7: replay/recovery after interruption... '
replay_dir="$test_root/replay"
replay_bin=$(extract_package "$replay_dir")
[[ -n "$replay_bin" ]] || fail "replay extraction did not contain executable"
export XDG_STATE_HOME="$test_root/state"
mkdir -p "$XDG_STATE_HOME"
(
    cd "$(dirname "$replay_bin")"
    "$replay_bin" init >/dev/null
    first=$("$replay_bin" describe --json)
    second=$("$replay_bin" describe --json)
    [[ "$first" == "$second" ]]
    "$replay_bin" --help >/dev/null
) || fail "replay/recovery sequence failed"
printf 'PASS\n'

printf '\n=== ALL 7 FULL TESTS PASSED ===\n'
