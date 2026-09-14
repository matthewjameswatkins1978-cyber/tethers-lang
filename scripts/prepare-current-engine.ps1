[CmdletBinding()]
param(
    [string]$OcamlSwitchPath,
    [switch]$ReleaseMode,
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$engineRoot = Join-Path $repositoryRoot 'tethers-0.1/engine-ocaml'
$manifestPath = Join-Path $repositoryRoot 'verification/current-engine-provenance.json'
$pwsh = if ([System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT) { 'pwsh.exe' } else { 'pwsh' }

function Emit-Failure {
    param([string]$Message)
    $result = [ordered]@{
        schema = 'tethers.engine/1'
        status = 'fail'
        classification = 'ENVIRONMENT / PREREQUISITE FAILURE'
        source_commit = $null
        message = $Message
    }
    if ($Json) { $result | ConvertTo-Json -Depth 8 } else {
        Write-Host 'VERIFICATION PREREQUISITE FAILED'
        Write-Host $Message
    }
    exit 1
}

try {
    $gitRoot = (& git -C $repositoryRoot rev-parse --show-toplevel 2>&1).Trim()
    $gitRoot = [System.IO.Path]::GetFullPath($gitRoot)
    if ($LASTEXITCODE -ne 0 -or -not $gitRoot.Equals($repositoryRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        Emit-Failure 'The engine preparation script is not running inside the intended Git worktree.'
    }

    $sourceCommit = (& git -C $repositoryRoot rev-parse HEAD 2>&1).Trim().ToLowerInvariant()
    $sourceTree = (& git -C $repositoryRoot rev-parse 'HEAD^{tree}' 2>&1).Trim().ToLowerInvariant()
    if ($LASTEXITCODE -ne 0) { Emit-Failure 'Could not identify the current source commit and tree.' }

    if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath)) {
        $OcamlSwitchPath = [string]$env:TETHERS_OCAML_SWITCH
    }
    if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath) -and (Test-Path (Join-Path $engineRoot '_opam'))) {
        $OcamlSwitchPath = $engineRoot
    }
    if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath)) {
        Emit-Failure 'No OCaml switch was supplied. Set TETHERS_OCAML_SWITCH or create the repository-local tethers-0.1/engine-ocaml switch.'
    }
    if (-not [System.IO.Path]::IsPathFullyQualified($OcamlSwitchPath)) {
        Emit-Failure "OCaml switch path must be absolute: $OcamlSwitchPath"
    }
    $OcamlSwitchPath = [System.IO.Path]::GetFullPath($OcamlSwitchPath)

    if ($ReleaseMode) {
        $status = @(& git -C $repositoryRoot status --porcelain=v1 --untracked-files=all)
        if ($status.Count -gt 0) { Emit-Failure 'Release verification requires a clean Git worktree.' }
    }

    $toolchainOutput = @(& $pwsh -NoProfile -File (Join-Path $repositoryRoot '.github/scripts/check-tethers-toolchains.ps1') -OcamlSwitchPath $OcamlSwitchPath 2>&1 | ForEach-Object { "$($_)" })
    if ($LASTEXITCODE -ne 0) {
        $first = ($toolchainOutput | Where-Object { $_ -match 'FAIL:' } | Select-Object -First 1)
        if ([string]::IsNullOrWhiteSpace($first)) { $first = ($toolchainOutput | Select-Object -Last 1) }
        Emit-Failure "Required toolchain is unavailable. $first. Cross-language host tests were not attempted."
    }

    Push-Location -LiteralPath $engineRoot
    try {
        $buildOutput = @(& opam exec --switch="$OcamlSwitchPath" -- dune build '@all' 2>&1 | ForEach-Object { "$($_)" })
    } finally {
        Pop-Location
    }
    if ($LASTEXITCODE -ne 0) {
        $firstBuildError = $buildOutput | Select-Object -First 1
        Emit-Failure "The current OCaml engine could not be built from source commit $sourceCommit. First error: $firstBuildError. Cross-language host tests were not attempted."
    }

    $engineName = if ([System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT) {
        'tethers_mcp_main.exe'
    } else {
        'tethers_mcp_main'
    }
    $enginePath = Join-Path $engineRoot (Join-Path '_build/default/bin' $engineName)
    if (-not (Test-Path -LiteralPath $enginePath -PathType Leaf)) {
        Emit-Failure "The current OCaml build completed without producing the required engine: tethers-0.1/engine-ocaml/_build/default/bin/$engineName"
    }
    $enginePath = [System.IO.Path]::GetFullPath($enginePath)
    $engineHash = (Get-FileHash -LiteralPath $enginePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $relativeEngine = [System.IO.Path]::GetRelativePath($repositoryRoot, $enginePath).Replace('\', '/')

    $manifest = [ordered]@{
        schema = 'tethers.engine/1'
        status = 'pass'
        source_commit = $sourceCommit
        source_tree = $sourceTree
        binary_relative_path = $relativeEngine
        binary_sha256 = $engineHash
        ocaml_version = ((& opam exec --switch="$OcamlSwitchPath" -- ocamlc -version).Trim())
        dune_version = ((& opam exec --switch="$OcamlSwitchPath" -- dune --version).Trim())
    }
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $manifestPath) | Out-Null
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM

    if ($Json) {
        $manifest | ConvertTo-Json -Depth 8
    } else {
        Write-Host "PASS current OCaml engine: $relativeEngine"
        Write-Host "PASS engine source commit: $sourceCommit"
        Write-Host "PASS engine source tree: $sourceTree"
        Write-Host "PASS engine SHA-256: $engineHash"
        Write-Host "PASS provenance manifest: verification/current-engine-provenance.json"
    }
} catch {
    Emit-Failure $_.Exception.Message
}
