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

$trailDir = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-demo-$([System.Guid]::NewGuid())"
$trailPath = Join-Path $trailDir "trail.jsonl"
try {
    Push-Location $HostDir
    try {
        $output = & cargo run -- __legacy $EnginePath $RequestPath "allow" $trailPath "success"
        if ($LASTEXITCODE -ne 0) { throw "Rust reference host demo failed with exit code $LASTEXITCODE." }
    }
    finally { Pop-Location }

    $text = ($output -join "`n").Trim()
    if ($text -eq "") { throw "Rust reference host produced no JSON output." }
    $response = $text | ConvertFrom-Json -ErrorAction Stop

    if ($response.status -ne "matched") {
        throw "Demo response status was '$($response.status)', expected 'matched'."
    }
    if ($response.execution_status -ne "denied") {
        throw "Unassessed structured scope must deny, got '$($response.execution_status)'."
    }
    if ($null -eq $response.plan.actions -or $response.plan.actions.Count -ne 1) {
        throw "Plan must remain atomic with one proposed Action."
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

    # J11 Packet 4: durable Trail must contain exactly one admission record.
    $durableRecords = @((Get-Content -Raw -LiteralPath $trailPath) -split "\r?\n" `
        | Where-Object { $_.Trim() -ne "" } `
        | ForEach-Object { $_ | ConvertFrom-Json })
    if ($durableRecords.Count -ne 1) {
        throw "Expected 1 durable admission record, found $($durableRecords.Count)."
    }
    $record = $durableRecords[0]
    if ($record.kind -ne "event_admitted") {
        throw "Durable record kind expected 'event_admitted', got '$($record.kind)'."
    }
    if ($record.source -ne "external") {
        throw "Durable record source expected 'external', got '$($record.source)'."
    }
    if ($record.generation -ne 0) {
        throw "Durable record generation expected 0, got $($record.generation)."
    }
    if ($record.processing -ne "continued") {
        throw "Durable record processing expected 'continued', got '$($record.processing)'."
    }
    if ($record.correlation_id -ne $record.event_id) {
        throw "Durable record correlation_id must equal event_id."
    }
    if ($null -ne $record.PSObject.Properties['causation_id']) {
        throw "causation_id must be omitted for external admission."
    }
    if ($null -ne $record.PSObject.Properties['reason_code']) {
        throw "reason_code must be omitted for accepted admission."
    }
    if ($null -ne $record.PSObject.Properties['maximum_generation']) {
        throw "maximum_generation must be omitted for accepted admission."
    }

    Write-Output "PASS demo (unassessed structured scope denies before execution)"
    $text
}
finally {
    Remove-Item -LiteralPath $trailPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $trailDir -ErrorAction SilentlyContinue
}
