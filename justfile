set shell := ["/bin/sh", "-cu"]
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]

[private]
_manifest := "tethers-0.1/host-rust/Cargo.toml"
[private]
_python := if env_var_or_default("OS", "") == "Windows_NT" { "py -3" } else { "/usr/bin/python3" }

# Required commands: pwsh, just, Cargo (resolved from root pin), and the tools checked by `just tools`.
tools:
    pwsh -NoProfile -File scripts/check-dev-tools.ps1

fmt:
    cargo fmt --manifest-path {{ _manifest }} --all -- --check

check:
    cargo check --manifest-path {{ _manifest }} --all-targets --all-features --locked

warning-ratchet:
    {{ _python }} scripts/check-warning-ratchet.py

test-rust:
    pwsh -NoProfile -File scripts/run-rust-tests.ps1

test-m2:
    cargo test --manifest-path {{ _manifest }} package::tests --locked
    cargo test --manifest-path {{ _manifest }} candidate::tests --locked

test-m3:
    cargo test --manifest-path {{ _manifest }} trust::tests --all-features --locked
    cargo test --manifest-path {{ _manifest }} --test m3_lifecycle --all-features --locked

test-m4:
    cargo test --manifest-path {{ _manifest }} file_tools --all-features --locked
    cargo test --manifest-path {{ _manifest }} --test m4_file_tools --all-features --locked

test-m5:
    cargo test --manifest-path {{ _manifest }} local_anchor --all-features --locked
    cargo test --manifest-path {{ _manifest }} --test m5_local_anchor --all-features --locked

verify:
    {{ _python }} scripts/verify-tethers.py

test-verifier:
    {{ _python }} scripts/test-verify-tethers.py

test-provenance:
    {{ _python }} scripts/test-engine-provenance.py

agent-tools:
    scripts/invoke-timed.ps1 -Label "agent-tools" -Executable "pwsh" -- -NoProfile -File scripts/check-rust-agent-tools.ps1

test-agent:
    scripts/invoke-timed.ps1 -Label "nextest" -Executable "cargo" -- nextest run --config-file .config/nextest.toml --manifest-path {{ _manifest }} --all-targets --all-features --locked

deps-policy:
    scripts/invoke-timed.ps1 -Label "deps-policy" -Executable "cargo" -- deny --locked --manifest-path {{ _manifest }} check licenses bans sources

deps-advisories:
    scripts/invoke-timed.ps1 -Label "deps-advisories" -Executable "cargo" -- deny --locked --manifest-path {{ _manifest }} check advisories

deps-unused:
    scripts/invoke-timed.ps1 -Label "deps-unused" -Executable "cargo" -- machete --with-metadata tethers-0.1/host-rust

test-pdf-reference:
    cargo fmt --manifest-path reference-plugs/pdf-tools/provider-rust/Cargo.toml -- --check
    cargo test --manifest-path reference-plugs/pdf-tools/provider-rust/Cargo.toml --locked
    cargo build --manifest-path reference-plugs/pdf-tools/provider-rust/Cargo.toml --locked
    $env:TETHERS_PDF_REFERENCE_PROVIDER_EXE=(Resolve-Path reference-plugs/pdf-tools/provider-rust/target/debug/pdf_tools_provider.exe); cargo test --manifest-path {{ _manifest }} --locked --test p3_pdf_reference_plug -- --ignored

test-evil-bunny-proof:
    cargo fmt --manifest-path reference-plugs/evil-bunny-proof/provider-rust/Cargo.toml -- --check
    cargo test --manifest-path reference-plugs/evil-bunny-proof/provider-rust/Cargo.toml --locked
    cargo build --manifest-path reference-plugs/evil-bunny-proof/provider-rust/Cargo.toml --locked
    $env:TETHERS_EVIL_BUNNY_PROVIDER_EXE=(Resolve-Path reference-plugs/evil-bunny-proof/provider-rust/target/debug/tethers_evil_bunny_provider.exe); cargo test --manifest-path {{ _manifest }} --locked --test p6_evil_bunny -- --ignored

verify-agent: verify agent-tools deps-policy deps-advisories test-agent test-pdf-reference

package-0-7:
    pwsh -NoProfile -File scripts/package-0.7.ps1
