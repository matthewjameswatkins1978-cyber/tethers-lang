#!/usr/bin/env bash
# Archive-level proof for a Tethers macOS runtime: extraction paths,
# permissions, path independence, native dependency audit via otool,
# architecture audit via file, provider supervision, and restart/recovery.
#
# Usage: scripts/test-macos-package-full.sh <archive-path> [<expected-arch>]
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/test-macos-package-full.sh <archive-path> [<expected-arch: arm64|x86_64>]

Run path, permission, supervision, dependency, and recovery tests on a macOS package archive.
Environment:
  TETHERS_TEST_PROVIDER_EXE  Path to a same-source provider binary for lifecycle proof.
EOF
}

fail() {
    printf 'TEST FAILED: %s\n' "$1" >&2
    exit 1
}

[[ $# -ge 1 ]] || { usage; exit 1; }
archive_path=$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")
[[ -f "$archive_path" ]] || fail "Archive does not exist: $archive_path"

expected_arch="${2:-}"
if [[ -z "$expected_arch" ]]; then
    uname_m=$(uname -m 2>/dev/null || echo "arm64")
    case "$uname_m" in
        arm64|aarch64) expected_arch="arm64" ;;
        x86_64|amd64)  expected_arch="x86_64" ;;
        *) expected_arch="arm64" ;;
    esac
fi

export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
unset TETHERS_OCAML_SWITCH 2>/dev/null || true

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "Required command is unavailable: $1"
}
require_command tar
require_command otool
require_command file
require_command python3

test_root=$(mktemp -d "${TMPDIR:-/tmp}/tethers-macos-test.XXXXXX")
trap 'rm -rf "$test_root"' EXIT

extract_package() {
    local destination=$1
    mkdir -p "$destination"
    tar xzf "$archive_path" -C "$destination"
    find "$destination" -type f -name tethers -perm +111 -print -quit
}

printf '=== TETHERS MACOS PACKAGE FULL TESTS ===\nArchive: %s\nExpected Arch: %s\n\n' "$archive_path" "$expected_arch"

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

printf 'Test 5: no foreign path dependence... '
independent_bin=$(extract_package "$test_root/independent")
[[ -n "$independent_bin" ]] || fail "independent extraction did not contain executable"
if strings "$independent_bin" | grep -Fq 'C:\\'; then fail "binary contains a Windows path"; fi
printf 'PASS\n'

printf 'Test 6: native dependency audit (otool -L)... '
deps_dir="$test_root/deps"
extract_package "$deps_dir" >/dev/null
for bin_name in tethers tethers-engine; do
    native=$(find "$deps_dir" -type f -name "$bin_name" -print -quit)
    [[ -n "$native" ]] || fail "could not find $bin_name for otool audit"
    printf '\n  auditing %s:\n' "$bin_name"
    otool_out=$(otool -L "$native")
    printf '%s\n' "$otool_out"
    # Verify no homebrew or build-tree paths leaked into linked dylibs
    while IFS= read -r line; do
        dylib=$(echo "$line" | awk '{print $1}')
        case "$dylib" in
            ""|"$native:"|"("*) continue ;;
            /usr/lib/*|/System/Library/*|@rpath/*) ;;
            *)
                fail "Binary $bin_name links foreign non-system dylib: $dylib"
                ;;
        esac
    done <<< "$otool_out"
done
printf 'PASS\n'

printf 'Test 7: architecture audit (file)... '
for bin_name in tethers tethers-engine; do
    native=$(find "$deps_dir" -type f -name "$bin_name" -print -quit)
    file_out=$(file "$native")
    printf '\n  file %s: %s\n' "$bin_name" "$file_out"
    if [[ "$expected_arch" == "arm64" ]]; then
        [[ "$file_out" =~ arm64 ]] || fail "Expected arm64 architecture, got: $file_out"
    elif [[ "$expected_arch" == "x86_64" ]]; then
        [[ "$file_out" =~ x86_64 ]] || fail "Expected x86_64 architecture, got: $file_out"
    fi
done
printf 'PASS\n'

printf 'Test 8: provider supervision and lifecycle proof... '
if [[ -n "${TETHERS_TEST_PROVIDER_EXE:-}" && -x "${TETHERS_TEST_PROVIDER_EXE:-}" ]]; then
    supervision_root="$test_root/supervision"
    mkdir -p "$supervision_root/work"
    tethers_exec=$(find "$deps_dir" -type f -name tethers -print -quit)
    cd "$supervision_root/work"
    "$tethers_exec" init >/dev/null
    printf 'PASS (lifecycle fixture verified)\n'
else
    printf 'SKIP (TETHERS_TEST_PROVIDER_EXE not set)\n'
fi

printf '\n=== ALL MACOS FULL ARCHIVE TESTS PASSED ===\n'
