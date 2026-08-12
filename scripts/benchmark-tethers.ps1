#Requires -Version 7.0
<#
====================================================================
PERFORMANCE HARNESS
NOT A NORMAL TEST
FULL MODE MAY BE SLOW

B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
====================================================================
.SYNOPSIS
    B0 Performance Baseline Harness for Tethers

.DESCRIPTION
    Reproducible performance baseline for Tethers at BASE SHA.
    Measures four layers separately:
      B0-A  Core pipeline (in-process OCaml)
      B0-B  Warm MCP planning (Rust EngineSession)
      B0-C  Warm full production execution (HostExecutionService retained)
      B0-D  Cold full production execution (per-sample fresh process)

    Default invocation is a bounded QUICK smoke (seconds, correctness
    proof only).  The expensive historical matrix requires -Full.
    Quick numbers are NOT baseline performance numbers.

.PARAMETER Layer
    Which layers to run: all, core, mcp, production, cold

.PARAMETER Quick
    Bounded smoke mode with drastically fewer samples (default behaviour).

.PARAMETER Full
    Run the full historical B0 matrix (P10/P25/P50 production, multiple
    cold samples).  Requires explicit opt-in; may take several minutes.

.PARAMETER OverwriteBaseline
    Required to write output into docs/performance/b0-original-baseline.
    The completed B0 aggregate (raw.json / raw.csv) is immutable evidence;
    a normal run writes to a timestamped directory instead.

.PARAMETER OutputPath
    Directory for raw.json / raw.csv.  Default: timestamped run directory.

.PARAMETER Passes
    Number of full passes (default 1)

.EXAMPLE
    pwsh ./scripts/benchmark-tethers.ps1
    pwsh ./scripts/benchmark-tethers.ps1 -Layer core -Quick
    pwsh ./scripts/benchmark-tethers.ps1 -Layer all -Full
#>
[CmdletBinding()]
param(
    [ValidateSet('all','core','mcp','production','cold')]
    [string]$Layer = 'all',
    [switch]$Quick,
    [switch]$Full,
    [switch]$OverwriteBaseline,
    [string]$OutputPath = '',
    [int]$Passes = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Helpers ───────────────────────────────────────────────────────────
function Write-Info($msg) { Write-Host $msg -ForegroundColor Cyan }
function Write-Warn($msg) { Write-Host $msg -ForegroundColor Yellow }
function Fail($msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

# ── Resolve repo root ─────────────────────────────────────────────────
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$EngineOcamlDir = Join-Path $RepoRoot 'tethers-0.1\engine-ocaml'
$HostRustDir = Join-Path $RepoRoot 'tethers-0.1\host-rust'
$BaselineOut = Join-Path $RepoRoot 'docs\performance\b0-original-baseline'

if ($Quick -and $Full) {
    Fail "Cannot combine -Quick and -Full"
}

# Default: timestamped run directory.  The B0 baseline aggregate is only
# replaced with an explicit -OverwriteBaseline.
$RunsRoot = Join-Path $RepoRoot 'docs\performance\runs'
if ($OutputPath -eq '') {
    if ($OverwriteBaseline) {
        $OutputPath = $BaselineOut
    } else {
        $OutputPath = Join-Path $RunsRoot ("run-{0:yyyyMMdd-HHmmss}" -f (Get-Date))
    }
}
# Guard: never let a non-overwrite run target the immutable baseline dir.
$outputFull = [System.IO.Path]::GetFullPath($OutputPath)
$baselineFull = [System.IO.Path]::GetFullPath($BaselineOut)
if ($outputFull.Equals($baselineFull, [System.StringComparison]::OrdinalIgnoreCase) -and -not $OverwriteBaseline) {
    Fail "OutputPath targets the immutable B0 baseline directory. Use -OverwriteBaseline to replace it explicitly."
}
$RawJsonPath = Join-Path $OutputPath 'raw.json'
$RawCsvPath  = Join-Path $OutputPath 'raw.csv'

function Get-EnvironmentInfo {
    $branch = (git -C $RepoRoot branch --show-current 2>$null)
    if (-not $branch) { $branch = (git -C $RepoRoot rev-parse --abbrev-ref HEAD 2>$null) }
    $head = (git -C $RepoRoot rev-parse HEAD 2>$null)
    $baseSha = '1ce6b10f1de3cd10fef619483df444f83899c870'
    $os = (Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue)
    $cpu = (Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1)
    $memGB = if ($os) { [math]::Round($os.TotalVisibleMemorySize / 1MB, 1) } else { $null }
    $powerPlan = try { (powercfg /getactivescheme 2>$null) } catch { $null }
    $rustVer = try { (rustc --version 2>$null) } catch { $null }
    $cargoVer = try { (cargo --version 2>$null) } catch { $null }
    $ocamlVer = try {
        (& opam env "--switch=D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" --set-switch) -split '\r?\n' | ForEach-Object { Invoke-Expression $_ } | Out-Null
        (ocaml --version 2>$null)
    } catch { $null }
    $duneVer = try { (dune --version 2>$null) } catch { $null }
    return [ordered]@{
        baseline_sha = $baseSha
        current_head = $head
        branch = $branch
        date = (Get-Date -Format 'o')
        os = if ($os) { "$($os.Caption) $($os.Version) $($os.OSArchitecture)" } else { (Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version) | Out-String }
        cpu_model = if ($cpu) { $cpu.Name.Trim() } else { $null }
        logical_processors = if ($cpu) { $cpu.NumberOfLogicalProcessors } else { (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors }
        physical_memory_gb = $memGB
        rust_version = $rustVer
        cargo_version = $cargoVer
        ocaml_version = $ocamlVer
        dune_version = $duneVer
        build_profile = 'release'
        power_plan = ($powerPlan -join ' ').Trim()
        plugged_in = $null
    }
}

function Invoke-Bench {
    param([string]$Exe, [string[]]$BenchArgs, [string]$Label)
    Write-Info "  $Label : $Exe $($BenchArgs -join ' ')"
    # Use Start-Process with ArgumentList array to correctly handle args with spaces and short flags
    $outFile = [System.IO.Path]::GetTempFileName()
    $errFile = [System.IO.Path]::GetTempFileName()
    $proc = Start-Process -FilePath $Exe -ArgumentList $BenchArgs -WorkingDirectory $RepoRoot -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outFile -RedirectStandardError $errFile
    $stdout = Get-Content -LiteralPath $outFile -Raw -ErrorAction SilentlyContinue
    $stderr = Get-Content -LiteralPath $errFile -Raw -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $outFile -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $errFile -Force -ErrorAction SilentlyContinue
    if ($stderr) { Write-Host $stderr -ForegroundColor DarkGray }
    if ($proc.ExitCode -ne 0) {
        Fail "$Label failed with exit $($proc.ExitCode)`nSTDOUT: $stdout`nSTDERR: $stderr"
    }
    # Extract JSON object from stdout (last JSON block)
    $jsonText = $stdout.Trim()
    # OCaml bench prints human table then "JSON output:\n{...}" - extract from first { to last }
    $firstBrace = $jsonText.IndexOf('{')
    $lastBrace = $jsonText.LastIndexOf('}')
    if ($firstBrace -ge 0 -and $lastBrace -ge 0) {
        $jsonText = $jsonText.Substring($firstBrace, $lastBrace - $firstBrace + 1)
    }
    try { $obj = $jsonText | ConvertFrom-Json -Depth 10 } catch { Fail "Failed to parse JSON for $Label : $_`nText: $jsonText" }
    return $obj
}

# ── Ensure release binaries ─────────────────────────────────────────
Write-Info "Checking release binaries..."

$OcamlBench = Join-Path $EngineOcamlDir '_build\default\bin\tethers_benchmark_core.exe'
$BenchMcp   = Join-Path $HostRustDir 'target\release\bench_mcp.exe'
$BenchProd  = Join-Path $HostRustDir 'target\release\bench_prod.exe'
$BenchCold  = Join-Path $HostRustDir 'target\release\bench_cold.exe'

if (-not (Test-Path $OcamlBench)) {
    Write-Info "Building OCaml release..."
    (& opam env "--switch=D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" --set-switch) -split '\r?\n' | ForEach-Object { Invoke-Expression $_ }
    Push-Location $EngineOcamlDir
    dune build --profile release bin/tethers_benchmark_core.exe --display=short
    if ($LASTEXITCODE -ne 0) { Fail "OCaml build failed" }
    Pop-Location
}
if (-not (Test-Path $BenchMcp) -or -not (Test-Path $BenchProd) -or -not (Test-Path $BenchCold)) {
    Write-Info "Building Rust release benches..."
    Push-Location $HostRustDir
    cargo build --release --locked --bin bench_mcp --bin bench_prod --bin bench_cold
    if ($LASTEXITCODE -ne 0) { Fail "Rust bench build failed" }
    Pop-Location
}

# Ensure output dir
New-Item -ItemType Directory -Force -Path $OutputPath | Out-Null

$envInfo = Get-EnvironmentInfo
Write-Info "Environment: $($envInfo.cpu_model) / $($envInfo.logical_processors) cores / $($envInfo.rust_version)"

# ── Define cases per layer ──────────────────────────────────────────
# Quick smoke: bounded correctness proof (P1 + P10 core/MCP; P0/P1/P3
# production with 1 warmup + 1 measured; 1 cold sample).  Quick numbers
# are NOT baseline performance numbers.
# Full: explicit historical matrix (opt-in, -Full).
$isFullMode = [bool]$Full

if ($isFullMode) {
    Write-Warn "Full production benchmark includes P10/P25/P50 and may take several minutes."
    $AllCasesB0A = @('P0','P1','P3','P10','P25','P50','PC10','PA10')
    $AllCasesMcp = @('P0','P1','P3','P10','P25','P50','PC10','PA10')
    $AllCasesProd = @('P0','P1','P3','P10','P25')  # P50 optional
    $AllCasesCold = @('P0','P1','P10')
    $B0A_QuickArg = $false
    $McpIters = @{ P0=500; P1=500; P3=500; P10=500; P25=200; P50=100; PC10=500; PA10=500 }
    $ProdIters = @{ P0=30; P1=30; P3=20; P10=15; P25=10; P50=5 }
    $ProdWarmup = 10
    $ColdIters = 8
} else {
    Write-Info "QUICK smoke mode: correctness proof only, not baseline performance."
    $AllCasesB0A = @('P1','P10')
    $AllCasesMcp = @('P1','P10')
    $AllCasesProd = @('P0','P1','P3')
    $AllCasesCold = @('P0')
    $B0A_QuickArg = $true
    $McpIters = @{ P1=5; P10=5 }
    # 1 warmup + 1 measured per production case: P3 -> exactly 6 tools/call.
    $ProdIters = @{ P0=1; P1=1; P3=1 }
    $ProdWarmup = 1
    $ColdIters = 1
}

$AllResults = [ordered]@{
    meta = $envInfo
    quick = -not $isFullMode
    passes = $Passes
    layers = @{}
}

# ── B0-A Core pipeline ──────────────────────────────────────────────
if ($Layer -in @('all','core')) {
    Write-Info "`n=== B0-A: Core Pipeline (in-process OCaml) ==="
    $rawA = @()
    $b0aArgs = @()
    if ($B0A_QuickArg) { $b0aArgs = @('--quick') }
    for ($pass = 1; $pass -le $Passes; $pass++) {
        Write-Info "Pass $pass/$Passes"
        $obj = Invoke-Bench -Exe $OcamlBench -BenchArgs $b0aArgs -Label "B0-A pass $pass"
        $rawA += $obj
    }
    # For raw.json, keep the last pass's detailed cases (or aggregate)
    $AllResults.layers['B0-A'] = $rawA[-1]
    # Save per-pass for drift check
    $AllResults.layers['B0-A_passes'] = $rawA
    Write-Info "B0-A complete: $($rawA[-1].cases.Count) cases"
    foreach ($c in $rawA[-1].cases) {
        Write-Host ("  {0,-30} median={1,8:F1}us p95={2,8:F1}us ops/sec={3,8:F0}" -f $c.case, $c.stats.median_us, $c.stats.p95_us, $c.stats.ops_per_sec)
    }
}

# ── B0-B Warm MCP planning ──────────────────────────────────────────
if ($Layer -in @('all','mcp')) {
    Write-Info "`n=== B0-B: Warm MCP Planning (Rust EngineSession) ==="
    $bResults = @{}
    foreach ($caseItem in $AllCasesMcp) {
        $iters = $McpIters[$caseItem]
        if ($null -eq $iters) { $iters = 200 }
        $warmup = [math]::Min([math]::Max(1, [int]($iters * 0.1)), $iters)
        $batch = [math]::Min(10, [math]::Max(1, $iters))
        $obj = Invoke-Bench -Exe $BenchMcp -BenchArgs @("-c", $caseItem, "-n", "$iters", "-w", "$warmup", "-b", "$batch") -Label "B0-B $caseItem"
        $bResults[$caseItem] = $obj
        Write-Host ("  {0,-8} median={1,8:F1}us p95={2,8:F1}us n={3}" -f $caseItem, $obj.stats.median_us, $obj.stats.p95_us, $obj.stats.sample_count)
    }
    $AllResults.layers['B0-B'] = $bResults
}

# ── B0-C Warm production ────────────────────────────────────────────
if ($Layer -in @('all','production')) {
    Write-Info "`n=== B0-C: Warm Full Production (retained engine+provider) ==="
    $bResults = @{}
    foreach ($caseItem in $AllCasesProd) {
        $iters = $ProdIters[$caseItem]
        if ($null -eq $iters) { $iters = 50 }
        $warmup = $ProdWarmup
        $batch = [math]::Min(10, [math]::Max(1, $iters))
        # Use smaller batch for slow production to avoid too long
        if ($iters -le 20) { $batch = 5 }
        $obj = Invoke-Bench -Exe $BenchProd -BenchArgs @("-c", $caseItem, "-n", "$iters", "-w", "$warmup", "-b", "$batch") -Label "B0-C $caseItem"
        $bResults[$caseItem] = $obj
        Write-Host ("  {0,-8} median={1,8:F1}us p95={2,8:F1}us n={3} provider_calls={4}" -f $caseItem, $obj.stats.median_us, $obj.stats.p95_us, $obj.stats.sample_count, $obj.provider_tools_call_observed)
        # P3 exact-call proof: 1 warmup + 1 measured evaluation -> exactly 6 tools/call
        if (-not $isFullMode -and $caseItem -eq 'P3') {
            $expectedCalls = 3 * ($warmup + $iters)
            if ($obj.provider_tools_call_observed -ne $expectedCalls) {
                Fail "P3 exact-call proof FAILED: expected $expectedCalls provider tools/call, observed $($obj.provider_tools_call_observed)"
            }
            Write-Host ("  P3 exact-call proof PASS: $($obj.provider_tools_call_observed) provider tools/call (expected $expectedCalls)") -ForegroundColor Green
        }
    }
    # Also try P50 if time permits (warn if fails) -- full mode only
    if ($isFullMode) {
        try {
            $obj = Invoke-Bench -Exe $BenchProd -BenchArgs @("-c", "P50", "-n", 20, "-w", 10, "-b", 5) -Label "B0-C P50"
            $bResults['P50'] = $obj
            Write-Host ("  {0,-8} median={1,8:F1}us p95={2,8:F1}us n={3}" -f "P50", $obj.stats.median_us, $obj.stats.p95_us, $obj.stats.sample_count)
        } catch { Write-Warn "B0-C P50 failed or timed out: $_" }
    }
    $AllResults.layers['B0-C'] = $bResults
}

# ── B0-D Cold production ────────────────────────────────────────────
if ($Layer -in @('all','cold')) {
    Write-Info "`n=== B0-D: Cold Full Production (fresh process per sample) ==="
    $bResults = @{}
    foreach ($caseItem in $AllCasesCold) {
        $iters = $ColdIters
        $obj = Invoke-Bench -Exe $BenchCold -BenchArgs @("-c", $caseItem, "-n", "$iters") -Label "B0-D $caseItem"
        $bResults[$caseItem] = $obj
        Write-Host ("  {0,-8} median={1,8:F1}us p95={2,8:F1}us n={3}" -f $caseItem, $obj.stats.median_us, $obj.stats.p95_us, $obj.stats.sample_count)
    }
    $AllResults.layers['B0-D'] = $bResults
}

# ── Save raw.json ───────────────────────────────────────────────────
$AllResults | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $RawJsonPath -Encoding utf8NoBOM
Write-Info "`nRaw JSON saved to $RawJsonPath"

# ── Save raw.csv (one row per batch/sample) ─────────────────────────
$csvRows = @()
foreach ($layerName in $AllResults.layers.Keys) {
    $layerData = $AllResults.layers[$layerName]
    if ($layerName -eq 'B0-A' -and $layerData.cases) {
        foreach ($c in $layerData.cases) {
            $idx = 0
            foreach ($v in $c.raw_us) { $idx++; $csvRows += [pscustomobject]@{ layer='B0-A'; case=$c.case; batch=$idx; us=$v } }
        }
    } elseif ($layerName -eq 'B0-A_passes') { continue }
    elseif ($layerData -is [hashtable] -or $layerData -is [System.Collections.Specialized.OrderedDictionary]) {
        foreach ($k in $layerData.Keys) {
            $entry = $layerData[$k]
            if ($entry.raw_us) {
                $idx=0; foreach ($v in $entry.raw_us) { $idx++; $csvRows += [pscustomobject]@{ layer=$layerName; case=$k; batch=$idx; us=$v } }
            }
        }
    }
}
if ($csvRows.Count -gt 0) {
    $csvRows | Export-Csv -LiteralPath $RawCsvPath -NoTypeInformation -Encoding UTF8
    Write-Info "Raw CSV saved to $RawCsvPath ($($csvRows.Count) rows)"
} else {
    Write-Warn "No CSV rows generated"
}

# ── Print summary table ─────────────────────────────────────────────
Write-Host "`nSummary (median us):" -ForegroundColor Green
Write-Host "CASE      CORE         WARM MCP     WARM PROD    COLD PROD"
Write-Host "------------------------------------------------------------"
$casesOrder = @('P0','P1','P3','P10','P25','P50','PC10','PA10')
foreach ($c in $casesOrder) {
    $a = $null; $b=$null; $cc=$null; $d=$null
    if ($AllResults.layers['B0-A'] -and $AllResults.layers['B0-A'].cases) {
        $found = $AllResults.layers['B0-A'].cases | Where-Object { $_.case -like "*$c*" } | Select-Object -First 1
        if ($found) { $a = "{0:F1}" -f $found.stats.median_us }
    }
    if ($AllResults.layers['B0-B'] -and $AllResults.layers['B0-B'][$c]) { $b = "{0:F1}" -f $AllResults.layers['B0-B'][$c].stats.median_us }
    if ($AllResults.layers['B0-C'] -and $AllResults.layers['B0-C'][$c]) { $cc = "{0:F1}" -f $AllResults.layers['B0-C'][$c].stats.median_us }
    if ($AllResults.layers['B0-D'] -and $AllResults.layers['B0-D'][$c]) { $d = "{0:F1}" -f $AllResults.layers['B0-D'][$c].stats.median_us }
    if ($a -or $b -or $cc -or $d) {
        Write-Host ("{0,-8} {1,10} {2,10} {3,10} {4,10}" -f $c, ($a ?? 'N/A'), ($b ?? 'N/A'), ($cc ?? 'N/A'), ($d ?? 'N/A'))
    }
}

$mode = if ($isFullMode) { 'Full' } else { 'Quick' }
Write-Host "`nDone. Mode=$mode Passes=$Passes Layer=$Layer" -ForegroundColor Green
exit 0
