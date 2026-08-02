set shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]

# Required commands: pwsh, just, Cargo Rust 1.89, and the tools checked by `just tools`.
tools:
    pwsh -NoProfile -File scripts/check-dev-tools.ps1

fmt:
    Push-Location tethers-0.1/host-rust; cargo +1.89.0 fmt --all -- --check

check:
    Push-Location tethers-0.1/host-rust; cargo +1.89.0 check --all-targets --all-features --locked

test-rust:
    Push-Location tethers-0.1/host-rust; cargo +1.89.0 test --all-targets --all-features --locked

test-m2:
    Push-Location tethers-0.1/host-rust; cargo +1.89.0 test package::tests --locked; cargo +1.89.0 test candidate::tests --locked

test-m3:
    Push-Location tethers-0.1/host-rust; cargo +1.89.0 test trust::tests --all-features --locked; cargo +1.89.0 test --test m3_lifecycle --all-features --locked

test-m4:
    Push-Location tethers-0.1/host-rust; cargo +1.89.0 test file_tools --all-features --locked; cargo +1.89.0 test --test m4_file_tools --all-features --locked

verify:
    pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1; just fmt; just check; just test-rust
