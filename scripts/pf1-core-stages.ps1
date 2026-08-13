#Requires -Version 7.0
# ====================================================================
# PERFORMANCE HARNESS
# NOT A NORMAL TEST
# FULL MODE MAY BE SLOW
#
# B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
# ====================================================================
<#
.SYNOPSIS
    PF1 Part E/F: Core stage profiler harness.

.DESCRIPTION
    Builds tethers_benchmark_core.exe, runs it with --profile-stages
    (parse / lower / validate / canonicalize / plan at sizes
    5,10,25,50,100,250,500 plus the shape probe at 100 and 250), captures the
    JSON, and writes docs/performance/pf1/core-stages.json and
    core-stages.csv.

.PARAMETER OutDir
    Output directory (default docs/performance/pf1).

.PARAMETER SkipRun
    Reuse an existing core-stages.json instead of re-running the benchmark.
#>
[CmdletBinding()]
param(
    [string]$OutDir = '',
    [string]$OcamlSwitchPath = 'D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml',
    [switch]$SkipRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$EngineOcamlDir = Join-Path $RepoRoot 'tethers-0.1\engine-ocaml'
if ($OutDir -eq '') { $OutDir = Join-Path $RepoRoot 'docs\performance\pf1' }
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$RawJsonPath = Join-Path $OutDir 'core-stages.json'
$RawCsvPath  = Join-Path $OutDir 'core-stages.csv'

function Fail($msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

$ExpectedPrefix = Join-Path $OcamlSwitchPath '_opam'
if (-not (Test-Path -LiteralPath $ExpectedPrefix -PathType Container)) {
    Fail "Supplied OCaml switch does not contain _opam: $OcamlSwitchPath"
}

# ── Build OCaml benchmark (release) ──────────────────────────────────
$OcamlBench = Join-Path $EngineOcamlDir '_build\default\bin\tethers_benchmark_core.exe'
if (-not (Test-Path $OcamlBench)) {
    Write-Host 'Building OCaml release benchmark...' -ForegroundColor Cyan
    Push-Location $EngineOcamlDir
    try {
        & opam env "--switch=$OcamlSwitchPath" --set-switch 2>$null | ForEach-Object { Invoke-Expression $_ }
        dune build --profile release bin/tethers_benchmark_core.exe --display=short
        if ($LASTEXITCODE -ne 0) { Fail "dune build failed ($LASTEXITCODE)" }
    } finally { Pop-Location }
}

# ── Run stage profile ────────────────────────────────────────────────
if ($SkipRun) {
    if (-not (Test-Path -LiteralPath $RawJsonPath)) {
        Fail "SkipRun requested but $RawJsonPath does not exist"
    }
    $obj = Get-Content -LiteralPath $RawJsonPath -Raw | ConvertFrom-Json -Depth 30
} else {
    Write-Host 'Running OCaml stage profiler (--profile-stages)...' -ForegroundColor Cyan
    $stdoutFile = [System.IO.Path]::GetTempFileName()
    $errFile = [System.IO.Path]::GetTempFileName()
    $proc = Start-Process -FilePath $OcamlBench -ArgumentList @('--profile-stages') -WorkingDirectory $EngineOcamlDir -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutFile -RedirectStandardError $errFile
    $stdout = Get-Content -LiteralPath $stdoutFile -Raw
    $stderr = Get-Content -LiteralPath $errFile -Raw -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $stdoutFile -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $errFile -Force -ErrorAction SilentlyContinue
    if ($proc.ExitCode -ne 0) {
        Fail "tethers_benchmark_core failed with exit $($proc.ExitCode)`nSTDERR: $stderr"
    }
    $firstBrace = $stdout.IndexOf('{')
    $lastBrace = $stdout.LastIndexOf('}')
    if ($firstBrace -lt 0 -or $lastBrace -lt 0) { Fail 'no JSON in benchmark output' }
    $jsonText = $stdout.Substring($firstBrace, $lastBrace - $firstBrace + 1)
    $obj = $jsonText | ConvertFrom-Json -Depth 30

    # ── Write core-stages.json ───────────────────────────────────────
    $obj | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $RawJsonPath -Encoding utf8NoBOM
    Write-Host "core-stages.json written to $RawJsonPath" -ForegroundColor Green
}

# ── Write core-stages.csv (one row per size x stage) ─────────────────
$rows = @()
$stageNames = @('parse','lower','validate','canonicalize','plan','whole_pipeline')
foreach ($s in $obj.stages) {
    foreach ($stage in $stageNames) {
        $stats = $s.$stage
        $rows += [pscustomobject]@{
            size = $s.size
            stage = $stage
            sample_count = $stats.sample_count
            median_us = $stats.median_us
            p95_us = $stats.p95_us
            mean_us = $stats.mean_us
            min_us = $stats.min_us
            max_us = $stats.max_us
            equivalence = "$($s.equivalence_staged)/$($s.equivalence_wire)"
        }
    }
}
$rows | Export-Csv -LiteralPath $RawCsvPath -NoTypeInformation -Encoding UTF8
Write-Host "core-stages.csv written to $RawCsvPath ($($rows.Count) rows)" -ForegroundColor Green

# ── Summary ──────────────────────────────────────────────────────────
Write-Host "`nCore stage curve (median us):" -ForegroundColor Green
Write-Host ("{0,6} {1,10} {2,10} {3,10} {4,12} {5,10} {6,12}" -f 'size','parse','lower','validate','canonicalize','plan','whole')
foreach ($s in $obj.stages) {
    Write-Host ("{0,6} {1,10:F1} {2,10:F1} {3,10:F1} {4,12:F1} {5,10:F1} {6,12:F1}" -f $s.size, $s.parse.median_us, $s.lower.median_us, $s.validate.median_us, $s.canonicalize.median_us, $s.plan.median_us, $s.whole_pipeline.median_us)
}
Write-Host "`nShape probe (canonicalize median us):" -ForegroundColor Green
foreach ($s in $obj.shape_probe) {
    Write-Host ("  size={0}: high={1:F1}us low={2:F1}us (ratio {3:F1}x)" -f $s.size, $s.high_symmetry_canonicalize.median_us, $s.low_symmetry_canonicalize.median_us, ([double]$s.high_symmetry_canonicalize.median_us / [double]$s.low_symmetry_canonicalize.median_us))
}
exit 0
