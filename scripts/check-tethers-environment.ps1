[CmdletBinding()]
param(
    [ValidateSet('rust-host', 'ocaml-core', 'cross-language')]
    [string]$Profile,
    [string]$OcamlSwitchPath,
    [string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$gitRoot = @(& git -C $repositoryRoot rev-parse --show-toplevel)[0].Trim()
$gitRoot = [System.IO.Path]::GetFullPath($gitRoot)
if ($LASTEXITCODE -ne 0 -or -not $gitRoot.Equals($repositoryRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'The environment probe must live under the intended Git worktree.'
}

function Invoke-Probe {
    param(
        [string]$Capability,
        [string]$WorkingDirectory,
        [string]$Program,
        [string[]]$Arguments
    )

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    Push-Location -LiteralPath $WorkingDirectory
    try {
        $lines = @(& $Program @Arguments 2>&1 | ForEach-Object { "$_" })
        $exitCode = $LASTEXITCODE
    } finally {
        Pop-Location
        $watch.Stop()
    }
    [pscustomobject]@{
        capability = $Capability
        command = @($Program) + $Arguments
        cwd = $WorkingDirectory
        exit_code = $exitCode
        duration_ms = $watch.ElapsedMilliseconds
        result = if ($exitCode -eq 0) { 'pass' } else { 'fail' }
        stderr_first_line = if ($exitCode -eq 0) { $null } else { $lines | Select-Object -First 1 }
    }
}

# Preserve the repository-owned workstation diagnostic. Its broader tool set is
# advisory here; profile probes below decide task capability readiness.
& pwsh.exe -NoProfile -File (Join-Path $repositoryRoot 'scripts/check-dev-tools.ps1') *> $null
$developerToolsExit = $LASTEXITCODE

$hostRust = Join-Path $repositoryRoot 'tethers-0.1/host-rust'
$engineOcaml = Join-Path $repositoryRoot 'tethers-0.1/engine-ocaml'
$probes = [System.Collections.Generic.List[object]]::new()

if ($Profile -in @('rust-host', 'cross-language')) {
    [void]$probes.Add((Invoke-Probe -Capability 'rust.check' -WorkingDirectory $hostRust -Program 'cargo' -Arguments @('+1.89.0', 'metadata', '--locked', '--offline', '--format-version', '1')))
    [void]$probes.Add((Invoke-Probe -Capability 'rust.fmt' -WorkingDirectory $hostRust -Program 'cargo' -Arguments @('+1.89.0', 'fmt', '--version')))
    [void]$probes.Add((Invoke-Probe -Capability 'rust.test' -WorkingDirectory $hostRust -Program 'cargo' -Arguments @('+1.89.0', 'test', '--no-run', '--locked')))
}

if ($Profile -in @('ocaml-core', 'cross-language')) {
    if ([string]::IsNullOrWhiteSpace($OcamlSwitchPath) -or -not [System.IO.Path]::IsPathFullyQualified($OcamlSwitchPath)) {
        throw 'OcamlSwitchPath must be an explicit absolute path for an OCaml profile.'
    }
    [void]$probes.Add((Invoke-Probe -Capability 'ocaml.build' -WorkingDirectory $engineOcaml -Program 'opam' -Arguments @('exec', "--switch=$OcamlSwitchPath", '--', 'dune', 'build')))
    [void]$probes.Add((Invoke-Probe -Capability 'ocaml.test' -WorkingDirectory $engineOcaml -Program 'opam' -Arguments @('exec', "--switch=$OcamlSwitchPath", '--', 'dune', 'runtest')))
}

[void]$probes.Add((Invoke-Probe -Capability 'packet.check' -WorkingDirectory $repositoryRoot -Program 'pwsh.exe' -Arguments @('-NoProfile', '-File', '.github/scripts/check-tethers-task-packet.ps1')))
[void]$probes.Add((Invoke-Probe -Capability 'task.runner' -WorkingDirectory $repositoryRoot -Program 'just' -Arguments @('--list')))

$missing = @($probes | Where-Object { $_.result -eq 'fail' } | ForEach-Object {
    [pscustomobject]@{
        capability = $_.capability
        command = $_.command
        exit_code = $_.exit_code
        stderr_first_line = $_.stderr_first_line
    }
})

$head = @(& git -C $repositoryRoot rev-parse HEAD)[0].Trim()
$originMain = @(& git -C $repositoryRoot rev-parse origin/main)[0].Trim()
$mergeBase = @(& git -C $repositoryRoot merge-base HEAD origin/main)[0].Trim()
$status = @(& git -C $repositoryRoot status --short --branch)
$result = [ordered]@{
    schema = 'tethers-host-environment-v1'
    profile = $Profile
    repository = [ordered]@{
        root = $repositoryRoot
        branch = @(& git -C $repositoryRoot branch --show-current)[0].Trim()
        head = $head
        origin_main = $originMain
        merge_base = $mergeBase
        status = $status
    }
    shell = [ordered]@{
        recipe_shell = 'pwsh.exe -NoLogo -NoProfile -Command'
        probe_shell = $PSVersionTable.PSEdition + '-' + $PSVersionTable.PSVersion
    }
    installation_allowed = $false
    network = [ordered]@{ cargo_offline = $true; installation = 'denied' }
    developer_tools_diagnostic_exit = $developerToolsExit
    probes = $probes
    missing = $missing
    degraded = @()
    denied = @('pkg.install', 'global.config.write', 'shell.switch')
}

$json = $result | ConvertTo-Json -Depth 8
if ($OutputPath) {
    $directory = Split-Path -Parent $OutputPath
    if ($directory) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }
    Set-Content -LiteralPath $OutputPath -Value $json -Encoding utf8NoBOM
} else {
    $json
}

if ($missing.Count -gt 0) { exit 1 }
