[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifest = Join-Path ([System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))) 'tethers-0.1/host-rust/Cargo.toml'
$output = @(& cargo check --manifest-path $manifest --all-targets --all-features --locked 2>&1 | ForEach-Object { "$($_)" })
if ($LASTEXITCODE -ne 0) { throw 'Cargo warning-ratchet probe could not complete.' }
$warnings = @($output | Where-Object { $_ -match '(?i)warning:' })
$unaccepted = @($warnings | Where-Object {
    $_ -notmatch '(?i)present in multiple build targets' -and
    $_ -notmatch '(?i)duplicate target'
})
if ($unaccepted.Count -gt 0) {
    throw "New warning(s) are outside the accepted baseline: $($unaccepted -join ' | ')"
}
if ($warnings.Count -gt 0) {
    Write-Host 'PASS warning ratchet: existing duplicate-target warning remains within baseline'
} else {
    Write-Host 'PASS warning ratchet: no warnings emitted'
}
