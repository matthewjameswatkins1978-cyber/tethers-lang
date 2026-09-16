[CmdletBinding()]
param(
    [string]$OutputRoot = "dist",
    [string]$OcamlSwitch = "",
    [string]$Version = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$hostManifest = Join-Path $repositoryRoot "tethers-0.1\host-rust\Cargo.toml"
$engineRoot = Join-Path $repositoryRoot "tethers-0.1\engine-ocaml"
$declaredVersion = (Get-Content -LiteralPath (Join-Path $repositoryRoot "VERSION") -Raw).Trim()
if ([string]::IsNullOrWhiteSpace($Version)) { $Version = $declaredVersion }
if ($Version -ne $declaredVersion) {
    throw "Requested release version '$Version' does not match VERSION '$declaredVersion'."
}
if ($Version -ne "0.7.1") { throw "This packer is reserved for the 0.7.1 Windows release, got '$Version'." }

$effectiveSwitch = $OcamlSwitch
if ([string]::IsNullOrWhiteSpace($effectiveSwitch)) { $effectiveSwitch = [string]$env:TETHERS_OCAML_SWITCH }
if ([string]::IsNullOrWhiteSpace($effectiveSwitch) -and (Test-Path -LiteralPath (Join-Path $engineRoot "_opam"))) {
    $effectiveSwitch = $engineRoot
}
if ([string]::IsNullOrWhiteSpace($effectiveSwitch)) {
    throw "An explicit OCaml switch is required. Pass -OcamlSwitch or set TETHERS_OCAML_SWITCH."
}
if (-not [System.IO.Path]::IsPathFullyQualified($effectiveSwitch)) {
    throw "OCaml switch path must be absolute: $effectiveSwitch"
}

Push-Location $repositoryRoot
try {
    $status = @(& git status --porcelain=v1 --untracked-files=all)
    if ($status.Count -gt 0) { throw "Release packaging requires a clean Git worktree." }
    $sourceCommit = (& git rev-parse HEAD).Trim().ToLowerInvariant()
    $sourceTree = (& git rev-parse "HEAD^{tree}").Trim().ToLowerInvariant()

    & cargo build --release --locked --manifest-path $hostManifest --bin tethers
    if ($LASTEXITCODE -ne 0) { throw "release Rust build failed" }

    Push-Location $engineRoot
    try {
        & opam exec --switch=$effectiveSwitch -- dune build '@all'
        if ($LASTEXITCODE -ne 0) { throw "OCaml engine build failed" }
    }
    finally { Pop-Location }

    $dist = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $OutputRoot))
    $stageRoot = Join-Path $dist ".tethers-$Version-windows-x64-stage"
    $package = Join-Path $stageRoot "Tethers-$Version-windows-x64"
    $archive = Join-Path $dist "Tethers-$Version-windows-x64.zip"
    $manifestPath = Join-Path $dist "Tethers-$Version-windows-x64-manifest.json"
    $sumsPath = Join-Path $dist "SHA256SUMS"

    foreach ($path in @($archive, $manifestPath, $sumsPath, $stageRoot)) {
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Recurse -Force }
    }
    New-Item -ItemType Directory -Force -Path $package | Out-Null

    Copy-Item -LiteralPath (Join-Path $repositoryRoot "tethers-0.1\host-rust\target\release\tethers.exe") -Destination (Join-Path $package "tethers.exe")
    Copy-Item -LiteralPath (Join-Path $engineRoot "_build\default\bin\tethers_mcp_main.exe") -Destination (Join-Path $package "tethers-engine.exe")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "VERSION") -Destination (Join-Path $package "VERSION")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "README.md") -Destination (Join-Path $package "README.md")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "QUICKSTART.md") -Destination (Join-Path $package "QUICKSTART.md")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs\AI_INTEGRATION.md") -Destination (Join-Path $package "AI-INTEGRATION.md")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs\SECURITY.md") -Destination (Join-Path $package "SECURITY.md")
    Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs\TETHERS_0_7_1_RELEASE.md") -Destination (Join-Path $package "TETHERS_0_7_1_RELEASE.md")
    if (Test-Path -LiteralPath (Join-Path $repositoryRoot "examples")) { Copy-Item -LiteralPath (Join-Path $repositoryRoot "examples") -Destination $package -Recurse }
    if (Test-Path -LiteralPath (Join-Path $repositoryRoot "policies")) { Copy-Item -LiteralPath (Join-Path $repositoryRoot "policies") -Destination $package -Recurse }

    $hashLines = foreach ($file in Get-ChildItem -LiteralPath $package -File -Recurse | Sort-Object FullName) {
        $relative = [System.IO.Path]::GetRelativePath($package, $file.FullName).Replace('\', '/')
        "{0}  {1}" -f (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $relative
    }
    Set-Content -LiteralPath (Join-Path $package "SHA256SUMS") -Value $hashLines -Encoding ascii

    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) { throw "Python is required for deterministic release packaging." }
    & $python.Source (Join-Path $repositoryRoot "tethers-0.1\portable-rust\scripts\deterministic_zip.py") $stageRoot $archive
    if ($LASTEXITCODE -ne 0) { throw "deterministic ZIP creation failed" }

    $archiveHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    $manifest = [ordered]@{
        schema = "tethers.release/1"
        product_version = $Version
        source_commit = $sourceCommit
        source_tree = $sourceTree
        platform = "windows-x64"
        archive = [ordered]@{ name = [System.IO.Path]::GetFileName($archive); sha256 = $archiveHash }
        executables = @(
            [ordered]@{ name = "tethers.exe"; sha256 = (Get-FileHash -LiteralPath (Join-Path $package "tethers.exe") -Algorithm SHA256).Hash.ToLowerInvariant() },
            [ordered]@{ name = "tethers-engine.exe"; sha256 = (Get-FileHash -LiteralPath (Join-Path $package "tethers-engine.exe") -Algorithm SHA256).Hash.ToLowerInvariant() }
        )
    }
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
    $manifestHash = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
    @(
        "$archiveHash  $(Split-Path $archive -Leaf)"
        "$manifestHash  $(Split-Path $manifestPath -Leaf)"
    ) | Set-Content -LiteralPath $sumsPath -Encoding ascii

    Write-Output "PACKAGE=$package"
    Write-Output "ARCHIVE=$archive"
    Write-Output "ARCHIVE_SHA256=$archiveHash"
    Write-Output "MANIFEST=$manifestPath"
    Write-Output "SHA256SUMS=$sumsPath"
}
finally { Pop-Location }
