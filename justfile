set shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]

_manifest := "tethers-0.1/host-rust/Cargo.toml"

# Required commands: pwsh, just, Cargo (resolved from root pin), and the tools checked by `just tools`.
tools:
    pwsh -NoProfile -File scripts/check-dev-tools.ps1

fmt:
    scripts/invoke-timed.ps1 -Label "cargo-fmt" -Executable "cargo" -- fmt --manifest-path {{_manifest}} --all -- --check

check:
    $env:RUSTFLAGS="-D warnings"; scripts/invoke-timed.ps1 -Label "cargo-check" -Executable "cargo" -- check --manifest-path {{_manifest}} --all-targets --all-features --locked

test-rust:
    scripts/invoke-timed.ps1 -Label "cargo-test" -Executable "cargo" -- test --manifest-path {{_manifest}} --all-targets --all-features --locked

test-m2:
    cargo test --manifest-path {{_manifest}} package::tests --locked
    cargo test --manifest-path {{_manifest}} candidate::tests --locked

test-m3:
    cargo test --manifest-path {{_manifest}} trust::tests --all-features --locked
    cargo test --manifest-path {{_manifest}} --test m3_lifecycle --all-features --locked

test-m4:
    cargo test --manifest-path {{_manifest}} file_tools --all-features --locked
    cargo test --manifest-path {{_manifest}} --test m4_file_tools --all-features --locked

test-m5:
    cargo test --manifest-path {{_manifest}} local_anchor --all-features --locked
    cargo test --manifest-path {{_manifest}} --test m5_local_anchor --all-features --locked

verify:
    scripts/invoke-timed.ps1 -Label "task-packet" -Executable "pwsh" -- -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
    scripts/invoke-timed.ps1 -Label "cargo-fmt" -Executable "cargo" -- fmt --manifest-path {{_manifest}} --all -- --check
    @just check
    scripts/invoke-timed.ps1 -Label "cargo-test" -Executable "cargo" -- test --manifest-path {{_manifest}} --all-targets --all-features --locked

agent-tools:
    scripts/invoke-timed.ps1 -Label "agent-tools" -Executable "pwsh" -- -NoProfile -File scripts/check-rust-agent-tools.ps1

test-agent:
    scripts/invoke-timed.ps1 -Label "nextest" -Executable "cargo" -- nextest run --config-file .config/nextest.toml --manifest-path {{_manifest}} --all-targets --all-features --locked

deps-policy:
    scripts/invoke-timed.ps1 -Label "deps-policy" -Executable "cargo" -- deny --locked --manifest-path {{_manifest}} check licenses bans sources

deps-advisories:
    scripts/invoke-timed.ps1 -Label "deps-advisories" -Executable "cargo" -- deny --locked --manifest-path {{_manifest}} check advisories

deps-unused:
    scripts/invoke-timed.ps1 -Label "deps-unused" -Executable "cargo" -- machete --with-metadata tethers-0.1/host-rust

verify-agent: verify agent-tools deps-policy deps-advisories test-agent
