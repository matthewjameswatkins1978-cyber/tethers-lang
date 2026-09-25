#!/usr/bin/env bash
# Tethers macOS (Apple Silicon arm64 / Intel x64) full-runtime packager (0.8 architecture).
#
# Builds the native macOS release binaries (Rust Host + matching OCaml engine)
# and stages a distributable archive whose layout mirrors the Windows and Linux bundles:
#   bin/tethers, bin/tethers-engine, docs/, examples/external-consumer/,
#   LICENSES, README, QUICKSTART, VERSION, SHA256SUMS
# plus a tethers.release/1 provenance manifest and top-level checksums.
#
# Version comes from the VERSION file, never from a hard-coded string.
# With --release the script additionally requires a clean worktree and that
# HEAD is exactly the release tag v<VERSION>; without it, it is a dev build.
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/package-macos-release.sh [--release] [--arch <arm64|x64|macos-arm64|macos-x64>] [--output-root <dir>]

Options:
  --release            Require a clean worktree and HEAD == v<VERSION> tag.
  --arch <arch>        Target architecture: macos-arm64 (default on Apple Silicon) or macos-x64.
  --output-root <dir>  Repository-relative output directory (default: dist).
  --help, -h           Print this help.
EOF
}

fail() {
    printf 'PACKAGE FAILED\n%s\n' "$1" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "Required command is unavailable: $1"
}

release_mode=false
output_root="dist"
target_arch=""

while (($# > 0)); do
    case "$1" in
        --release) release_mode=true ;;
        --arch)
            shift
            [[ $# -gt 0 ]] || fail "--arch requires a value."
            case "$1" in
                arm64|macos-arm64) target_arch="macos-arm64" ;;
                x64|x86_64|macos-x64) target_arch="macos-x64" ;;
                *) fail "Unsupported architecture: $1 (expected macos-arm64 or macos-x64)" ;;
            esac
            ;;
        --output-root)
            shift
            [[ $# -gt 0 ]] || fail "--output-root requires a value."
            output_root="$1"
            ;;
        --help|-h) usage; exit 0 ;;
        *) fail "Unknown argument: $1" ;;
    esac
    shift
done

if [[ -z "$target_arch" ]]; then
    uname_m=$(uname -m 2>/dev/null || echo "unknown")
    case "$uname_m" in
        arm64|aarch64) target_arch="macos-arm64" ;;
        x86_64|amd64) target_arch="macos-x64" ;;
        *) fail "Could not detect macOS architecture (uname -m returned '$uname_m'); use --arch" ;;
    esac
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_dir/.." && pwd -P)
engine_root="$repository_root/tethers-0.1/engine-ocaml"
host_root="$repository_root/tethers-0.1/host-rust"
version=$(tr -d '[:space:]' < "$repository_root/VERSION")
[[ -n "$version" ]] || fail "VERSION file is empty."
expected_tag="v$version"

require_command git
require_command cargo
require_command opam
require_command tar
require_command gzip
require_command find
require_command sort

if command -v sha256sum >/dev/null 2>&1; then
    compute_sha256() { sha256sum "$1" | awk '{print $1}'; }
elif command -v shasum >/dev/null 2>&1; then
    compute_sha256() { shasum -a 256 "$1" | awk '{print $1}'; }
else
    fail "Required command is unavailable: sha256sum or shasum"
fi

source_commit=$(git -C "$repository_root" rev-parse HEAD 2>/dev/null) ||
    fail 'Could not identify the current source commit.'
source_tree=$(git -C "$repository_root" rev-parse 'HEAD^{tree}' 2>/dev/null) ||
    fail 'Could not identify the current source tree.'

if [[ "$release_mode" == true ]]; then
    if [[ -n "$(git -C "$repository_root" status --porcelain=v1 --untracked-files=all)" ]]; then
        fail 'Release packaging requires a clean Git worktree.'
    fi
    head_tag=$(git -C "$repository_root" describe --exact-match --tags HEAD 2>/dev/null || true)
    [[ "$head_tag" == "$expected_tag" ]] ||
        fail "Release packaging requires HEAD to be exactly $expected_tag; got '${head_tag:-<no tag>}'."
fi

ocaml_switch="${TETHERS_OCAML_SWITCH:-}"
if [[ -z "$ocaml_switch" && -d "$engine_root/_opam" ]]; then
    ocaml_switch="$engine_root"
fi
[[ -n "$ocaml_switch" ]] ||
    fail 'No OCaml switch was supplied. Set TETHERS_OCAML_SWITCH or create the repository-local tethers-0.1/engine-ocaml switch.'
ocaml_version=$(opam exec --switch="$ocaml_switch" -- ocamlc -version 2>/dev/null) ||
    fail "Could not run OCaml from switch: $ocaml_switch"
dune_version=$(opam exec --switch="$ocaml_switch" -- dune --version 2>/dev/null) ||
    fail "Could not run Dune from switch: $ocaml_switch"
rustc_version=$(rustc --version 2>/dev/null) ||
    fail 'Could not determine rustc version.'

printf 'Building Rust Host (%s)...\n' "$target_arch"
cargo build --release --locked --manifest-path "$host_root/Cargo.toml" --bin tethers ||
    fail 'Rust Host release build failed.'

printf 'Building OCaml engine (%s)...\n' "$target_arch"
(cd "$engine_root" && opam exec --switch="$ocaml_switch" -- dune build --profile release '@all') ||
    fail 'OCaml engine release build failed.'

tethers_bin="$host_root/target/release/tethers"
engine_bin=""
for candidate in \
    "$engine_root/_build/default/bin/tethers_mcp_main" \
    "$engine_root/_build/default/bin/tethers_mcp_main.exe"; do
    if [[ -f "$candidate" && -x "$candidate" ]]; then
        engine_bin="$candidate"
        break
    fi
done
[[ -f "$tethers_bin" ]] || fail "Missing Rust Host binary: $tethers_bin"
[[ -n "$engine_bin" ]] || fail 'The OCaml build did not produce the tethers_mcp_main executable.'

dist_dir="$repository_root/$output_root"
stage_dir="$dist_dir/.tethers-$version-$target_arch-stage"
package_name="Tethers-$version-$target_arch"
package_dir="$stage_dir/$package_name"
archive="$dist_dir/$package_name.tar.gz"
manifest_path="$dist_dir/$package_name-manifest.json"
sums_path="$dist_dir/SHA256SUMS-$version"

for path in "$archive" "$archive.sha256" "$manifest_path" "$stage_dir"; do
    if [[ -e "$path" ]]; then
        fail "Refusing to replace existing release output: $path"
    fi
done
mkdir -p "$dist_dir" "$package_dir/bin" "$package_dir/docs" "$package_dir/examples/external-consumer"

cp "$tethers_bin" "$package_dir/bin/tethers"
cp "$engine_bin" "$package_dir/bin/tethers-engine"
chmod 755 "$package_dir/bin/tethers" "$package_dir/bin/tethers-engine"

cp "$repository_root/VERSION" "$package_dir/VERSION"
cp "$repository_root/README.md" "$package_dir/README.md"
cp "$repository_root/QUICKSTART.md" "$package_dir/QUICKSTART.md"
cp "$repository_root/LICENSE-MIT" "$package_dir/LICENSE-MIT"
cp "$repository_root/LICENSE-APACHE" "$package_dir/LICENSE-APACHE"
cp "$repository_root/docs/INTEGRATING_TETHERS.md" "$package_dir/docs/INTEGRATING_TETHERS.md"
release_notes="$repository_root/docs/TETHERS_${version//./_}_RELEASE.md"
[[ -f "$release_notes" ]] || fail "Missing release notes: $release_notes"
cp "$release_notes" "$package_dir/docs/"
cp "$repository_root/examples/external-consumer/README.md" "$package_dir/examples/external-consumer/README.md"
cp "$repository_root/examples/external-consumer/consumer.py" "$package_dir/examples/external-consumer/consumer.py"
cp "$repository_root/examples/external-consumer/smoke.py" "$package_dir/examples/external-consumer/smoke.py"
cp "$repository_root/examples/external-consumer/fixture-ping.json" "$package_dir/examples/external-consumer/fixture-ping.json"

for req in README.md consumer.py smoke.py fixture-ping.json; do
    [[ -f "$package_dir/examples/external-consumer/$req" ]] ||
        fail "Packaging verification failure: missing required example file: examples/external-consumer/$req"
done

# Per-file hashes inside the stage.
(cd "$package_dir" && find . -type f | sort | while IFS= read -r file; do
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file"
    else
        shasum -a 256 "$file"
    fi
done > SHA256SUMS)

# Package archive using tar -czf.
(cd "$stage_dir" && tar -czf "$archive" "$package_name")

archive_hash=$(compute_sha256 "$archive")
tethers_hash=$(compute_sha256 "$package_dir/bin/tethers")
engine_hash=$(compute_sha256 "$package_dir/bin/tethers-engine")

cat > "$manifest_path" <<EOF
{
  "schema": "tethers.release/1",
  "product_version": "$version",
  "tag": "$expected_tag",
  "source_commit": "$source_commit",
  "source_tree": "$source_tree",
  "platform": "$target_arch",
  "rustc": "$rustc_version",
  "ocaml": "$ocaml_version",
  "dune": "$dune_version",
  "archive": {
    "name": "$package_name.tar.gz",
    "sha256": "$archive_hash"
  },
  "executables": [
    {
      "name": "bin/tethers",
      "sha256": "$tethers_hash"
    },
    {
      "name": "bin/tethers-engine",
      "sha256": "$engine_hash"
    }
  ]
}
EOF
manifest_hash=$(compute_sha256 "$manifest_path")

cat >> "$sums_path" <<EOF
$archive_hash  $package_name.tar.gz
$manifest_hash  $package_name-manifest.json
EOF
printf '%s  %s\n' "$archive_hash" "$package_name.tar.gz" > "$archive.sha256"

rm -rf "$stage_dir"

printf '\nPACKAGE COMPLETE\n'
printf 'Archive: %s\n' "$archive"
printf 'Manifest: %s\n' "$manifest_path"
printf 'Archive SHA256: %s\n' "$archive_hash"
printf 'Manifest SHA256: %s\n' "$manifest_hash"
