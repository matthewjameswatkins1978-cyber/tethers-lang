# ====================================================================
# PERFORMANCE HARNESS
# NOT A NORMAL TEST
# FULL MODE MAY BE SLOW
#
# B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
# ====================================================================
[CmdletBinding()]
param(
    [switch]$OverwriteBaseline,
    [string]$OutputPath = ''
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$HostRustDir = "D:\The Next Thing\Tethers Lang - Goose Integration\tethers-0.1\host-rust"
$EngineOcamlDir = "D:\The Next Thing\Tethers Lang - Goose Integration\tethers-0.1\engine-ocaml"
$BaselineOut = "D:\The Next Thing\Tethers Lang - Goose Integration\docs\performance\b0-original-baseline"

# The completed B0 aggregate is immutable.  A normal run writes to a
# timestamped directory; only -OverwriteBaseline targets the baseline dir.
$RunsRoot = "D:\The Next Thing\Tethers Lang - Goose Integration\docs\performance\runs"
if ($OutputPath -eq '') {
    if ($OverwriteBaseline) {
        $OutDir = $BaselineOut
    } else {
        $OutDir = Join-Path $RunsRoot ("run-{0:yyyyMMdd-HHmmss}" -f (Get-Date))
    }
} else {
    $OutDir = $OutputPath
}
$outFull = [System.IO.Path]::GetFullPath($OutDir)
$baselineFull = [System.IO.Path]::GetFullPath($BaselineOut)
if ($outFull.Equals($baselineFull, [System.StringComparison]::OrdinalIgnoreCase) -and -not $OverwriteBaseline) {
    Write-Host "ERROR: this script targets the immutable B0 baseline directory. Use -OverwriteBaseline to replace it explicitly." -ForegroundColor Red
    exit 1
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "=== B0-A Core ===" -ForegroundColor Cyan
(& opam env "--switch=D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" --set-switch) -split '\r?\n' | ForEach-Object { Invoke-Expression $_ } | Out-Null
$ocamlOut = & "$EngineOcamlDir\_build\default\bin\tethers_benchmark_core.exe" 2>&1 | Out-String
$firstBrace = $ocamlOut.IndexOf('{')
$lastBrace = $ocamlOut.LastIndexOf('}')
$jsonText = $ocamlOut.Substring($firstBrace, $lastBrace - $firstBrace + 1)
$b0a = $jsonText | ConvertFrom-Json -Depth 10
$b0a | ConvertTo-Json -Depth 10 | Set-Content "$OutDir\b0a.json" -Encoding utf8NoBOM
Write-Host "B0-A saved: $($b0a.cases.Count) cases"

Write-Host "`n=== B0-B Warm MCP ===" -ForegroundColor Cyan
$casesB = @('P0','P1','P3','P10','P25','P50','PC10','PA10')
$itersB = @{ P0=500; P1=500; P3=500; P10=500; P25=200; P50=100; PC10=500; PA10=500 }
$resultsB = @{}
foreach ($c in $casesB) {
    $iters = $itersB[$c]
    $argList = @("-c", $c, "-n", "$iters", "-w", "50", "-b", "10")
    $out = [System.IO.Path]::GetTempFileName(); $err = [System.IO.Path]::GetTempFileName()
    Start-Process -FilePath "$HostRustDir\target\release\bench_mcp.exe" -ArgumentList $argList -Wait -NoNewWindow -RedirectStandardOutput $out -RedirectStandardError $err
    $j = Get-Content $out -Raw | ConvertFrom-Json -Depth 10
    $resultsB[$c] = $j
    Write-Host "  $c median $($j.stats.median_us)us"
    Remove-Item $out,$err -Force
}
$resultsB | ConvertTo-Json -Depth 10 | Set-Content "$OutDir\b0b.json" -Encoding utf8NoBOM
Write-Host "B0-B saved"

Write-Host "`n=== B0-C Warm Production ===" -ForegroundColor Cyan
$casesC = @('P0','P1','P3','P10','P25','P50')
$itersC = @{ P0=30; P1=30; P3=20; P10=15; P25=10; P50=5 }
$resultsC = @{}
foreach ($c in $casesC) {
    $iters = $itersC[$c]
    $argList = @("-c", $c, "-n", "$iters", "-w", "10", "-b", "5")
    $out = [System.IO.Path]::GetTempFileName(); $err = [System.IO.Path]::GetTempFileName()
    Write-Host "  Running B0-C $c with $iters iterations..."
    Start-Process -FilePath "$HostRustDir\target\release\bench_prod.exe" -ArgumentList $argList -Wait -NoNewWindow -RedirectStandardOutput $out -RedirectStandardError $err
    $j = Get-Content $out -Raw | ConvertFrom-Json -Depth 10
    $resultsC[$c] = $j
    Write-Host "  $c median $($j.stats.median_us)us (provider_tools_call=$($j.provider_tools_call_observed))"
    Remove-Item $out,$err -Force
}
$resultsC | ConvertTo-Json -Depth 10 | Set-Content "$OutDir\b0c.json" -Encoding utf8NoBOM
Write-Host "B0-C saved"

Write-Host "`n=== B0-D Cold Production ===" -ForegroundColor Cyan
$casesD = @('P0','P1','P10')
$resultsD = @{}
foreach ($c in $casesD) {
    $iters = 8
    $argList = @("-c", $c, "-n", "$iters")
    $out = [System.IO.Path]::GetTempFileName(); $err = [System.IO.Path]::GetTempFileName()
    Write-Host "  Running B0-D $c with $iters cold samples (slow)..."
    Start-Process -FilePath "$HostRustDir\target\release\bench_cold.exe" -ArgumentList $argList -Wait -NoNewWindow -RedirectStandardOutput $out -RedirectStandardError $err
    $j = Get-Content $out -Raw | ConvertFrom-Json -Depth 10
    $resultsD[$c] = $j
    Write-Host "  $c median $($j.stats.median_us)us"
    Remove-Item $out,$err -Force
}
$resultsD | ConvertTo-Json -Depth 10 | Set-Content "$OutDir\b0d.json" -Encoding utf8NoBOM
Write-Host "B0-D saved"

# Aggregate raw.json
$envInfo = @{
    baseline_sha = "1ce6b10f1de3cd10fef619483df444f83899c870"
    date = (Get-Date -Format 'o')
    cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name.Trim()
    logical_processors = (Get-CimInstance Win32_Processor | Select-Object -First 1).NumberOfLogicalProcessors
}
$agg = @{ meta = $envInfo; 'B0-A' = $b0a; 'B0-B' = $resultsB; 'B0-C' = $resultsC; 'B0-D' = $resultsD }
$agg | ConvertTo-Json -Depth 12 | Set-Content "$OutDir\raw.json" -Encoding utf8NoBOM
Write-Host "`nAll layers saved to $OutDir\raw.json" -ForegroundColor Green

# CSV
$rows = @()
foreach ($c in $b0a.cases) { $i=0; foreach ($v in $c.raw_us) { $i++; $rows += [pscustomobject]@{ layer='B0-A'; case=$c.case; idx=$i; us=$v } } }
foreach ($k in $resultsB.Keys) { $i=0; foreach ($v in $resultsB[$k].raw_us) { $i++; $rows += [pscustomobject]@{ layer='B0-B'; case=$k; idx=$i; us=$v } } }
foreach ($k in $resultsC.Keys) { $i=0; foreach ($v in $resultsC[$k].raw_us) { $i++; $rows += [pscustomobject]@{ layer='B0-C'; case=$k; idx=$i; us=$v } } }
foreach ($k in $resultsD.Keys) { $i=0; foreach ($v in $resultsD[$k].raw_us) { $i++; $rows += [pscustomobject]@{ layer='B0-D'; case=$k; idx=$i; us=$v } } }
$rows | Export-Csv "$OutDir\raw.csv" -NoTypeInformation -Encoding UTF8
Write-Host "CSV saved with $($rows.Count) rows" -ForegroundColor Green
