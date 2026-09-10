[CmdletBinding()]
param(
    [string]$Output = ""
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\")).Path
$hostManifest = Join-Path $repo "tethers-0.1\host-rust\Cargo.toml"
$author = Join-Path $repo "reference-plugs\tethers-agent-workspace\author"
$buildProvider = Join-Path $repo "tethers-0.1\host-rust\target\release\agent_workspace_provider.exe"
$buildHost = Join-Path $repo "tethers-0.1\host-rust\target\release\tethers.exe"

if ([string]::IsNullOrWhiteSpace($Output)) {
    $Output = Join-Path $repo "tethers-0.1\dist\tethers-agent-workspace-0.1.0.tetherplug"
}
$Output = [System.IO.Path]::GetFullPath($Output)
$outputParent = Split-Path -Parent $Output
New-Item -ItemType Directory -Force -Path $outputParent | Out-Null

$stage = Join-Path ([System.IO.Path]::GetTempPath()) ("tethers-agent-workspace-pack-" + [guid]::NewGuid().ToString("N"))
try {
    Write-Host "Building the reviewed provider..."
    & cargo build --manifest-path $hostManifest --release --locked --bin agent_workspace_provider --bin tethers
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
    if (-not (Test-Path -LiteralPath $buildProvider)) { throw "release provider executable was not produced" }
    if (-not (Test-Path -LiteralPath $buildHost)) { throw "release tethers executable was not produced" }

    New-Item -ItemType Directory -Force -Path (Join-Path $stage "provider"), (Join-Path $stage "manifests") | Out-Null
    Copy-Item -LiteralPath (Join-Path $author "plug.json") -Destination $stage
    Copy-Item -Path (Join-Path $author "manifests\*.json") -Destination (Join-Path $stage "manifests")
    Copy-Item -LiteralPath $buildProvider -Destination (Join-Path $stage "provider\agent_workspace_provider.exe")

    if (Test-Path -LiteralPath $Output) { throw "output already exists; refusing to replace: $Output" }
    Write-Host "Packing deterministic Plug: $Output"
    & $buildHost plug pack --source ([System.IO.Path]::GetFullPath($stage)) --output $Output
    if ($LASTEXITCODE -ne 0) { throw "tethers plug pack failed with exit code $LASTEXITCODE" }
    Write-Host "Package created: $Output"
}
finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
}
