# J13A check command acceptance tests
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$HostBinary = Join-Path $HostDir "target/debug/tethers-reference-host.exe"
$HostBinary = (Resolve-Path $HostBinary).Path

if (-not (Test-Path $HostBinary)) {
    Push-Location $HostDir
    try { & cargo build; if ($LASTEXITCODE -ne 0) { throw "build failed" } }
    finally { Pop-Location }
}

$passed = 0
$failed = 0

function Run-Host {
    param([string[]]$ArgList)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $script:HostBinary
    $psi.Arguments = $ArgList -join " "
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $proc = [System.Diagnostics.Process]::Start($psi)
    $out = $proc.StandardOutput.ReadToEnd()
    $proc.WaitForExit(10000) | Out-Null
    if (-not $proc.HasExited) { $proc.Kill(); throw "timeout" }
    @{ ExitCode = $proc.ExitCode; Stdout = $out }
}

function Test-Case {
    param([string]$Name, [scriptblock]$Test)
    Write-Output "TEST: $Name"
    try {
        & $Test
        Write-Output "  PASS"
        $script:passed++
    } catch {
        Write-Output "  FAIL: $_"
        $script:failed++
    }
}

try {
    # 1: Invalid CLI usage -> exit 2, one JSON envelope
    Test-Case "invalid command returns exit 2" {
        $r = Run-Host -ArgList @("nonexistent-command")
        if ($r.ExitCode -ne 2) { throw "expected exit 2, got $($r.ExitCode)" }
        $env = $r.Stdout.Trim() | ConvertFrom-Json
        if ($env.status -ne "invalid_cli_usage") { throw "expected invalid_cli_usage, got $($env.status)" }
    }

    # 2: Misspelled runn never enters legacy
    Test-Case "misspelled runn blocked" {
        $r = Run-Host -ArgList @("runn", "engine.exe", "req.json")
        if ($r.ExitCode -ne 2) { throw "expected exit 2, got $($r.ExitCode)" }
        $env = $r.Stdout.Trim() | ConvertFrom-Json
        if ($env.status -ne "invalid_cli_usage") { throw "expected invalid_cli_usage" }
    }

    # 3: No command produces envelope
    Test-Case "no command" {
        $r = Run-Host -ArgList @()
        if ($r.ExitCode -ne 2) { throw "expected exit 2, got $($r.ExitCode)" }
        $env = $r.Stdout.Trim() | ConvertFrom-Json
        if ($env.status -ne "invalid_cli_usage") { throw "expected invalid_cli_usage" }
    }

    # 4: __legacy reachable (returns failed/6, not invalid_cli_usage/2)
    Test-Case "__legacy reachable" {
        $r = Run-Host -ArgList @("__legacy", "engine.exe", "req.json")
        if ($r.ExitCode -eq 2) { throw "__legacy returned invalid_cli_usage (exit 2)" }
    }

    # 5: Envelope has no timestamp
    Test-Case "no timestamp" {
        $r = Run-Host -ArgList @("nonexistent-command")
        if ($r.Stdout -match "timestamp") { throw "contains timestamp" }
    }

    # 6: Output parses as valid JSON
    Test-Case "output parses as JSON" {
        $r = Run-Host -ArgList @("nonexistent-command")
        $null = $r.Stdout.Trim() | ConvertFrom-Json
    }

    # 7: Check with nonexistent config returns invalid_data/3
    Test-Case "nonexistent config returns invalid_data" {
        $r = Run-Host -ArgList @("check", "--config", "no-such-config.json", "--engine", "no-such-engine.exe")
        if ($r.ExitCode -ne 3) { throw "expected exit 3, got $($r.ExitCode)" }
        $env = $r.Stdout.Trim() | ConvertFrom-Json
        if ($env.status -ne "invalid_data") { throw "expected invalid_data, got $($env.status)" }
    }

    # 8: No tools/call in check output
    Test-Case "no provider tools/call" {
        $r = Run-Host -ArgList @("check", "--config", "no.json", "--engine", "no.exe")
        if ($r.Stdout -match "tools/call") { throw "check must not invoke tools/call" }
    }

    Write-Output ""
    Write-Output "J13A acceptance: $passed passed, $failed failed"
    if ($failed -gt 0) { throw "$failed test(s) failed" }
    Write-Output "PASS test-j13a-check"

} finally {
    Remove-Item -LiteralPath (Join-Path $PSScriptRoot "debug-test.ps1") -ErrorAction SilentlyContinue
}
