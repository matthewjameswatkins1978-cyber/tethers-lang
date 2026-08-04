param(
    [string]$OcamlSwitchPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Read-TomlString {
    param([string]$Content, [string]$Key)
    if ($Content -match "$Key\s*=\s*`"([^`"]+)`"") {
        return $Matches[1]
    }
    return $null
}

function Invoke-TethersToolchainCheck {
    param(
        [string]$SwitchPath
    )
    $OcamlSwitchPath = $SwitchPath

    # Reset all per-invocation state
    $Script:ExitCode = 0
    $Script:CheckOutput = [System.Collections.Generic.List[string]]::new()

    function Fail {
        param([string]$Message)
        $msg = "FAIL: $Message"
        Write-Host $msg
        [void]$Script:CheckOutput.Add($msg)
        $Script:ExitCode = 1
    }

    function Pass {
        param([string]$Message)
        $msg = "PASS: $Message"
        Write-Host $msg
        [void]$Script:CheckOutput.Add($msg)
    }

    # --- Read repository authority ---
    $repoRoot = $PSScriptRoot | Split-Path -Parent | Split-Path -Parent

    $rtFile = Join-Path $repoRoot "rust-toolchain.toml"
    if (-not (Test-Path $rtFile)) { Fail "rust-toolchain.toml missing"; return $Script:ExitCode }
    $rtContent = Get-Content $rtFile -Raw
    $derivedChannel = Read-TomlString -Content $rtContent -Key "channel"
    if (-not $derivedChannel) { Fail "rust-toolchain.toml: could not read channel"; return $Script:ExitCode }
    Pass "rust-toolchain.toml: channel $derivedChannel"
    if ($rtContent -match 'components\s*=.*"rustfmt"' -and $rtContent -match 'components\s*=.*"clippy"') {
        Pass "rust-toolchain.toml: rustfmt + clippy"
    } else {
        Fail "rust-toolchain.toml: missing components"
    }

    $cargoToml = Join-Path $repoRoot "tethers-0.1/host-rust/Cargo.toml"
    $cargoContent = Get-Content $cargoToml -Raw
    $derivedEdition = Read-TomlString -Content $cargoContent -Key "edition"
    $derivedRustVersion = Read-TomlString -Content $cargoContent -Key "rust-version"
    if ($derivedEdition -eq "2021") { Pass "Cargo.toml: edition 2021" } else { Fail "Cargo.toml: expected edition 2021, got $derivedEdition" }
    if ($derivedRustVersion -eq "$($derivedChannel -replace '\.\d+$','')") {
        Pass "Cargo.toml: rust-version $derivedRustVersion"
    } else {
        Fail "Cargo.toml: rust-version $derivedRustVersion does not match toolchain channel $derivedChannel"
    }

    $cargoLock = Join-Path $repoRoot "tethers-0.1/host-rust/Cargo.lock"
    if (Test-Path $cargoLock) { Pass "Cargo.lock present" } else { Fail "Cargo.lock missing" }

    # --- OcamlSwitchPath validation ---
    if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath)) {
        Fail "OcamlSwitchPath is required but was not supplied"
        return $Script:ExitCode
    }

    if (-not [System.IO.Path]::IsPathFullyQualified($OcamlSwitchPath)) {
        Fail "OcamlSwitchPath must be absolute: $OcamlSwitchPath"
        return $Script:ExitCode
    }

    $CanonicalSwitch = [System.IO.Path]::GetFullPath($OcamlSwitchPath)
    if (-not (Test-Path -LiteralPath $CanonicalSwitch -PathType Container)) {
        Fail "OcamlSwitchPath does not exist: $CanonicalSwitch"
        return $Script:ExitCode
    }

    $OpamDir = Join-Path $CanonicalSwitch "_opam"
    if (-not (Test-Path -LiteralPath $OpamDir -PathType Container)) {
        Fail "_opam not found under OcamlSwitchPath: $OpamDir"
        return $Script:ExitCode
    }

    $SwitchMarker = Join-Path $OpamDir ".opam-switch"
    if (-not ((Test-Path -LiteralPath $SwitchMarker -PathType Container) -or
              (Test-Path -LiteralPath $SwitchMarker -PathType Leaf))) {
        Fail ".opam-switch not found in _opam: $SwitchMarker"
        return $Script:ExitCode
    }

    # --- Rust process guard ---
    $hadRustupAutoInstall = Test-Path Env:RUSTUP_AUTO_INSTALL
    $prevRustupAutoInstall = if ($hadRustupAutoInstall) { $env:RUSTUP_AUTO_INSTALL } else { $null }

    try {
        $env:RUSTUP_AUTO_INSTALL = "0"

        # --- Rust toolchain verification ---
        $chMajorMinor = $derivedChannel -replace '\.\d+$', ''
        $toolchains = @(& rustup toolchain list 2>&1 | ForEach-Object { "$_" })
        if ($LASTEXITCODE -ne 0) {
            Fail "rustup toolchain list failed"
        } elseif (($toolchains -join "`n") -notmatch [regex]::Escape($derivedChannel)) {
            Fail "Rust toolchain $derivedChannel not found"
        } else {
            Pass "Rust toolchain $derivedChannel installed"
        }

        $components = @(& rustup component list --toolchain $derivedChannel --installed 2>&1 | ForEach-Object { "$_" })
        if ($LASTEXITCODE -ne 0) {
            Fail "rustup component list failed"
        } else {
            if (($components -join "`n") -notmatch "rustfmt") {
                Fail "rustfmt not installed for $derivedChannel"
            } else {
                Pass "rustfmt installed for $derivedChannel"
            }
            if (($components -join "`n") -notmatch "clippy") {
                Fail "clippy not installed for $derivedChannel"
            } else {
                Pass "clippy installed for $derivedChannel"
            }
        }

        if ($Script:ExitCode -eq 0) {
            $rustcVer = & rustup run $derivedChannel rustc --version 2>&1
            if ($rustcVer -match "^rustc $($derivedChannel)\s") { Pass "rustc: $rustcVer" } else { Fail "rustc version: $rustcVer" }

            $cargoVer = & rustup run $derivedChannel cargo --version 2>&1
            if ($cargoVer -match "cargo $($chMajorMinor)\.") { Pass "cargo: $cargoVer" } else { Fail "cargo version: $cargoVer" }

            $rustfmtVer = & rustup run $derivedChannel rustfmt --version 2>&1
            if ($LASTEXITCODE -eq 0 -and $rustfmtVer -match "rustfmt") { Pass "rustfmt: $rustfmtVer" } else { Fail "rustfmt version: $rustfmtVer" }

            $clippyVer = & rustup run $derivedChannel cargo clippy --version 2>&1
            if ($LASTEXITCODE -eq 0 -and $clippyVer -match "clippy") { Pass "clippy: $clippyVer" } else { Fail "clippy version: $clippyVer" }
        }

    } finally {
        if ($hadRustupAutoInstall) {
            $env:RUSTUP_AUTO_INSTALL = $prevRustupAutoInstall
        } else {
            Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
        }
    }

    if ($Script:ExitCode -ne 0) {
        return $Script:ExitCode
    }

    # --- OCaml verification ---
    $opamVer = & opam --version 2>&1
    if ($LASTEXITCODE -ne 0) { Fail "opam not found"; return $Script:ExitCode }
    $opamMajor = [int]($opamVer -split '\.')[0]
    $opamMinor = [int]($opamVer -split '\.')[1]
    if ($opamMajor -lt 2 -or ($opamMajor -eq 2 -and $opamMinor -lt 2)) {
        Fail "opam is $opamVer; requires 2.2 or newer"
    } else {
        Pass "opam $opamVer"
    }

    $switchRootRaw = & opam switch show --switch="$CanonicalSwitch" 2>&1
    if ($LASTEXITCODE -ne 0) { Fail "opam switch show failed: $switchRootRaw"; return $Script:ExitCode }
    $switchRootCanonical = [System.IO.Path]::GetFullPath($switchRootRaw.Trim())
    if (-not $switchRootCanonical.Equals($CanonicalSwitch, [System.StringComparison]::OrdinalIgnoreCase)) {
        Fail "Switch root mismatch: expected $CanonicalSwitch, got $switchRootCanonical"
        return $Script:ExitCode
    }
    Pass "Switch root matches"

    $prefixRaw = & opam var prefix --switch="$CanonicalSwitch" 2>&1
    if ($LASTEXITCODE -ne 0) { Fail "opam var prefix failed: $prefixRaw"; return $Script:ExitCode }
    $prefixCanonical = [System.IO.Path]::GetFullPath($prefixRaw.Trim())
    $expectedPrefix = [System.IO.Path]::GetFullPath($OpamDir)
    if (-not $prefixCanonical.Equals($expectedPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        Fail "Prefix mismatch: expected $expectedPrefix, got $prefixCanonical"
        return $Script:ExitCode
    }
    Pass "Prefix matches _opam"

    $ocamlcVer = & opam exec --switch="$CanonicalSwitch" -- ocamlc -version 2>&1
    if ($ocamlcVer -eq "5.5.0") { Pass "OCaml $ocamlcVer" } else { Fail "OCaml: expected 5.5.0, got $ocamlcVer" }

    $ocamloptVer = & opam exec --switch="$CanonicalSwitch" -- ocamlopt -version 2>&1
    if ($ocamloptVer -eq "5.5.0") { Pass "ocamlopt $ocamloptVer" } else { Fail "ocamlopt: expected 5.5.0, got $ocamloptVer" }

    $duneVer = & opam exec --switch="$CanonicalSwitch" -- dune --version 2>&1
    if ($duneVer -eq "3.24.0") { Pass "Dune $duneVer" } else { Fail "Dune: expected 3.24.0, got $duneVer" }

    $pkgList = & opam list --switch="$CanonicalSwitch" --installed --columns=name,version 2>&1
    if ($LASTEXITCODE -ne 0) { Fail "opam list failed"; return $Script:ExitCode }
    if ($pkgList -match "yojson\s+2\.2\.2") { Pass "Yojson 2.2.2" } else { Fail "Yojson 2.2.2 not found in installed packages" }

    # --- Repository OCaml checks ---
    $opamFile = Join-Path $repoRoot "tethers-0.1/engine-ocaml/tethers_engine.opam"
    $opamContent = Get-Content $opamFile -Raw
    if ($opamContent -match '"ocaml"\s*\{>=\s*"5\.5\.0"\s*&\s*<\s*"5\.6\.0"\s*\}') { Pass "OCaml range: >= 5.5.0 & < 5.6.0" } else { Fail "OCaml range does not match tightened constraint" }

    $lockedFile = Join-Path $repoRoot "tethers-0.1/engine-ocaml/tethers_engine.opam.locked"
    if (-not (Test-Path $lockedFile)) { Fail "opam.locked missing"; return $Script:ExitCode }
    $locked = Get-Content $lockedFile -Raw
    if ($locked -match '"ocaml"\s*\{=\s*"5\.5\.0"\}') { Pass "opam.locked: OCaml 5.5.0" } else { Fail "opam.locked: wrong OCaml" }
    if ($locked -match '"dune"\s*\{=\s*"3\.24\.0"\}') { Pass "opam.locked: Dune 3.24.0" } else { Fail "opam.locked: wrong Dune" }
    if ($locked -match '"yojson"\s*\{=\s*"2\.2\.2"\}') { Pass "opam.locked: Yojson 2.2.2" } else { Fail "opam.locked: wrong Yojson" }

    $duneProject = Join-Path $repoRoot "tethers-0.1/engine-ocaml/dune-project"
    $dp = Get-Content $duneProject -Raw
    if ($dp -match '\(lang dune 3\.10\)') { Pass "dune-project: lang dune 3.10" } else { Fail "dune-project: wrong language" }

    # --- Final result ---
    Write-Host ""
    $finalMsg = if ($Script:ExitCode -eq 0) { "All toolchain checks passed." } else { "One or more toolchain checks failed." }
    Write-Host $finalMsg
    [void]$Script:CheckOutput.Add($finalMsg)
    return $Script:ExitCode
}

# --- When executed as a script, invoke the function ---
if ($MyInvocation.InvocationName -ne '.') {
    if (-not $OcamlSwitchPath) {
        $OcamlSwitchPath = [string]::Empty
    }
    $result = Invoke-TethersToolchainCheck -SwitchPath $OcamlSwitchPath
    exit $result
}
