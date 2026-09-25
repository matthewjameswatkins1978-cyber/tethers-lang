[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OcamlSwitch,
    [string]$OutputRoot = 'dist'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$version = (Get-Content -LiteralPath (Join-Path $repo 'VERSION') -Raw).Trim()
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "VERSION must be a plain product version, found '$version'." }
$expectedTag = "v$version"
$releaseNotes = "TETHERS_$($version.Replace('.', '_'))_RELEASE.md"
if (-not [System.IO.Path]::IsPathFullyQualified($OcamlSwitch)) {
    throw 'OcamlSwitch must be an absolute path.'
}
$switch = [System.IO.Path]::GetFullPath($OcamlSwitch)
$tag = (& git -C $repo describe --exact-match --tags HEAD 2>$null).Trim()
if ($LASTEXITCODE -ne 0 -or $tag -ne $expectedTag) {
    throw "Packaging requires HEAD to be the exact $expectedTag tag; got '$tag'."
}
if (@(& git -C $repo status --porcelain=v1 --untracked-files=all).Count -ne 0) {
    throw 'Packaging requires a clean worktree.'
}

$commit = (& git -C $repo rev-parse HEAD).Trim().ToLowerInvariant()
$tree = (& git -C $repo rev-parse 'HEAD^{tree}').Trim().ToLowerInvariant()
$target = Join-Path $repo $OutputRoot
$target = [System.IO.Path]::GetFullPath($target)
if (-not $target.StartsWith($repo + [System.IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw "OutputRoot must stay inside the repository: $target"
}
$archive = Join-Path $target "Tethers-$version-windows-x64.zip"
$manifestPath = Join-Path $target "Tethers-$version-windows-x64-manifest.json"
$sumsPath = Join-Path $target "SHA256SUMS-$version"
$stage = Join-Path $target ".tethers-$version-windows-x64-stage"
foreach ($path in @($archive, "$archive.sha256", $manifestPath, $sumsPath, $stage)) {
    if (Test-Path -LiteralPath $path) { throw "Refusing to replace existing release output: $path" }
}

$hostManifest = Join-Path $repo 'tethers-0.1/host-rust/Cargo.toml'
$engineRoot = Join-Path $repo 'tethers-0.1/engine-ocaml'
$dune = Join-Path $switch 'bin/dune.exe'
$ocamlc = Join-Path $switch 'bin/ocamlc.exe'
foreach ($tool in @($dune, $ocamlc)) {
    if (-not (Test-Path -LiteralPath $tool -PathType Leaf)) { throw "Missing pinned OCaml switch tool: $tool" }
}
& cargo build --release --locked --manifest-path $hostManifest --bin tethers
if ($LASTEXITCODE -ne 0) { throw "Rust host release build failed: $LASTEXITCODE" }
$priorPath = $env:PATH
$env:PATH = (Join-Path $switch 'bin') + [System.IO.Path]::PathSeparator + $priorPath
Push-Location $engineRoot
try {
    & $dune build --profile release '@all'
    if ($LASTEXITCODE -ne 0) { throw "OCaml engine release build failed: $LASTEXITCODE" }
}
finally {
    Pop-Location
    $env:PATH = $priorPath
}

$hostBinary = Join-Path $repo 'tethers-0.1/host-rust/target/release/tethers.exe'
$engineBinary = Join-Path $engineRoot '_build/default/bin/tethers_mcp_main.exe'
foreach ($path in @($hostBinary, $engineBinary)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing release binary: $path" }
}

New-Item -ItemType Directory -Path $stage -Force | Out-Null
try {
    $bin = Join-Path $stage 'bin'
    $examples = Join-Path $stage 'examples/external-consumer'
    New-Item -ItemType Directory -Path $bin, $examples, (Join-Path $stage 'docs') -Force | Out-Null
    Copy-Item -LiteralPath $hostBinary -Destination (Join-Path $bin 'tethers.exe')
    Copy-Item -LiteralPath $engineBinary -Destination (Join-Path $bin 'tethers-engine.exe')
    Copy-Item -LiteralPath (Join-Path $repo 'LICENSE-MIT') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'LICENSE-APACHE') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'README.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'QUICKSTART.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'docs/INTEGRATING_TETHERS.md') -Destination (Join-Path $stage 'docs')
    Copy-Item -LiteralPath (Join-Path $repo "docs/$releaseNotes") -Destination (Join-Path $stage 'docs')
    Copy-Item -LiteralPath (Join-Path $repo 'examples/external-consumer/README.md') -Destination $examples
    Copy-Item -LiteralPath (Join-Path $repo 'examples/external-consumer/consumer.py') -Destination $examples
    Copy-Item -LiteralPath (Join-Path $repo 'examples/external-consumer/smoke.py') -Destination $examples
    Copy-Item -LiteralPath (Join-Path $repo 'examples/external-consumer/fixture-ping.json') -Destination $examples

    $requiredExamples = @(
        'examples/external-consumer/README.md',
        'examples/external-consumer/consumer.py',
        'examples/external-consumer/fixture-ping.json',
        'examples/external-consumer/smoke.py'
    )
    foreach ($req in $requiredExamples) {
        $stagedPath = Join-Path $stage $req
        if (-not (Test-Path -LiteralPath $stagedPath -PathType Leaf)) {
            throw "Packaging verification failure: required documented example missing from bundle: $req"
        }
    }

    $hashes = [ordered]@{}
    foreach ($file in Get-ChildItem -LiteralPath $stage -File -Recurse | Sort-Object FullName) {
        $relative = [System.IO.Path]::GetRelativePath($stage, $file.FullName).Replace('\', '/')
        $hashes[$relative] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $hashLines = foreach ($relative in $hashes.Keys) { "$($hashes[$relative])  $relative" }
    $hashLines | Set-Content -LiteralPath (Join-Path $stage 'SHA256SUMS') -Encoding ascii

    $python = Get-Command python -ErrorAction Stop
    & $python.Source (Join-Path $repo 'tethers-0.1/portable-rust/scripts/deterministic_zip.py') $stage $archive
    if ($LASTEXITCODE -ne 0) { throw "Deterministic archive creation failed: $LASTEXITCODE" }

    $manifest = [ordered]@{
        schema = 'tethers.release/1'
        product_version = $version
        tag = $tag
        source_commit = $commit
        source_tree = $tree
        platform = 'windows-x64'
        rustc = (& rustc --version).Trim()
        ocaml = (& $ocamlc -version).Trim()
        dune = (& $dune --version).Trim()
        archive = [ordered]@{ name = [System.IO.Path]::GetFileName($archive); sha256 = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() }
        executables = @(
            [ordered]@{ name = 'tethers.exe'; sha256 = (Get-FileHash -LiteralPath (Join-Path $bin 'tethers.exe') -Algorithm SHA256).Hash.ToLowerInvariant() },
            [ordered]@{ name = 'tethers-engine.exe'; sha256 = (Get-FileHash -LiteralPath (Join-Path $bin 'tethers-engine.exe') -Algorithm SHA256).Hash.ToLowerInvariant() }
        )
    }
    $manifest | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
    $manifestHash = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
    @(
        "$($manifest.archive.sha256)  $(Split-Path $archive -Leaf)"
        "$manifestHash  $(Split-Path $manifestPath -Leaf)"
    ) | Set-Content -LiteralPath $sumsPath -Encoding ascii
    "$($manifest.archive.sha256)  $(Split-Path $archive -Leaf)" | Set-Content -LiteralPath "$archive.sha256" -Encoding ascii

    Write-Output "TAG=$tag"
    Write-Output "SOURCE_COMMIT=$commit"
    Write-Output "SOURCE_TREE=$tree"
    Write-Output "ARCHIVE=$archive"
    Write-Output "ARCHIVE_SHA256=$($manifest.archive.sha256)"
    Write-Output "MANIFEST=$manifestPath"
    Write-Output "MANIFEST_SHA256=$manifestHash"
}
finally {
    if (Test-Path -LiteralPath $stage) {
        $resolvedStage = [System.IO.Path]::GetFullPath($stage)
        if (-not $resolvedStage.StartsWith($target + [System.IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove unexpected stage path: $resolvedStage"
        }
        Remove-Item -LiteralPath $resolvedStage -Recurse -Force
    }
}
