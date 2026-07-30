param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$PreflightPath = Join-Path $PSScriptRoot "check-tethers-toolchains.ps1"
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

function Assert-NotContains {
    param([string]$Haystack, [string]$Needle, [string]$Description)
    if ($Haystack -notmatch $Needle) {
        Write-Host "  PASS: $Description"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: $Description"
        $Script:Failed++
    }
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-toolchain-test-$(Get-Random)"
New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null

try {
    # --- Test 1: Missing OcamlSwitchPath ---
    Write-Host "Test 1: Missing OcamlSwitchPath"
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath 2>&1
    Assert-ExitCode 1 $LASTEXITCODE "Missing switch path fails"
    # PowerShell's mandatory parameter error won't contain our "FAIL" text
    Assert-Contains ($out -join "`n") "OcamlSwitchPath" "Error mentions OcamlSwitchPath"

    # --- Test 2: Relative OcamlSwitchPath ---
    Write-Host ""
    Write-Host "Test 2: Relative OcamlSwitchPath"
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath "relative\path" 2>&1
    Assert-ExitCode 1 $LASTEXITCODE "Relative path fails"
    Assert-Contains ($out -join "`n") "absolute" "Output mentions absolute"

    # --- Test 3: Nonexistent switch root ---
    Write-Host ""
    Write-Host "Test 3: Nonexistent switch root"
    $fakePath = Join-Path $tempRoot "does-not-exist"
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $fakePath 2>&1
    Assert-ExitCode 1 $LASTEXITCODE "Nonexistent root fails"
    Assert-Contains ($out -join "`n") "does not exist" "Output mentions nonexistent"

    # --- Test 4: Root without _opam ---
    Write-Host ""
    Write-Host "Test 4: Root without _opam"
    $noOpamDir = Join-Path $tempRoot "no-opam"
    New-Item -ItemType Directory -Path $noOpamDir -Force | Out-Null
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $noOpamDir 2>&1
    Assert-ExitCode 1 $LASTEXITCODE "No _opam fails"
    Assert-Contains ($out -join "`n") "_opam not found" "Output mentions _opam"

    # --- Test 5: Root with _opam but no .opam-switch ---
    Write-Host ""
    Write-Host "Test 5: _opam without .opam-switch"
    $partialDir = Join-Path $tempRoot "partial"
    $partialOpam = Join-Path $partialDir "_opam"
    New-Item -ItemType Directory -Path $partialOpam -Force | Out-Null
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $partialDir 2>&1
    Assert-ExitCode 1 $LASTEXITCODE "No .opam-switch fails"
    Assert-Contains ($out -join "`n") ".opam-switch" "Output mentions .opam-switch"

    # --- Test 6: RUSTUP_AUTO_INSTALL preservation ---
    Write-Host ""
    Write-Host "Test 6: RUSTUP_AUTO_INSTALL preservation"
    $prevValue = "test-preserve-value"
    $env:RUSTUP_AUTO_INSTALL = $prevValue
    try {
        $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $partialDir 2>&1
    } finally {
        $afterValue = $env:RUSTUP_AUTO_INSTALL
        Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
    }
    if ($afterValue -eq $prevValue) {
        Write-Host "  PASS: RUSTUP_AUTO_INSTALL preserved ($afterValue)"
        $Script:Passed++
    } else {
        Write-Host "  FAIL: RUSTUP_AUTO_INSTALL changed ($prevValue -> $afterValue)"
        $Script:Failed++
    }

    # --- Test 7: RUSTUP_AUTO_INSTALL removal when absent ---
    Write-Host ""
    Write-Host "Test 7: RUSTUP_AUTO_INSTALL removal when absent"
    if (Test-Path Env:RUSTUP_AUTO_INSTALL) { Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue }
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $partialDir 2>&1
    if (Test-Path Env:RUSTUP_AUTO_INSTALL) {
        Write-Host "  FAIL: RUSTUP_AUTO_INSTALL left behind"
        $Script:Failed++
    } else {
        Write-Host "  PASS: RUSTUP_AUTO_INSTALL not set after absent"
        $Script:Passed++
    }

    # --- Test 8: No fallback search ---
    Write-Host ""
    Write-Host "Test 8: No fallback search for _opam"
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $noOpamDir 2>&1
    Assert-NotContains ($out -join "`n") "worktree" "No worktree search"
    Assert-NotContains ($out -join "`n") "scan" "No scan mention"

    # --- Test 9: Output is non-zero and identifies failure ---
    Write-Host ""
    Write-Host "Test 9: Failure output content"
    $out = & pwsh -NoProfile -ExecutionPolicy Bypass -File $PreflightPath -OcamlSwitchPath $noOpamDir 2>&1
    Assert-ExitCode 1 $LASTEXITCODE "Non-zero exit on failure"
    $joined = $out -join "`n"
    Assert-Contains $joined "FAIL" "Failure output contains FAIL identifier"

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
