param(
    [Parameter(Mandatory = $true)]
    [string]$OcamlSwitchPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Script:ExitCode = 0

function Fail {
    param([string]$Message)
    Write-Host "FAIL: $Message"
    $Script:ExitCode = 1
}

function Pass {
    param([string]$Message)
    Write-Host "PASS: $Message"
}

# --- OcamlSwitchPath validation ---
if (-not [System.IO.Path]::IsPathFullyQualified($OcamlSwitchPath)) {
    Fail "OcamlSwitchPath must be absolute: $OcamlSwitchPath"
    exit $Script:ExitCode
}

$CanonicalSwitch = [System.IO.Path]::GetFullPath($OcamlSwitchPath)
if (-not (Test-Path -LiteralPath $CanonicalSwitch -PathType Container)) {
    Fail "OcamlSwitchPath does not exist: $CanonicalSwitch"
    exit $Script:ExitCode
}

$OpamDir = Join-Path $CanonicalSwitch "_opam"
if (-not (Test-Path -LiteralPath $OpamDir -PathType Container)) {
    Fail "_opam not found under OcamlSwitchPath: $OpamDir"
    exit $Script:ExitCode
}

$SwitchMarker = Join-Path $OpamDir ".opam-switch"
if (-not ((Test-Path -LiteralPath $SwitchMarker -PathType Container) -or
          (Test-Path -LiteralPath $SwitchMarker -PathType Leaf))) {
    Fail ".opam-switch not found in _opam: $SwitchMarker"
    exit $Script:ExitCode
}

# --- Rust process guard ---
$PrevRustupAutoInstall = $env:RUSTUP_AUTO_INSTALL
try {
    $env:RUSTUP_AUTO_INSTALL = "0"

    # --- Rust toolchain verification ---
    $toolchains = @(& rustup toolchain list 2>&1 | ForEach-Object { "$_" })
    if ($LASTEXITCODE -ne 0) {
        Fail "rustup toolchain list failed"
    } elseif (($toolchains -join "`n") -notmatch "1\.89\.0-x86_64-pc-windows-msvc") {
        Fail "Rust toolchain 1.89.0-x86_64-pc-windows-msvc not found"
    } else {
        Pass "Rust toolchain 1.89.0-x86_64-pc-windows-msvc installed"
    }

    $components = @(& rustup component list --toolchain 1.89.0 --installed 2>&1 | ForEach-Object { "$_" })
    if ($LASTEXITCODE -ne 0) {
        Fail "rustup component list failed"
    } else {
        if (($components -join "`n") -notmatch "rustfmt") {
            Fail "rustfmt not installed for 1.89.0"
        } else {
            Pass "rustfmt installed for 1.89.0"
        }
        if (($components -join "`n") -notmatch "clippy") {
            Fail "clippy not installed for 1.89.0"
        } else {
            Pass "clippy installed for 1.89.0"
        }
    }

    # Version checks (only if components present)
    if ($Script:ExitCode -eq 0) {
        $rustcVer = & rustup run 1.89.0 rustc --version 2>&1
        if ($rustcVer -match "1\.89\.0") { Pass "rustc: $rustcVer" } else { Fail "rustc version: $rustcVer" }

        $cargoVer = & rustup run 1.89.0 cargo --version 2>&1
        if ($cargoVer -match "1\.89\.0") { Pass "cargo: $cargoVer" } else { Fail "cargo version: $cargoVer" }

        $rustfmtVer = & rustup run 1.89.0 rustfmt --version 2>&1
        if ($rustfmtVer -match "1\.8") { Pass "rustfmt: $rustfmtVer" } else { Fail "rustfmt version: $rustfmtVer" }

        $clippyVer = & rustup run 1.89.0 cargo clippy --version 2>&1
        if ($clippyVer -match "0\.1\.89") { Pass "clippy: $clippyVer" } else { Fail "clippy version: $clippyVer" }
    }

} finally {
    if ($null -eq $PrevRustupAutoInstall) {
        Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
    } else {
        $env:RUSTUP_AUTO_INSTALL = $PrevRustupAutoInstall
    }
}

if ($Script:ExitCode -ne 0) {
    exit $Script:ExitCode
}

# --- OCaml verification ---
$opamVer = & opam --version 2>&1
if ($LASTEXITCODE -ne 0) { Fail "opam not found"; exit $Script:ExitCode }
$opamMajor = [int]($opamVer -split '\.')[0]
$opamMinor = [int]($opamVer -split '\.')[1]
if ($opamMajor -lt 2 -or ($opamMajor -eq 2 -and $opamMinor -lt 2)) {
    Fail "opam is $opamVer; requires 2.2 or newer"
} else {
    Pass "opam $opamVer"
}

$switchRootRaw = & opam switch show --switch="$CanonicalSwitch" 2>&1
if ($LASTEXITCODE -ne 0) { Fail "opam switch show failed: $switchRootRaw"; exit $Script:ExitCode }
$switchRootCanonical = [System.IO.Path]::GetFullPath($switchRootRaw.Trim())
if (-not $switchRootCanonical.Equals($CanonicalSwitch, [System.StringComparison]::OrdinalIgnoreCase)) {
    Fail "Switch root mismatch: expected $CanonicalSwitch, got $switchRootCanonical"
    exit $Script:ExitCode
}
Pass "Switch root matches"

$prefixRaw = & opam var prefix --switch="$CanonicalSwitch" 2>&1
if ($LASTEXITCODE -ne 0) { Fail "opam var prefix failed: $prefixRaw"; exit $Script:ExitCode }
$prefixCanonical = [System.IO.Path]::GetFullPath($prefixRaw.Trim())
$expectedPrefix = [System.IO.Path]::GetFullPath($OpamDir)
if (-not $prefixCanonical.Equals($expectedPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    Fail "Prefix mismatch: expected $expectedPrefix, got $prefixCanonical"
    exit $Script:ExitCode
}
Pass "Prefix matches _opam"

$ocamlcVer = & opam exec --switch="$CanonicalSwitch" -- ocamlc -version 2>&1
if ($ocamlcVer -eq "5.5.0") { Pass "OCaml $ocamlcVer" } else { Fail "OCaml: expected 5.5.0, got $ocamlcVer" }

$ocamloptVer = & opam exec --switch="$CanonicalSwitch" -- ocamlopt -version 2>&1
if ($ocamloptVer -eq "5.5.0") { Pass "ocamlopt $ocamloptVer" } else { Fail "ocamlopt: expected 5.5.0, got $ocamloptVer" }

$duneVer = & opam exec --switch="$CanonicalSwitch" -- dune --version 2>&1
if ($duneVer -eq "3.24.0") { Pass "Dune $duneVer" } else { Fail "Dune: expected 3.24.0, got $duneVer" }

$pkgList = & opam list --switch="$CanonicalSwitch" --installed --columns=name,version 2>&1
if ($LASTEXITCODE -ne 0) { Fail "opam list failed"; exit $Script:ExitCode }
if ($pkgList -match "yojson\s+2\.2\.2") { Pass "Yojson 2.2.2" } else { Fail "Yojson 2.2.2 not found in installed packages" }

# --- Repository baseline checks ---
$repoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent

$rtFile = Join-Path $repoRoot "rust-toolchain.toml"
if (-not (Test-Path $rtFile)) { Fail "rust-toolchain.toml missing"; exit $Script:ExitCode }
$rt = Get-Content $rtFile -Raw
if ($rt -match 'channel\s*=\s*"1\.89\.0"') { Pass "rust-toolchain.toml: channel 1.89.0" } else { Fail "rust-toolchain.toml: wrong channel" }
if ($rt -match 'components\s*=.*"rustfmt"' -and $rt -match 'components\s*=.*"clippy"') { Pass "rust-toolchain.toml: rustfmt + clippy" } else { Fail "rust-toolchain.toml: missing components" }

$cargoToml = Join-Path $repoRoot "tethers-0.1/host-rust/Cargo.toml"
$cargo = Get-Content $cargoToml -Raw
if ($cargo -match 'edition\s*=\s*"2021"') { Pass "Cargo.toml: edition 2021" } else { Fail "Cargo.toml: wrong edition" }
if ($cargo -match 'rust-version\s*=\s*"1\.89"') { Pass "Cargo.toml: rust-version 1.89" } else { Fail "Cargo.toml: missing rust-version" }

$cargoLock = Join-Path $repoRoot "tethers-0.1/host-rust/Cargo.lock"
if (Test-Path $cargoLock) { Pass "Cargo.lock present" } else { Fail "Cargo.lock missing" }

$opamFile = Join-Path $repoRoot "tethers-0.1/engine-ocaml/tethers_engine.opam"
$opamContent = Get-Content $opamFile -Raw
if ($opamContent -match '"ocaml"\s*\{>=\s*"5\.5\.0"\s*&\s*<\s*"5\.6\.0"\s*\}') { Pass "OCaml range: >= 5.5.0 & < 5.6.0" } else { Fail "OCaml range does not match tightened constraint" }

$lockedFile = Join-Path $repoRoot "tethers-0.1/engine-ocaml/tethers_engine.opam.locked"
if (-not (Test-Path $lockedFile)) { Fail "opam.locked missing"; exit $Script:ExitCode }
$locked = Get-Content $lockedFile -Raw
if ($locked -match '"ocaml"\s*\{=\s*"5\.5\.0"\}') { Pass "opam.locked: OCaml 5.5.0" } else { Fail "opam.locked: wrong OCaml" }
if ($locked -match '"dune"\s*\{=\s*"3\.24\.0"\}') { Pass "opam.locked: Dune 3.24.0" } else { Fail "opam.locked: wrong Dune" }
if ($locked -match '"yojson"\s*\{=\s*"2\.2\.2"\}') { Pass "opam.locked: Yojson 2.2.2" } else { Fail "opam.locked: wrong Yojson" }

$duneProject = Join-Path $repoRoot "tethers-0.1/engine-ocaml/dune-project"
$dp = Get-Content $duneProject -Raw
if ($dp -match '\(lang dune 3\.10\)') { Pass "dune-project: lang dune 3.10" } else { Fail "dune-project: wrong language" }

# --- Final result ---
Write-Host ""
if ($Script:ExitCode -eq 0) {
    Write-Host "All toolchain checks passed."
} else {
    Write-Host "One or more toolchain checks failed."
}
exit $Script:ExitCode
