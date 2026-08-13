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
    PF1 Part B/D: retained P10 growth measurement harness.

.DESCRIPTION
    Builds bench_retained (release, bench-timing feature), runs ONE retained
    production session over 12 individual P10 evaluations, captures the JSON,
    and writes docs/performance/pf1/retained-p10.json and retained-p10.csv.

.PARAMETER Evaluations
    Number of retained evaluations (default 12).

.PARAMETER Warmup
    not_matched warmup evaluations that create no replay state (default 3).

.PARAMETER OutDir
    Output directory (default docs/performance/pf1).
#>
[CmdletBinding()]
param(
    [int]$Evaluations = 12,
    [int]$Warmup = 3,
    [string]$OutDir = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$HostRustDir = Join-Path $RepoRoot 'tethers-0.1\host-rust'
if ($OutDir -eq '') { $OutDir = Join-Path $RepoRoot 'docs\performance\pf1' }
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$RawJsonPath = Join-Path $OutDir 'retained-p10.json'
$RawCsvPath  = Join-Path $OutDir 'retained-p10.csv'

function Fail($msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

# ── Build release bench_retained with bench-timing ───────────────────
$BenchExe = Join-Path $HostRustDir 'target\release\bench_retained.exe'
if (-not (Test-Path $BenchExe)) {
    Write-Host 'Building release bench_retained (bench-timing)...' -ForegroundColor Cyan
    Push-Location $HostRustDir
    try {
        cargo build --release --locked --features bench-timing --bin bench_retained
        if ($LASTEXITCODE -ne 0) { Fail "cargo build bench_retained failed ($LASTEXITCODE)" }
    } finally { Pop-Location }
}

# ── Run retained measurement ─────────────────────────────────────────
Write-Host "Running retained P10: evaluations=$Evaluations warmup=$Warmup" -ForegroundColor Cyan
$stdoutFile = [System.IO.Path]::GetTempFileName()
$errFile = [System.IO.Path]::GetTempFileName()
$proc = Start-Process -FilePath $BenchExe -ArgumentList @('-n', "$Evaluations", '-w', "$Warmup") -WorkingDirectory $HostRustDir -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutFile -RedirectStandardError $errFile
$stdout = Get-Content -LiteralPath $stdoutFile -Raw
$stderr = Get-Content -LiteralPath $errFile -Raw -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $stdoutFile -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $errFile -Force -ErrorAction SilentlyContinue
if ($proc.ExitCode -ne 0) {
    Fail "bench_retained failed with exit $($proc.ExitCode)`nSTDERR: $stderr"
}
$firstBrace = $stdout.IndexOf('{')
$lastBrace = $stdout.LastIndexOf('}')
if ($firstBrace -lt 0 -or $lastBrace -lt 0) { Fail 'no JSON in bench_retained output' }
$jsonText = $stdout.Substring($firstBrace, $lastBrace - $firstBrace + 1)
$obj = $jsonText | ConvertFrom-Json -Depth 30

# ── Write retained-p10.json ──────────────────────────────────────────
$obj | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $RawJsonPath -Encoding utf8NoBOM
Write-Host "retained-p10.json written to $RawJsonPath" -ForegroundColor Green

# ── Write retained-p10.csv (one row per evaluation) ──────────────────
$rows = @()
foreach ($e in $obj.eval) {
    $st = $e.stages
    $rpTotal = ([double]0) + [double]$st.replay_publish_intent.total_us + [double]$st.replay_publish_armed.total_us + [double]$st.replay_publish_terminal.total_us
    $rows += [pscustomobject]@{
        eval_number = $e.eval_number
        wall_ms = [math]::Round([double]$e.wall_us / 1000.0, 3)
        tools_call_delta = $e.provider_tools_call_delta
        tools_call_cumulative = $e.provider_tools_call_cumulative
        trail_lines = $e.trail_lines
        trail_bytes = $e.trail_bytes
        replay_files = $e.replay_files
        replay_bytes = $e.replay_bytes
        replay_claims = $e.replay_claims
        replay_locks = $e.replay_locks
        replay_chain_files = $e.replay_chain_files
        replay_execution_dirs = $e.replay_execution_dirs
        replay_directories = $e.replay_directories
        stage_core_mcp_us = [math]::Round([double]$st.core_mcp.total_us, 1)
        stage_shared_boundary_total_us = [math]::Round([double]$st.shared_boundary.total_us, 1)
        stage_replay_admit_total_us = [math]::Round([double]$st.replay_admit.total_us, 1)
        stage_replay_admit_mean_us = [math]::Round([double]$st.replay_admit.mean_us, 1)
        stage_replay_publish_total_us = [math]::Round($rpTotal, 1)
        stage_provider_call_total_us = [math]::Round([double]$st.provider_call.total_us, 1)
        stage_trail_total_us = [math]::Round([double]$st.trail_intent.total_us + [double]$st.trail_outcome.total_us, 1)
        stage_scope_policy_total_us = [math]::Round([double]$st.scope_policy.total_us, 1)
    }
}
$rows | Export-Csv -LiteralPath $RawCsvPath -NoTypeInformation -Encoding UTF8
Write-Host "retained-p10.csv written to $RawCsvPath ($($rows.Count) rows)" -ForegroundColor Green

# ── Summary ──────────────────────────────────────────────────────────
Write-Host "`nRetained P10 summary:" -ForegroundColor Green
Write-Host ("{0,5} {1,10} {2,12} {3,12} {4,8}" -f 'eval', 'wall_ms', 'replay_admit_us/action', 'claims', 'calls')
foreach ($r in $rows) {
    Write-Host ("{0,5} {1,10} {2,12} {3,12} {4,8}" -f $r.eval_number, $r.wall_ms, $r.stage_replay_admit_mean_us, $r.replay_claims, $r.tools_call_cumulative)
}
Write-Host "`nProvider call proof: all_exact=$($obj.provider_call_proof.all_exact) total=$($obj.provider_call_proof.total) expected_per_eval=$($obj.provider_call_proof.expected_per_eval)"
exit 0
