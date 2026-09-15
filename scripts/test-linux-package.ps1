[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Archive
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT) {
    throw 'Linux package smoke must run on native GNU/Linux.'
}
$archivePath = [System.IO.Path]::GetFullPath($Archive)
if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
    throw "package archive not found: $archivePath"
}
$root = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-linux-smoke-$([guid]::NewGuid())"
$extract = Join-Path $root 'extract'
New-Item -ItemType Directory -Force -Path $extract | Out-Null
try {
    & tar -xzf $archivePath -C $extract
    if ($LASTEXITCODE -ne 0) { throw "tar extraction failed with exit code $LASTEXITCODE" }
    $package = Get-ChildItem -LiteralPath $extract -Directory | Select-Object -First 1
    if ($null -eq $package) { throw 'package archive contained no top-level directory' }
    foreach ($name in @('tethers', 'tethers-engine', 'manifest.json', 'SHA256SUMS')) {
        if (-not (Test-Path -LiteralPath (Join-Path $package.FullName $name) -PathType Leaf)) {
            throw "clean package is missing $name"
        }
    }
    $manifest = Get-Content -Raw (Join-Path $package.FullName 'manifest.json') | ConvertFrom-Json
    foreach ($binary in $manifest.binaries) {
        $actual = (Get-FileHash (Join-Path $package.FullName $binary.name) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $binary.sha256) { throw "binary digest mismatch: $($binary.name)" }
    }
    $hostBinary = Join-Path $package.FullName 'tethers'
    & $hostBinary --version | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'clean package --version failed' }
    & $hostBinary --help | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'clean package --help failed' }
    $describe = & $hostBinary describe --json
    if ($LASTEXITCODE -ne 0) { throw 'clean package describe --json failed' }
    $describe | ConvertFrom-Json | Out-Null
    $doctor = & $hostBinary doctor --json
    if ($LASTEXITCODE -ne 0) { throw 'clean package doctor --json failed' }
    $doctor | ConvertFrom-Json | Out-Null
    Write-Host 'PASS clean Linux package install smoke (--version, --help, describe, doctor)'
}
finally {
    if (Test-Path -LiteralPath $root) {
        Remove-Item -LiteralPath $root -Recurse -Force
    }
}
