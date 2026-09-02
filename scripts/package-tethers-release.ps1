[CmdletBinding()]
param(
    [ValidateSet('windows-x64', 'linux-x64-musl')]
    [string]$Target = 'windows-x64',
    [string]$Version = '0.5.0',
    [string]$Output = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$hostManifest = Join-Path $repo 'tethers-0.1\host-rust\Cargo.toml'
$portableRoot = Join-Path $repo 'tethers-0.1\portable-rust'
$cargoTarget = if ($Target -eq 'windows-x64') { 'x86_64-pc-windows-msvc' } else { 'x86_64-unknown-linux-musl' }
$extension = if ($Target -eq 'windows-x64') { '.exe' } else { '' }
$dist = Join-Path $repo 'dist'
$stage = Join-Path $dist "tethers-$Version-$Target"
if ([string]::IsNullOrWhiteSpace($Output)) {
    $Output = Join-Path $dist "tethers-$Version-$Target.zip"
}
$Output = [System.IO.Path]::GetFullPath($Output)

if (Test-Path -LiteralPath $Output) {
    throw "output already exists; refusing to replace: $Output"
}
if (Test-Path -LiteralPath $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
New-Item -ItemType Directory -Force -Path `
    (Join-Path $stage 'bin'),
    (Join-Path $stage 'portable'),
    (Join-Path $stage 'docs') | Out-Null

try {
    & cargo build --release --locked --target $cargoTarget --manifest-path $hostManifest `
        --bin tethers --bin agent_workspace_provider --bin agent_coding_provider
    if ($LASTEXITCODE -ne 0) { throw "native host build failed with exit code $LASTEXITCODE" }
    Push-Location $portableRoot
    try {
        & cargo build --release --locked --target $cargoTarget --manifest-path (Join-Path $portableRoot 'Cargo.toml')
        if ($LASTEXITCODE -ne 0) { throw "portable workbench build failed with exit code $LASTEXITCODE" }
    }
    finally { Pop-Location }

    $hostRelease = Join-Path $repo "tethers-0.1\host-rust\target\$cargoTarget\release"
    $portableRelease = Join-Path $portableRoot "target\$cargoTarget\release"
    foreach ($name in @('tethers', 'agent_workspace_provider', 'agent_coding_provider')) {
        $source = Join-Path $hostRelease "$name$extension"
        if (-not (Test-Path -LiteralPath $source)) { throw "missing native release binary: $source" }
        Copy-Item -LiteralPath $source -Destination (Join-Path $stage "bin\$name$extension")
    }
    $portable = Join-Path $portableRelease "tethers$extension"
    if (-not (Test-Path -LiteralPath $portable)) { throw "missing portable release binary: $portable" }
    Copy-Item -LiteralPath $portable -Destination (Join-Path $stage "portable\tethers$extension")

    Copy-Item -LiteralPath (Join-Path $repo 'README.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'QUICKSTART.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'docs\AGENT_QUICKSTART.md') -Destination (Join-Path $stage 'docs')
    Copy-Item -LiteralPath (Join-Path $repo 'docs\TETHERS_0_5_RELEASE.md') -Destination (Join-Path $stage 'docs')
    Copy-Item -LiteralPath (Join-Path $repo 'docs\SECURITY.md') -Destination (Join-Path $stage 'docs')
    Copy-Item -LiteralPath (Join-Path $repo 'tethers-0.1\SPEC.md') -Destination (Join-Path $stage 'docs')
    Copy-Item -LiteralPath (Join-Path $portableRoot 'RELEASE.md') -Destination (Join-Path $stage 'docs')

    $hashLines = foreach ($file in Get-ChildItem -LiteralPath $stage -File -Recurse | Sort-Object FullName) {
        $relative = [System.IO.Path]::GetRelativePath($stage, $file.FullName).Replace('\', '/')
        "{0}  {1}" -f (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToUpperInvariant(), $relative
    }
    $hashLines | Set-Content -NoNewline (Join-Path $stage 'SHA256SUMS')

    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) { throw 'Python is required for deterministic release packaging' }
    & $python.Source (Join-Path $portableRoot 'scripts\deterministic_zip.py') $stage $Output
    if ($LASTEXITCODE -ne 0) { throw "deterministic ZIP creation failed with exit code $LASTEXITCODE" }
    $zipHash = (Get-FileHash -LiteralPath $Output -Algorithm SHA256).Hash.ToUpperInvariant()
    "$zipHash  $(Split-Path $Output -Leaf)" | Set-Content -NoNewline "$Output.sha256"
    Write-Output "Bundle: $Output"
    Write-Output "SHA256: $zipHash"
}
finally {
    if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
}
