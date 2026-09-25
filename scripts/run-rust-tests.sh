#!/usr/bin/env bash
set -Eeuo pipefail

fail() {
    printf 'VERIFICATION PREREQUISITE FAILED\n%s\n' "$1" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: scripts/run-rust-tests.sh [--verify-only]

Build and verify the current OCaml engine, then run the Linux Rust host test
targets. With --verify-only, validate the existing provenance and engine
without rebuilding or running tests; this is for fail-closed diagnostics.
EOF
}

verify_only=false
while (($# > 0)); do
    case "$1" in
        --verify-only)
            verify_only=true
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
manifest_path="$repository_root/verification/current-engine-provenance.json"
rust_manifest="$repository_root/tethers-0.1/host-rust/Cargo.toml"

command -v jq >/dev/null 2>&1 || fail 'Required command is unavailable: jq'
if command -v sha256sum >/dev/null 2>&1; then
    compute_sha256() { sha256sum "$1" | awk '{print $1}'; }
elif command -v shasum >/dev/null 2>&1; then
    compute_sha256() { shasum -a 256 "$1" | awk '{print $1}'; }
else
    fail 'Required command is unavailable: sha256sum or shasum'
fi
command -v cargo >/dev/null 2>&1 || fail 'Required command is unavailable: cargo'
command -v git >/dev/null 2>&1 || fail 'Required command is unavailable: git'
command -v realpath >/dev/null 2>&1 || fail 'Required command is unavailable: realpath'

if [[ "$verify_only" == false ]]; then
    "$repository_root/scripts/prepare-current-engine.sh"
fi
[[ -f "$manifest_path" ]] ||
    fail 'Current engine provenance manifest was not produced; Rust tests were not attempted.'

schema=$(jq -er '.schema' "$manifest_path") ||
    fail 'Current engine provenance manifest is malformed; Rust tests were not attempted.'
status=$(jq -er '.status' "$manifest_path") ||
    fail 'Current engine provenance manifest is malformed; Rust tests were not attempted.'
[[ "$schema" == 'tethers.engine/1' && "$status" == 'pass' ]] ||
    fail 'Current engine provenance schema/status is invalid; Rust tests were not attempted.'

source_commit=$(jq -er '.source_commit' "$manifest_path") ||
    fail 'Current engine provenance is missing source_commit; Rust tests were not attempted.'
source_tree=$(jq -er '.source_tree' "$manifest_path") ||
    fail 'Current engine provenance is missing source_tree; Rust tests were not attempted.'
binary_relative_path=$(jq -er '.binary_relative_path' "$manifest_path") ||
    fail 'Current engine provenance is missing binary_relative_path; Rust tests were not attempted.'
expected_hash=$(jq -er '.binary_sha256' "$manifest_path") ||
    fail 'Current engine provenance is missing binary_sha256; Rust tests were not attempted.'

current_commit=$(git -C "$repository_root" rev-parse HEAD) ||
    fail 'Could not identify current Git HEAD; Rust tests were not attempted.'
current_tree=$(git -C "$repository_root" rev-parse 'HEAD^{tree}') ||
    fail 'Could not identify current Git tree; Rust tests were not attempted.'
[[ "$source_commit" == "$current_commit" && "$source_tree" == "$current_tree" ]] ||
    fail 'Current engine provenance does not match this source checkout; Rust tests were not attempted.'

case "$binary_relative_path" in
    ''|/*|../*|*/../*|*/..|*'/./'*|./*)
        fail 'Current engine provenance contains an unsafe binary path; Rust tests were not attempted.'
        ;;
esac
engine_path="$repository_root/$binary_relative_path"
[[ -f "$engine_path" && -x "$engine_path" ]] ||
    fail 'Current engine binary is missing or not executable; Rust tests were not attempted.'
resolved_engine_path=$(realpath -e -- "$engine_path") ||
    fail 'Current engine path could not be resolved; Rust tests were not attempted.'
case "$resolved_engine_path" in
    "$repository_root"/*)
        ;;
    *)
        fail 'Current engine resolves outside the current repository; Rust tests were not attempted.'
        ;;
esac
actual_hash=$(compute_sha256 "$engine_path")
[[ "$actual_hash" == "$expected_hash" ]] ||
    fail 'Current engine binary hash does not match provenance; Rust tests were not attempted.'

export TETHERS_VERIFIED_ENGINE="$engine_path"
export TETHERS_ENGINE_PROVENANCE="$manifest_path"
printf 'PASS verified engine: %s\n' "$TETHERS_VERIFIED_ENGINE"
printf 'PASS engine provenance: %s\n' "$TETHERS_ENGINE_PROVENANCE"
if [[ "$verify_only" == true ]]; then
    printf 'PASS provenance-only verification; Rust tests were not run\n'
    exit 0
fi
test_threads=${TETHERS_TEST_THREADS:-1}
if [[ "$test_threads" == default ]]; then
    printf 'RUN Rust host tests with the default Rust test threading\n'
else
    printf 'RUN Rust host tests with --test-threads=%s\n' "$test_threads"
fi
overall_status=0

run_test_target() {
    local target_kind="$1"
    local target_name="$2"
    local cargo_args=(
        cargo test
        --manifest-path "$rust_manifest"
    )
    if [[ "$target_kind" == lib ]]; then
        cargo_args+=(--lib)
    else
        cargo_args+=(--test "$target_name")
    fi
    cargo_args+=(--locked)
    if [[ "$test_threads" != default ]]; then
        cargo_args+=(-- "--test-threads=$test_threads")
    fi
    "${cargo_args[@]}"
}

# Cargo's `--all-targets --all-features` combination also tries to compile
# Windows-only benchmark binaries on Linux. Run the same product test targets
# explicitly: the host library, then every integration test file. The only
# optional features are benchmark features, so omit them on Linux rather than
# compiling Windows-only benchmark binaries.
if ! run_test_target lib ''; then
    overall_status=1
fi

shopt -s nullglob
integration_tests=("$repository_root/tethers-0.1/host-rust/tests/"*.rs)
[[ ${#integration_tests[@]} -gt 0 ]] || fail 'No Rust integration test targets were found.'

for test_file in "${integration_tests[@]}"; do
    test_name=$(basename -- "$test_file" .rs)
    if ! run_test_target integration "$test_name"; then
        overall_status=1
    fi
done

exit "$overall_status"
