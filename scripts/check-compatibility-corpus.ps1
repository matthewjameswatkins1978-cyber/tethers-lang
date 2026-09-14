[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = Join-Path ([System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))) 'compat/0.7'
$required = @(
    'README.md',
    'config/evaluation-request.json',
    'cli/evaluation-request.json',
    'manifests/fixture-ping.json',
    'plans/record-completed-task.json',
    'trails/record-completed-task.json',
    'mcp/validate-valid.stdin.jsonl',
    'mcp/validate-valid.stdout.jsonl'
)
foreach ($relative in $required) {
    $path = Join-Path $root ($relative -replace '/', '\')
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing compatibility fixture: $relative" }
}

function Read-Json($relative) {
    $path = Join-Path $root ($relative -replace '/', '\')
    try { return Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -ErrorAction Stop }
    catch { throw "Invalid compatibility JSON ${relative}: $($_.Exception.Message)" }
}

$request = Read-Json 'config/evaluation-request.json'
$manifest = Read-Json 'manifests/fixture-ping.json'
$plan = Read-Json 'plans/record-completed-task.json'
if ($request.protocol_version -ne '0.1' -or $request.language_version -ne '0.1') { throw 'Compatibility request is missing its historical version identifiers.' }
if ([string]::IsNullOrWhiteSpace($request.tether.source)) { throw 'Human Tether source is missing from compatibility request.' }
if ($manifest.manifest_format_version -ne '1.0' -or [string]::IsNullOrWhiteSpace($manifest.digest)) { throw 'Capability manifest identity is incomplete.' }
if ($plan.protocol_version -ne '0.1' -or $null -eq $plan.plan.actions -or $plan.plan.actions.Count -ne 1) { throw 'Plan compatibility fixture is not the expected ordered-action shape.' }
foreach ($relative in @('mcp/validate-valid.stdin.jsonl','mcp/validate-valid.stdout.jsonl')) {
    foreach ($line in [System.IO.File]::ReadLines((Join-Path $root ($relative -replace '/', '\')))) {
        if ([string]::IsNullOrWhiteSpace($line)) { throw "Blank JSONL line in $relative" }
        try { $line | ConvertFrom-Json -ErrorAction Stop | Out-Null } catch { throw "Invalid JSONL in ${relative}: $($_.Exception.Message)" }
    }
}
Write-Host 'PASS compatibility corpus seed (0.7 fixtures, provenance and required version identifiers)'
