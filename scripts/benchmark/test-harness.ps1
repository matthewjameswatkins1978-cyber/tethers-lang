# ====================================================================
# PERFORMANCE HARNESS
# NOT A NORMAL TEST
# FULL MODE MAY BE SLOW
#
# B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
#
# Bounded smoke harness for bench_mcp.exe: tiny sample counts only.
# Correctness proof, not a baseline run.
# ====================================================================
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$exe = 'D:\The Next Thing\Tethers Lang - Goose Integration\tethers-0.1\host-rust\target\release\bench_mcp.exe'
$cases = @('P1','P10')
foreach ($caseItem in $cases) {
    $iters = 5
    $warmup = 2
    $batch = 5
    $argList = @("-c", $caseItem, "-n", "$iters", "-w", "$warmup", "-b", "$batch")
    Write-Host "Testing $caseItem with args: $($argList -join ' ')"
    $out = [System.IO.Path]::GetTempFileName()
    $err = [System.IO.Path]::GetTempFileName()
    $proc = Start-Process -FilePath $exe -ArgumentList $argList -Wait -NoNewWindow -PassThru -RedirectStandardOutput $out -RedirectStandardError $err
    $e = Get-Content $err -Raw
    if ($e -match 'Case: (\w+)') { Write-Host "  bench saw $($Matches[1])" }
    else { Write-Host "  no match, err: $e" }
    $j = Get-Content $out -Raw | ConvertFrom-Json
    Write-Host "  median $($j.stats.median_us)us"
    Remove-Item $out,$err -Force
}
