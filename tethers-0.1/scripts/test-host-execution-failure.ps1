Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$HostDir = Join-Path $Root "host-rust"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
$RequestPath = Join-Path $Root "protocol/request.json"

foreach ($command in "opam", "cargo") {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "Required command '$command' was not found on PATH."
    }
}

Push-Location $EngineDir
try {
    & opam exec -- dune build
    if ($LASTEXITCODE -ne 0) { throw "Dune build failed with exit code $LASTEXITCODE." }
}
finally { Pop-Location }

$trailDir = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-exec-fail-$([System.Guid]::NewGuid())"
$trailPath = Join-Path $trailDir "trail.jsonl"
try {
    Push-Location $HostDir
    try {
        # The failing executor must never be reached: structured scope is not
        # established for this demo binding.
        $output = & cargo run -- $EnginePath $RequestPath "allow" $trailPath "fail"
        if ($LASTEXITCODE -ne 0) { throw "Rust reference host exited with code $LASTEXITCODE." }
    }
    finally { Pop-Location }

    $text = ($output -join "`n").Trim()
    if ($text -eq "") { throw "Rust reference host produced no JSON output." }
    $response = $text | ConvertFrom-Json -ErrorAction Stop

    if ($response.status -ne "matched") {
        throw "Expected engine status 'matched' but got '$($response.status)'."
    }
    if ($response.execution_status -ne "denied") {
        throw "Unassessed structured scope must deny before the failing executor, got '$($response.execution_status)'."
    }

    $trail = @($response.trail)
    foreach ($kind in "action_started", "action_failed", "action_completed") {
        if (@($trail | Where-Object { $_.kind -eq $kind }).Count -ne 0) {
            throw "Unassessed scope must not contain '$kind'."
        }
    }
    if (@($trail | Where-Object { $_.kind -eq "intent_failed" }).Count -ne 1) {
        throw "Expected exactly one intent_failed entry after policy denial."
    }
    if ($null -ne $response.PSObject.Properties["result_anchor"]) {
        throw "Unattempted Action must not produce a result_anchor."
    }

    $durableRecords = @((Get-Content -Raw -LiteralPath $trailPath) -split "\r?\n" | Where-Object { $_.Trim() -ne "" })
    if ($durableRecords.Count -ne 0) {
        throw "Denied Action must write zero durable intent/outcome records, found $($durableRecords.Count)."
    }

    Write-Output "PASS test-host-execution-failure (scope gate prevents executor call)"
    Write-Output "Engine status: $($response.status)"
    Write-Output "Execution status: $($response.execution_status)"
}
finally {
    Remove-Item -LiteralPath $trailPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $trailDir -ErrorAction SilentlyContinue
}
