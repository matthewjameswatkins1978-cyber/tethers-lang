[CmdletBinding()]
param(
    [string]$OcamlSwitchPath,
    [switch]$Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$prepare = Join-Path $repositoryRoot 'scripts/prepare-current-engine.ps1'

& pwsh.exe -NoProfile -File $prepare -OcamlSwitchPath $OcamlSwitchPath -ReleaseMode:$Release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$provenancePath = Join-Path $repositoryRoot 'verification/current-engine-provenance.json'
$provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
if ($provenance.schema -ne 'tethers.engine/1' -or $provenance.status -ne 'pass') {
    Write-Host 'VERIFICATION PREREQUISITE FAILED'
    Write-Host 'Current engine provenance manifest is missing or invalid; Rust tests were not attempted.'
    exit 1
}
$enginePath = Join-Path $repositoryRoot ($provenance.binary_relative_path -replace '/', '\')
$actualHash = (Get-FileHash -LiteralPath $enginePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualHash -ne $provenance.binary_sha256 -or $provenance.source_commit -ne ((& git -C $repositoryRoot rev-parse HEAD).Trim().ToLowerInvariant())) {
    Write-Host 'VERIFICATION PREREQUISITE FAILED'
    Write-Host 'Current engine provenance no longer matches this source checkout; Rust tests were not attempted.'
    exit 1
}

$env:TETHERS_VERIFIED_ENGINE = $enginePath
$env:TETHERS_ENGINE_PROVENANCE = $provenancePath
$manifest = Join-Path $repositoryRoot 'tethers-0.1/host-rust/Cargo.toml'
$args = @('test', '--manifest-path', $manifest, '--all-targets', '--all-features', '--locked', '--', '--test-threads=1')
if ($Release) { $args = @('test', '--manifest-path', $manifest, '--all-targets', '--all-features', '--locked', '--release', '--', '--test-threads=1') }
& cargo @args
exit $LASTEXITCODE
