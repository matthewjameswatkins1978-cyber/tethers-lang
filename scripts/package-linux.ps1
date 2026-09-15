[CmdletBinding()]
param(
    [string]$OcamlSwitchPath,
    [string]$Output = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT) {
    throw 'Linux packaging must run on native GNU/Linux; WSL is not a release artifact target.'
}
if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') {
    throw 'Linux packaging requires x86-64 (amd64).'
}

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$version = (Get-Content -Raw (Join-Path $repositoryRoot 'VERSION')).Trim()
$hostManifest = Join-Path $repositoryRoot 'tethers-0.1/host-rust/Cargo.toml'
$engineRoot = Join-Path $repositoryRoot 'tethers-0.1/engine-ocaml'
$releaseDirectory = Join-Path $repositoryRoot 'tethers-0.1/host-rust/target/release'
$engineDirectory = Join-Path $engineRoot '_build/default/bin'
$enginePath = $null
$dist = Join-Path $repositoryRoot 'dist'
if ([string]::IsNullOrWhiteSpace($Output)) {
    $Output = Join-Path $dist "tethers-$version-linux-x86_64.tar.gz"
}
$Output = [System.IO.Path]::GetFullPath($Output)
if (Test-Path -LiteralPath $Output) {
    throw "output already exists; refusing to replace: $Output"
}

if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath)) {
    $OcamlSwitchPath = [string]$env:TETHERS_OCAML_SWITCH
}
if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath)) {
    throw 'Supply -OcamlSwitchPath or set TETHERS_OCAML_SWITCH to the native Linux OCaml switch.'
}
if (-not [System.IO.Path]::IsPathFullyQualified($OcamlSwitchPath)) {
    throw "OCaml switch path must be absolute: $OcamlSwitchPath"
}
$OcamlSwitchPath = [System.IO.Path]::GetFullPath($OcamlSwitchPath)

New-Item -ItemType Directory -Force -Path $dist | Out-Null
$stageParent = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-linux-package-$([guid]::NewGuid())"
$stage = Join-Path $stageParent "tethers-$version-linux-x86_64"
New-Item -ItemType Directory -Force -Path $stage | Out-Null

try {
    & cargo build --release --locked --manifest-path $hostManifest --bin tethers
    if ($LASTEXITCODE -ne 0) { throw "native Linux host build failed with exit code $LASTEXITCODE" }

    Push-Location -LiteralPath $engineRoot
    try {
        & opam exec --switch="$OcamlSwitchPath" -- dune build '@all'
        if ($LASTEXITCODE -ne 0) { throw "native Linux OCaml engine build failed with exit code $LASTEXITCODE" }
    }
    finally { Pop-Location }

    foreach ($engineName in @('tethers_mcp_main.exe', 'tethers_mcp_main')) {
        $candidate = Join-Path $engineDirectory $engineName
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            $enginePath = $candidate
            break
        }
    }
    if ($null -eq $enginePath) {
        throw 'native Linux OCaml build produced no tethers_mcp_main(.exe) engine binary'
    }

    $hostPath = Join-Path $releaseDirectory 'tethers'
    foreach ($path in @($hostPath, $enginePath)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "required Linux package binary is missing: $path"
        }
    }
    & chmod +x $hostPath $enginePath
    if ($LASTEXITCODE -ne 0) { throw 'could not mark Linux package binaries executable' }

    Copy-Item -LiteralPath $hostPath -Destination (Join-Path $stage 'tethers')
    Copy-Item -LiteralPath $enginePath -Destination (Join-Path $stage 'tethers-engine')
    foreach ($relative in @('README.md', 'QUICKSTART.md', 'VERSION')) {
        Copy-Item -LiteralPath (Join-Path $repositoryRoot $relative) -Destination $stage
    }
    New-Item -ItemType Directory -Force -Path (Join-Path $stage 'docs') | Out-Null
    foreach ($relative in @('docs/SECURITY.md', 'docs/LINUX_PORTABILITY_AUDIT.md', 'tethers-0.1/SPEC.md')) {
        Copy-Item -LiteralPath (Join-Path $repositoryRoot $relative) -Destination (Join-Path $stage 'docs')
    }

    $sourceCommit = (& git -C $repositoryRoot rev-parse HEAD).Trim().ToLowerInvariant()
    $manifest = [ordered]@{
        schema = 'tethers.linux-package/1'
        version = $version
        platform = 'linux-x86_64'
        libc = 'glibc; minimum supported runtime is recorded by the build host and CI image'
        source_commit = $sourceCommit
        binaries = @(
            [ordered]@{ name = 'tethers'; sha256 = (Get-FileHash (Join-Path $stage 'tethers') -Algorithm SHA256).Hash.ToLowerInvariant() },
            [ordered]@{ name = 'tethers-engine'; sha256 = (Get-FileHash (Join-Path $stage 'tethers-engine') -Algorithm SHA256).Hash.ToLowerInvariant() }
        )
    }
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'manifest.json') -Encoding utf8NoBOM
    $sumLines = foreach ($file in Get-ChildItem -LiteralPath $stage -File -Recurse | Sort-Object FullName) {
        $relative = [System.IO.Path]::GetRelativePath($stage, $file.FullName).Replace('\', '/')
        "{0}  {1}" -f (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $relative
    }
    $sumLines | Set-Content -LiteralPath (Join-Path $stage 'SHA256SUMS') -Encoding utf8NoBOM

    $leaf = Split-Path -Leaf $stage
    & tar -czf $Output -C $stageParent $leaf
    if ($LASTEXITCODE -ne 0) { throw "tar archive creation failed with exit code $LASTEXITCODE" }
    $archiveHash = (Get-FileHash $Output -Algorithm SHA256).Hash.ToLowerInvariant()
    "${archiveHash}  $(Split-Path -Leaf $Output)" | Set-Content -LiteralPath "$Output.sha256" -Encoding ascii
    Write-Host "PASS Linux package: $Output"
    Write-Host "PASS archive SHA-256: $archiveHash"
}
finally {
    if (Test-Path -LiteralPath $stageParent) {
        Remove-Item -LiteralPath $stageParent -Recurse -Force
    }
}
