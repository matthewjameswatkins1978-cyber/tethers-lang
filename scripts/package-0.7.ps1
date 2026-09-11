param(
    [string]$OutputRoot = "release",
    [string]$OcamlSwitch = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$hostManifest = Join-Path $repositoryRoot "tethers-0.1\host-rust\Cargo.toml"
$engineRoot = Join-Path $repositoryRoot "tethers-0.1\engine-ocaml"
$version = (Get-Content -LiteralPath (Join-Path $repositoryRoot "VERSION") -Raw).Trim()
if ($version -ne "0.7.0") { throw "VERSION must be 0.7.0, got '$version'." }

Push-Location $repositoryRoot
try {
    & cargo +1.97.1 build --manifest-path $hostManifest --release --bins
    if ($LASTEXITCODE -ne 0) { throw "release Rust build failed" }
    Push-Location $engineRoot
    try {
        if ([string]::IsNullOrWhiteSpace($OcamlSwitch)) {
            & opam exec -- dune build '@all'
        }
        else {
            & opam exec --switch=$OcamlSwitch -- dune build '@all'
        }
        if ($LASTEXITCODE -ne 0) { throw "OCaml engine build failed" }
    }
    finally { Pop-Location }

    $package = Join-Path $repositoryRoot (Join-Path $OutputRoot "tethers-0.7.0-windows-x64")
    if (Test-Path -LiteralPath $package) { Remove-Item -LiteralPath $package -Recurse -Force }
    New-Item -ItemType Directory -Path $package | Out-Null
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "tethers-0.1\host-rust\target\release\tethers.exe") -Destination (Join-Path $package "tethers.exe")
    Copy-Item -LiteralPath (Join-Path $engineRoot "_build\default\bin\tethers_mcp_main.exe") -Destination (Join-Path $package "tethers-engine.exe")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "VERSION") -Destination (Join-Path $package "VERSION")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "README.md") -Destination (Join-Path $package "README.md")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs\AI_INTEGRATION.md") -Destination (Join-Path $package "AI-INTEGRATION.md")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs\SECURITY.md") -Destination (Join-Path $package "SECURITY.md")
    if (Test-Path -LiteralPath (Join-Path $repositoryRoot "examples")) { Copy-Item -LiteralPath (Join-Path $repositoryRoot "examples") -Destination $package -Recurse }
    if (Test-Path -LiteralPath (Join-Path $repositoryRoot "policies")) { Copy-Item -LiteralPath (Join-Path $repositoryRoot "policies") -Destination $package -Recurse }
    $hashLines = @(
        "$( (Get-FileHash -LiteralPath (Join-Path $package "tethers.exe") -Algorithm SHA256).Hash.ToLowerInvariant() )  tethers.exe",
        "$( (Get-FileHash -LiteralPath (Join-Path $package "tethers-engine.exe") -Algorithm SHA256).Hash.ToLowerInvariant() )  tethers-engine.exe"
    )
    Set-Content -LiteralPath (Join-Path $package "SHA256SUMS") -Value $hashLines -Encoding ascii
    $archive = Join-Path $repositoryRoot (Join-Path $OutputRoot "Tethers-0.7.0-windows-x64.zip")
    if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force }
    Compress-Archive -LiteralPath $package -DestinationPath $archive
    $archiveHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    $manifest = [ordered]@{
        schema = "tethers.release/1"
        product_version = $version
        source_commit = (& git rev-parse HEAD).Trim()
        platform = "windows-x64"
        archive = [ordered]@{
            name = [System.IO.Path]::GetFileName($archive)
            sha256 = $archiveHash
        }
        executables = @(
            [ordered]@{
                name = "tethers.exe"
                sha256 = (Get-FileHash -LiteralPath (Join-Path $package "tethers.exe") -Algorithm SHA256).Hash.ToLowerInvariant()
            },
            [ordered]@{
                name = "tethers-engine.exe"
                sha256 = (Get-FileHash -LiteralPath (Join-Path $package "tethers-engine.exe") -Algorithm SHA256).Hash.ToLowerInvariant()
            }
        )
    }
    $manifestPath = Join-Path $repositoryRoot (Join-Path $OutputRoot "Tethers-0.7.0-windows-x64-manifest.json")
    $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
    Write-Output "PACKAGE=$package"
    Write-Output "ARCHIVE=$archive"
    Write-Output "ARCHIVE_SHA256=$archiveHash"
    Write-Output "MANIFEST=$manifestPath"
}
finally { Pop-Location }
