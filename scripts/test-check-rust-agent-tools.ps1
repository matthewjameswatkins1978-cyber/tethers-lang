[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$checkerScript = Join-Path ($PSScriptRoot | Split-Path -Parent) 'scripts/check-rust-agent-tools.ps1'
. $checkerScript
$testRepoRoot = $PSScriptRoot | Split-Path -Parent
$passed = 0
$failed = 0
$testRoot = Join-Path $env:TEMP ('tethers-m01b-checker-' + [guid]::NewGuid())

function Assert-Result {
    param([bool]$Condition, [string]$Name)
    if ($Condition) { Write-Host "PASS: $Name"; $script:passed++ }
    else { Write-Host "FAIL: $Name"; $script:failed++ }
}

function New-TestRepository {
    param([string]$Name)
    $path = Join-Path $testRoot $Name
    & git clone --quiet $testRepoRoot $path
    if ($LASTEXITCODE -ne 0) { throw "git clone failed for $Name" }
    Remove-Item -LiteralPath (Join-Path $path 'tethers-0.1/host-rust/.config/nextest.toml') -Force -ErrorAction SilentlyContinue
    return [string]$path
}

function Invoke-Checker {
    param([string]$Path, [string]$OpenCodePath)
    return (Invoke-RustAgentToolCheck -RepoRoot $Path -OpenCodePath $OpenCodePath)
}

function Assert-CheckerExitCode {
    param([string]$Path, [string]$OpenCodePath, [int]$Expected, [string]$Name)
    $actual = Invoke-Checker -Path $Path -OpenCodePath $OpenCodePath
    Assert-Result ($actual -eq $Expected) $Name
}

function New-FakeOpenCode {
    $path = Join-Path $testRoot 'fake-opencode.cmd'
    @'
@echo off
if "%1"=="--version" (
  echo OpenCode 1.0.0-test
  exit /b 0
)
if "%1"=="debug" if "%2"=="config" (
  echo {"lsp":true,"permission":{"lsp":"allow"}}
  exit /b 0
)
exit /b 1
'@ | Set-Content -LiteralPath $path -Encoding ASCII
    return [string]$path
}

try {
    New-Item -ItemType Directory -Path $testRoot -ErrorAction Stop | Out-Null
    $fakeOpenCode = [string](New-FakeOpenCode)

    $missing = New-TestRepository -Name 'missing-config'
    Remove-Item -LiteralPath (Join-Path $missing 'tools/rust-agent-tools.json') -Force
    Assert-CheckerExitCode -Path $missing -OpenCodePath $fakeOpenCode -Expected 1 -Name 'missing tool JSON is rejected'

    $malformed = New-TestRepository -Name 'malformed-json'
    Set-Content -LiteralPath (Join-Path $malformed 'tools/rust-agent-tools.json') -Value '{bad' -Encoding UTF8
    Assert-CheckerExitCode -Path $malformed -OpenCodePath $fakeOpenCode -Expected 1 -Name 'malformed JSON is rejected'

    $wrongSchema = New-TestRepository -Name 'wrong-schema'
    Set-Content -LiteralPath (Join-Path $wrongSchema 'tools/rust-agent-tools.json') -Value '{"schema":2,"cargo_nextest":"0.9.140","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"toolchain-component"}' -Encoding UTF8
    Assert-CheckerExitCode -Path $wrongSchema -OpenCodePath $fakeOpenCode -Expected 1 -Name 'wrong schema is rejected'

    $unknown = New-TestRepository -Name 'unknown-field'
    Set-Content -LiteralPath (Join-Path $unknown 'tools/rust-agent-tools.json') -Value '{"schema":1,"cargo_nextest":"0.9.140","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"toolchain-component","extra":true}' -Encoding UTF8
    Assert-CheckerExitCode -Path $unknown -OpenCodePath $fakeOpenCode -Expected 1 -Name 'unknown tool JSON field is rejected'

    $badRa = New-TestRepository -Name 'bad-ra'
    Set-Content -LiteralPath (Join-Path $badRa 'tools/rust-agent-tools.json') -Value '{"schema":1,"cargo_nextest":"0.9.140","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"weekly"}' -Encoding UTF8
    Assert-CheckerExitCode -Path $badRa -OpenCodePath $fakeOpenCode -Expected 1 -Name 'non-component rust-analyzer is rejected'

    $badVersion = New-TestRepository -Name 'bad-version'
    Set-Content -LiteralPath (Join-Path $badVersion 'tools/rust-agent-tools.json') -Value '{"schema":1,"cargo_nextest":"nine","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"toolchain-component"}' -Encoding UTF8
    Assert-CheckerExitCode -Path $badVersion -OpenCodePath $fakeOpenCode -Expected 1 -Name 'malformed version is rejected'

    $accepted = New-TestRepository -Name 'accepted'
    Assert-CheckerExitCode -Path $accepted -OpenCodePath $fakeOpenCode -Expected 0 -Name 'supplied executable proves accepted effective configuration'

    $missingOpenCode = New-TestRepository -Name 'missing-opencode'
    $oldOpenCodeBin = $env:OPENCODE_BIN
    Remove-Item Env:OPENCODE_BIN -ErrorAction SilentlyContinue
    Assert-CheckerExitCode -Path $missingOpenCode -OpenCodePath $null -Expected 1 -Name 'missing OpenCode fails closed'
    if ($null -ne $oldOpenCodeBin) { $env:OPENCODE_BIN = $oldOpenCodeBin }

    $invalidOpenCode = New-TestRepository -Name 'invalid-opencode'
    $invalidPath = Join-Path $testRoot 'missing.exe'
    Assert-CheckerExitCode -Path $invalidOpenCode -OpenCodePath $invalidPath -Expected 1 -Name 'invalid explicit OpenCode path fails'

    $nonMutation = New-TestRepository -Name 'non-mutation'
    $before = (& git -C $nonMutation status --porcelain=v1 | Out-String).Trim()
    $exitCode = Invoke-Checker -Path $nonMutation -OpenCodePath $fakeOpenCode
    $after = (& git -C $nonMutation status --porcelain=v1 | Out-String).Trim()
    Assert-Result ($exitCode -eq 0 -and $before -eq $after) 'checker does not mutate repository'
} finally {
    Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "Checks: $passed passed, $failed failed."
if ($failed -gt 0) { exit 1 }
