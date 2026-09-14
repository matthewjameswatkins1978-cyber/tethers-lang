[CmdletBinding()]
param(
    [string]$OcamlSwitchPath,
    [switch]$ReleaseMode
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$reportPath = Join-Path $repositoryRoot 'verification/tethers-verification.json'
$results = [System.Collections.Generic.List[object]]::new()
$script:firstFailure = $null

function Invoke-VerificationStep {
    param(
        [string]$Name,
        [string]$Program,
        [string[]]$Arguments,
        [string]$WorkingDirectory = $repositoryRoot,
        [string]$Category = 'required'
    )
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $lines = @()
    Push-Location -LiteralPath $WorkingDirectory
    try {
        $lines = @(& $Program @Arguments 2>&1 | ForEach-Object { "$($_)" })
        $exitCode = $LASTEXITCODE
    } catch {
        $lines = @($_.Exception.Message)
        $exitCode = 1
    } finally {
        Pop-Location
        $watch.Stop()
    }
    $status = if ($exitCode -eq 0) { 'PASS' } else { 'FAIL' }
    $result = [pscustomobject]@{
        name = $Name
        category = $Category
        status = $status
        exit_code = $exitCode
        duration_ms = $watch.ElapsedMilliseconds
        first_failure = if ($exitCode -eq 0) { $null } else { $lines | Select-Object -First 1 }
    }
    [void]$results.Add($result)
    Write-Host ("{0}: {1}" -f $status, $Name)
    if ($exitCode -ne 0) {
        $detail = (@($lines | Select-Object -First 8) -join "`n").Trim()
        if (-not [string]::IsNullOrWhiteSpace($detail)) {
            Write-Host ("First failure detail: {0}" -f $detail)
            $result | Add-Member -NotePropertyName failure_detail -NotePropertyValue $detail
        }
        if ($null -eq $script:firstFailure) { $script:firstFailure = $result }
    }
    return $result
}

$head = ((& git -C $repositoryRoot rev-parse HEAD).Trim()).ToLowerInvariant()
$tree = ((& git -C $repositoryRoot rev-parse 'HEAD^{tree}').Trim()).ToLowerInvariant()
$statusLines = @(& git -C $repositoryRoot status --porcelain=v1 --untracked-files=all)
$dirty = $statusLines.Count -gt 0
$effectiveSwitch = $OcamlSwitchPath
if ([string]::IsNullOrWhiteSpace($effectiveSwitch)) { $effectiveSwitch = [string]$env:TETHERS_OCAML_SWITCH }
if ([string]::IsNullOrWhiteSpace($effectiveSwitch)) { $effectiveSwitch = Join-Path $repositoryRoot 'tethers-0.1/engine-ocaml' }
$nativeWindows = [System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT
$architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
$pwsh = if ($nativeWindows) { 'pwsh.exe' } else { 'pwsh' }

$report = [ordered]@{
    schema = 'tethers.verify/1'
    source_commit = $head
    source_tree = $tree
    dirty = $dirty
    platform = [ordered]@{
        os = if ($nativeWindows) { 'windows' } else { 'linux' }
        architecture = $architecture
        runtime = [System.Runtime.InteropServices.RuntimeInformation]::FrameworkDescription
    }
    toolchain = [ordered]@{ status = 'NOT_CHECKED' }
    engine = $null
    suites = $results
    first_meaningful_failure = $null
    verdict = 'FAIL'
    release_eligible = $false
}

$packet = Invoke-VerificationStep 'task packet checker' $pwsh @('-NoProfile', '-File', '.github/scripts/check-tethers-task-packet.ps1') -Category 'external'
$fmt = Invoke-VerificationStep 'Rust formatting' 'cargo' @('fmt', '--manifest-path', 'tethers-0.1/host-rust/Cargo.toml', '--all', '--', '--check')
$engineArgs = @('-NoProfile', '-File', 'scripts/prepare-current-engine.ps1')
if (-not [string]::IsNullOrWhiteSpace($effectiveSwitch)) { $engineArgs += @('-OcamlSwitchPath', $effectiveSwitch) }
$engine = Invoke-VerificationStep 'current OCaml engine build and provenance' $pwsh $engineArgs
$manifestPath = Join-Path $repositoryRoot 'verification/current-engine-provenance.json'
if ($engine.status -eq 'PASS' -and (Test-Path -LiteralPath $manifestPath)) {
    $report.engine = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $report.toolchain.status = 'PASS'
} else {
    $report.toolchain.status = 'FAIL'
}

if ($engine.status -eq 'PASS') {
    $ocamlArgs = @('exec', "--switch=$effectiveSwitch", '--', 'dune', 'runtest', '--force')
    [void](Invoke-VerificationStep 'OCaml tests' 'opam' $ocamlArgs -WorkingDirectory (Join-Path $repositoryRoot 'tethers-0.1/engine-ocaml'))
    [void](Invoke-VerificationStep 'Rust static checks' 'just' @('check'))
    [void](Invoke-VerificationStep 'warning ratchet' 'just' @('warning-ratchet'))
    $rustArgs = @('-NoProfile', '-File', 'scripts/run-rust-tests.ps1', '-OcamlSwitchPath', $effectiveSwitch)
    if ($ReleaseMode) { $rustArgs += '-Release' }
    [void](Invoke-VerificationStep 'Rust and cross-language tests' $pwsh $rustArgs)
    [void](Invoke-VerificationStep 'protocol fixture sanity' $pwsh @('-NoProfile', '-File', 'tethers-0.1/scripts/check-fixtures.ps1'))
    [void](Invoke-VerificationStep 'MCP transcript suite' $pwsh @('-NoProfile', '-File', 'tethers-0.1/scripts/test-mcp-transcripts.ps1'))
    [void](Invoke-VerificationStep 'compatibility corpus' $pwsh @('-NoProfile', '-File', 'scripts/check-compatibility-corpus.ps1'))
} else {
    [void]$results.Add([pscustomobject]@{ name = 'dependent OCaml/Rust cross-language suites'; category = 'required'; status = 'SKIPPED WITH REASON'; exit_code = $null; duration_ms = 0; first_failure = 'current engine prerequisite failed' })
}

$report.suites = @($results)
$report.outcome_counts = [ordered]@{
    PASS = @($results | Where-Object status -eq 'PASS').Count
    FAIL = @($results | Where-Object status -eq 'FAIL').Count
    'SKIPPED WITH REASON' = @($results | Where-Object status -eq 'SKIPPED WITH REASON').Count
    'NOT APPLICABLE' = @($results | Where-Object status -eq 'NOT APPLICABLE').Count
}
$report.first_meaningful_failure = if ($null -ne $script:firstFailure) { $script:firstFailure } else { $null }
$report.verdict = if (@($results | Where-Object status -eq 'FAIL').Count -eq 0) { 'PASS' } else { 'FAIL' }
$report.release_eligible = (-not $dirty -and $report.verdict -eq 'PASS')
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $reportPath) | Out-Null
$report | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $reportPath -Encoding utf8NoBOM
Write-Host "Machine verification report: verification/tethers-verification.json"
Write-Host ("Verdict: {0}" -f $report.verdict)
if ($report.verdict -ne 'PASS') { exit 1 }
