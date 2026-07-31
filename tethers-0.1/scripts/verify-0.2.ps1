param(
    [string[]]$Suite,
    [switch]$List
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($null -eq $Suite) { $Suite = @() }
$expanded = @()
foreach ($raw in @($Suite) + @($args)) {
    foreach ($part in ($raw -split ',')) {
        $part = $part.Trim()
        if ($part.Length -gt 0) { $expanded += $part }
    }
}
$Suite = $expanded

$ScriptRoot = $PSScriptRoot
if ([string]::IsNullOrEmpty($ScriptRoot)) {
    $ScriptRoot = Split-Path -Parent -Path $MyInvocation.MyCommand.Definition
}

$SuiteMap = [ordered]@{
    "J13A" = "test-j13a-check.ps1"
    "J13B" = "test-j13b-run.ps1"
    "J13C" = "test-j13c-trail.ps1"
}

$DefaultOrder = @("J13A", "J13B", "J13C")
$KnownIds = $SuiteMap.Keys

if ($List) {
    foreach ($id in $DefaultOrder) {
        Write-Output "$id $($SuiteMap[$id])"
    }
    exit 0
}

if ($null -eq $Suite -or $Suite.Count -eq 0) {
    $selected = $DefaultOrder
}
else {
    foreach ($id in $Suite) {
        if (-not $SuiteMap.Contains($id)) {
            [Console]::Error.WriteLine("verify-0.2.ps1: unknown suite id: '$id'. Valid ids: $($KnownIds -join ', ')")
            exit 2
        }
    }

    $seen = @{}
    foreach ($id in $Suite) {
        if ($seen.Contains($id)) {
            [Console]::Error.WriteLine("verify-0.2.ps1: duplicate suite id: '$id'. Each suite may be selected at most once.")
            exit 2
        }
        $seen[$id] = $true
    }

    $selected = $Suite
}

$passed = 0
$failed = 0

foreach ($id in $selected) {
    $fileName = $SuiteMap[$id]
    $childPath = Join-Path -Path $ScriptRoot -ChildPath $fileName

    Write-Output "SUITE $id START $fileName"

    if (-not (Test-Path -LiteralPath $childPath -PathType Leaf)) {
        Write-Output "$id | ERROR missing child script: $childPath"
        Write-Output "SUITE $id FAIL exit=-1"
        $failed++
        continue
    }

    $childOutput = & pwsh.exe -NoProfile -ExecutionPolicy Bypass -File $childPath 2>&1
    $childExit = $LASTEXITCODE

    foreach ($rawLine in $childOutput) {
        $line = [string]$rawLine
        if ($line.Length -eq 0) {
            continue
        }
        Write-Output "$id | $line"
    }

    if ($childExit -eq 0) {
        Write-Output "SUITE $id PASS exit=0"
        $passed++
    }
    else {
        Write-Output "SUITE $id FAIL exit=$childExit"
        $failed++
    }
}

Write-Output "============================================"
Write-Output "J15 CONSOLIDATED VERIFICATION"
Write-Output "TOTAL: $($selected.Count) suites, $passed passed, $failed failed"
if ($failed -eq 0) {
    Write-Output "RESULT: PASS"
    Write-Output "============================================"
    exit 0
}
else {
    Write-Output "RESULT: FAIL"
    Write-Output "============================================"
    exit 1
}
