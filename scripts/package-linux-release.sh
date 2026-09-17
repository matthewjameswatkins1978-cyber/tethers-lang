#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/package-linux-release.sh [--release]

Build native Linux release binaries and package them into a distributable archive.

The OCaml switch is selected from TETHERS_OCAML_SWITCH, or from the prepared
workshop switch bl-tethers-5.5.0 when no override is supplied.
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
host_root="$repository_root/tethers-0.1/host-rust"
version=$(cat "$repository_root/VERSION" | tr -d '[:space:]')

require_command git
require_command cargo
require_command opam
require_command sha256sum
require_command tar
require_command gzip

# Verify clean worktree for release mode
if [[ "$release_mode" == true ]]; then
    if [[ -n "$(git -C "$repository_root" status --porcelain=v1 --untracked-files=all)" ]]; then
        fail 'Release packaging requires a clean Git worktree.'
    fi
fi

# Get source commit and tree
source_commit=$(git -C "$repository_root" rev-parse HEAD 2>/dev/null) ||
    fail 'Could not identify the current source commit.'
source_tree=$(git -C "$repository_root" rev-parse 'HEAD^{tree}' 2>/dev/null) ||
    fail 'Could not identify the current source tree.'

# Select OCaml switch
ocaml_switch=${TETHERS_OCAML_SWITCH:-bl-tethers-5.5.0}
ocaml_version=$(opam exec --switch="$ocaml_switch" -- ocamlc -version 2>/dev/null) ||
    fail "Could not run OCaml from switch: $ocaml_switch"
dune_version=$(opam exec --switch="$ocaml_switch" -- dune --version 2>/dev/null) ||
    fail "Could not run Dune from switch: $ocaml_switch"

# Build Rust Host
printf 'Building Rust Host...\n'
cargo build --release --locked --manifest-path "$host_root/Cargo.toml" \
    --bin tethers --bin agent_workspace_provider --bin agent_coding_provider ||
    fail "Rust Host build failed"

# Build OCaml Engine
printf 'Building OCaml Engine...\n'
(cd "$engine_root" && opam exec --switch="$ocaml_switch" -- dune build '@all') ||
    fail "OCaml Engine build failed"

# Locate built binaries
tethers_bin="$host_root/target/release/tethers"
workspace_provider_bin="$host_root/target/release/agent_workspace_provider"
coding_provider_bin="$host_root/target/release/agent_coding_provider"

# Engine binary may have .exe extension from dune
engine_bin=""
for candidate in \
    "$engine_root/_build/default/bin/tethers_mcp_main" \
    "$engine_root/_build/default/bin/tethers_mcp_main.exe"; do
    if [[ -f "$candidate" && -x "$candidate" ]]; then
        engine_bin="$candidate"
        break
    fi
done
[[ -n "$engine_bin" ]] ||
    fail 'The OCaml build did not produce the tethers_mcp_main executable.'

# Verify all binaries exist
for bin in "$tethers_bin" "$workspace_provider_bin" "$coding_provider_bin" "$engine_bin"; do
    [[ -f "$bin" ]] || fail "Missing binary: $bin"
done

# Create staging directory
dist_dir="$repository_root/dist"
stage_dir="$dist_dir/.tethers-$version-linux-x64-stage"
package_dir="$stage_dir/Tethers-$version-linux-x64"
archive="$dist_dir/Tethers-$version-linux-x64.tar.gz"
manifest_path="$dist_dir/Tethers-$version-linux-x64-manifest.json"
sums_path="$dist_dir/SHA256SUMS"

# Clean previous artifacts
for path in "$archive" "$manifest_path" "$sums_path" "$stage_dir"; do
    if [[ -e "$path" ]]; then
        rm -rf "$path"
    fi
done

mkdir -p "$dist_dir"
mkdir -p "$package_dir"

# Copy binaries
cp "$tethers_bin" "$package_dir/tethers"
cp "$engine_bin" "$package_dir/tethers-engine"
cp "$workspace_provider_bin" "$package_dir/agent_workspace_provider"
cp "$coding_provider_bin" "$package_dir/agent_coding_provider"

# Set executable permissions
chmod 755 "$package_dir/tethers"
chmod 755 "$package_dir/tethers-engine"
chmod 755 "$package_dir/agent_workspace_provider"
chmod 755 "$package_dir/agent_coding_provider"

# Copy documentation and metadata
cp "$repository_root/VERSION" "$package_dir/VERSION"
cp "$repository_root/README.md" "$package_dir/README.md"
cp "$repository_root/QUICKSTART.md" "$package_dir/QUICKSTART.md"
cp "$repository_root/docs/AI_INTEGRATION.md" "$package_dir/AI-INTEGRATION.md"
cp "$repository_root/docs/SECURITY.md" "$package_dir/SECURITY.md"
cp "$repository_root/docs/TETHERS_0_7_1_RELEASE.md" "$package_dir/TETHERS_0_7_1_RELEASE.md"

# Copy examples if they exist
if [[ -d "$repository_root/examples" ]]; then
    cp -r "$repository_root/examples" "$package_dir/examples"
fi

# Copy policies if they exist
if [[ -d "$repository_root/policies" ]]; then
    cp -r "$repository_root/policies" "$package_dir/policies"
fi

# Generate internal SHA256SUMS for package contents
(cd "$package_dir" && find . -type f -not -name 'SHA256SUMS' | sort | while read -r file; do
    sha256sum "$file" | sed 's|^\./||'
done > SHA256SUMS)

# Create deterministic tar.gz archive
# Use sorted entries, stable timestamps, and deterministic gzip
(cd "$stage_dir" && find "Tethers-$version-linux-x64" -type f | sort | \
    tar --no-recursion --owner=root --group=root --numeric-owner \
    --mtime="2026-01-01T00:00:00Z" \
    -czf "$archive" -T -)

# Record archive hash
archive_hash=$(sha256sum "$archive" | awk '{print $1}')

# Generate release manifest
cat > "$manifest_path" <<EOF
{
  "schema": "tethers.release/1",
  "product_version": "$version",
  "source_commit": "$source_commit",
  "source_tree": "$source_tree",
  "platform": "linux-x64",
  "archive": {
    "name": "Tethers-$version-linux-x64.tar.gz",
    "sha256": "$archive_hash"
  },
  "engine": {
    "name": "tethers-engine",
    "sha256": "$(sha256sum "$package_dir/tethers-engine" | awk '{print $1}')",
    "ocaml_version": "$ocaml_version",
    "dune_version": "$dune_version"
  },
  "executables": [
    {
      "name": "tethers",
      "sha256": "$(sha256sum "$package_dir/tethers" | awk '{print $1}')"
    },
    {
      "name": "tethers-engine",
      "sha256": "$(sha256sum "$package_dir/tethers-engine" | awk '{print $1}')"
    },
    {
      "name": "agent_workspace_provider",
      "sha256": "$(sha256sum "$package_dir/agent_workspace_provider" | awk '{print $1}')"
    },
    {
      "name": "agent_coding_provider",
      "sha256": "$(sha256sum "$package_dir/agent_coding_provider" | awk '{print $1}')"
    }
  ]
}
EOF

# Record manifest hash
manifest_hash=$(sha256sum "$manifest_path" | awk '{print $1}')

# Generate top-level SHA256SUMS
cat > "$sums_path" <<EOF
$archive_hash  Tethers-$version-linux-x64.tar.gz
$manifest_hash  Tethers-$version-linux-x64-manifest.json
EOF

# Clean staging directory
rm -rf "$stage_dir"

printf '\nPACKAGE COMPLETE\n'
printf 'Archive: %s\n' "$archive"
printf 'Manifest: %s\n' "$manifest_path"
printf 'SHA256SUMS: %s\n' "$sums_path"
printf 'Archive SHA256: %s\n' "$archive_hash"
printf 'Manifest SHA256: %s\n' "$manifest_hash"
