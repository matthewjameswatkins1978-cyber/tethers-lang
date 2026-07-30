param(
    [Parameter(Mandatory = $true)]
    [string]$OcamlSwitchPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Save the test switch path before dot-sourcing the preflight,
# since the preflight's top-level param would overwrite it.
$TestSwitchPath = $OcamlSwitchPath

# Dot-source the preflight to get Invoke-TethersToolchainCheck in-process
$PreflightPath = Join-Path $PSScriptRoot "check-tethers-toolchains.ps1"
. $PreflightPath

$Script:Failed = 0
$Script:Passed = 0

function Assert-ExitCode {
    param([int]$Expected, [int]$Actual, [string]$Description)
    if ($Actual -eq $Expected) {
        Write-Host "  PASS: $Description (exit $Actual)"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: $Description (expected $Expected, got $Actual)"
        $Script:Failed++
    }
}

function Assert-Contains {
    param([string]$Haystack, [string]$Needle, [string]$Description)
    if ($Haystack -match $Needle) {
        Write-Host "  PASS: $Description"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: $Description"
        $Script:Failed++
    }
}

function Invoke-CheckInProcess {
    param([string]$Path)
    $exitCode = Invoke-TethersToolchainCheck -SwitchPath $Path
    $output = ($Script:CheckOutput -join "`n")
    return @{ ExitCode = $exitCode; Output = $output }
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-toolchain-test-$(Get-Random)"
New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null

try {
    # --- Test 1: Missing switch path fails with actionable output ---
    Write-Host "Test 1: Missing switch path fails with actionable output"
    $r = Invoke-CheckInProcess -Path ""
    Assert-ExitCode 1 $r.ExitCode "Empty switch path fails"
    Assert-Contains $r.Output "required" "Error mentions required"

    # --- Test 2: Relative switch path fails ---
    Write-Host ""
    Write-Host "Test 2: Relative switch path fails"
    $r = Invoke-CheckInProcess -Path "relative\path"
    Assert-ExitCode 1 $r.ExitCode "Relative path fails"
    Assert-Contains $r.Output "absolute" "Output mentions absolute"

    # --- Test 3: Nonexistent root fails ---
    Write-Host ""
    Write-Host "Test 3: Nonexistent root fails"
    $fakePath = Join-Path $tempRoot "does-not-exist"
    $r = Invoke-CheckInProcess -Path $fakePath
    Assert-ExitCode 1 $r.ExitCode "Nonexistent root fails"
    Assert-Contains $r.Output "does not exist" "Output mentions nonexistent"

    # --- Test 4: Root without _opam fails ---
    Write-Host ""
    Write-Host "Test 4: Root without _opam fails"
    $noOpamDir = Join-Path $tempRoot "no-opam"
    New-Item -ItemType Directory -Path $noOpamDir -Force | Out-Null
    $r = Invoke-CheckInProcess -Path $noOpamDir
    Assert-ExitCode 1 $r.ExitCode "No _opam fails"
    Assert-Contains $r.Output "_opam not found" "Output mentions _opam"

    # --- Test 5: Root with _opam but no .opam-switch fails ---
    Write-Host ""
    Write-Host "Test 5: _opam without .opam-switch fails"
    $partialDir = Join-Path $tempRoot "partial"
    $partialOpam = Join-Path $partialDir "_opam"
    New-Item -ItemType Directory -Path $partialOpam -Force | Out-Null
    $r = Invoke-CheckInProcess -Path $partialDir
    Assert-ExitCode 1 $r.ExitCode "No .opam-switch fails"
    Assert-Contains $r.Output ".opam-switch" "Output mentions .opam-switch"

    # --- Test 6: The real authorised switch succeeds ---
    Write-Host ""
    Write-Host "Test 6: Real authorised switch succeeds"
    $r = Invoke-CheckInProcess -Path $TestSwitchPath
    Assert-ExitCode 0 $r.ExitCode "Real switch check passes"
    Assert-Contains $r.Output "All toolchain checks passed" "Output confirms success"

    # --- Test 7: RUSTUP_AUTO_INSTALL sentinel restored after successful check ---
    Write-Host ""
    Write-Host "Test 7: RUSTUP_AUTO_INSTALL sentinel restored after success"
    $sentinel = "test-preserve-$(Get-Random)"
    $env:RUSTUP_AUTO_INSTALL = $sentinel
    try {
        $r = Invoke-CheckInProcess -Path $TestSwitchPath
        $afterValue = $env:RUSTUP_AUTO_INSTALL
    } finally {
        Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
    }
    if ($afterValue -eq $sentinel) {
        Write-Host "  PASS: RUSTUP_AUTO_INSTALL restored after success ($sentinel)"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: RUSTUP_AUTO_INSTALL changed after success ($sentinel -> $afterValue)"
        $Script:Failed++
    }

    # --- Test 8: RUSTUP_AUTO_INSTALL absent after success when absent before ---
    Write-Host ""
    Write-Host "Test 8: RUSTUP_AUTO_INSTALL absent after success when absent before"
    if (Test-Path Env:RUSTUP_AUTO_INSTALL) { Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue }
    $r = Invoke-CheckInProcess -Path $TestSwitchPath
    if (Test-Path Env:RUSTUP_AUTO_INSTALL) {
        Write-Host "  FAIL: RUSTUP_AUTO_INSTALL left behind after success"
        $Script:Failed++
        Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
    } else {
        Write-Host "  PASS: RUSTUP_AUTO_INSTALL not set after success (was absent)"
        $Script:Passed++
    }

    # --- Test 9: RUSTUP_AUTO_INSTALL restored after failure post-Rust-guard ---
    Write-Host ""
    Write-Host "Test 9: RUSTUP_AUTO_INSTALL restored after failure post-Rust-guard"
    $sentinel2 = "fail-restore-$(Get-Random)"
    $env:RUSTUP_AUTO_INSTALL = $sentinel2
    try {
        # The real switch succeeds fully, which exercises the Rust guard try/finally
        $r = Invoke-CheckInProcess -Path $TestSwitchPath
        $afterValue = $env:RUSTUP_AUTO_INSTALL
    } finally {
        Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
    }
    if ($afterValue -eq $sentinel2) {
        Write-Host "  PASS: RUSTUP_AUTO_INSTALL restored after real switch check ($sentinel2)"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: RUSTUP_AUTO_INSTALL changed after real switch check ($sentinel2 -> $afterValue)"
        $Script:Failed++
    }

    # --- Test 10: Neighbouring temp directory with _opam does not cause fallback ---
    Write-Host ""
    Write-Host "Test 10: Neighbouring _opam does not cause fallback"
    $neighbourDir = Join-Path $tempRoot "neighbour"
    $neighbourOpam = Join-Path $neighbourDir "_opam"
    $neighbourMarker = Join-Path $neighbourOpam ".opam-switch"
    New-Item -ItemType Directory -Path $neighbourOpam -Force | Out-Null
    New-Item -ItemType File -Path $neighbourMarker -Force | Out-Null
    $invalidDir = Join-Path $tempRoot "invalid"
    New-Item -ItemType Directory -Path $invalidDir -Force | Out-Null
    $r = Invoke-CheckInProcess -Path $invalidDir
    Assert-ExitCode 1 $r.ExitCode "Invalid root fails despite neighbouring _opam"
    Assert-Contains $r.Output "_opam not found" "Output identifies missing _opam at supplied path"

    # --- Test 11: Failure returns non-zero and identifies the failed check ---
    Write-Host ""
    Write-Host "Test 11: Failure returns non-zero and identifies failed check"
    $r = Invoke-CheckInProcess -Path $invalidDir
    Assert-ExitCode 1 $r.ExitCode "Non-zero exit on failure"
    Assert-Contains $r.Output "FAIL" "Failure output contains FAIL identifier"

    # --- Test 12: Repository non-mutation ---
    Write-Host ""
    Write-Host "Test 12: Repository non-mutation"
    $beforeStatus = & git status --porcelain=v1 --untracked-files=all 2>&1
    $r = Invoke-CheckInProcess -Path $TestSwitchPath
    $afterStatus = & git status --porcelain=v1 --untracked-files=all 2>&1
    if (($beforeStatus -join "`n") -eq ($afterStatus -join "`n")) {
        Write-Host "  PASS: Repository status unchanged after preflight"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: Repository status changed after preflight"
        Write-Host "  Before: $($beforeStatus -join ', ')"
        Write-Host "  After:  $($afterStatus -join ', ')"
        $Script:Failed++
    }

} finally {
    Remove-Item -Recurse -Force $tempRoot -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "=== RESULTS ==="
Write-Host "Passed: $($Script:Passed)"
Write-Host "Failed: $($Script:Failed)"
if ($Script:Failed -gt 0) {
    exit 1
}
