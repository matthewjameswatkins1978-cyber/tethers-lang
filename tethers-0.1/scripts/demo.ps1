Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$HostDir = Join-Path $Root "host-rust"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
$RequestPath = Join-Path $Root "protocol/request.json"

function Assert-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

Assert-Command "opam"
Assert-Command "cargo"

if (-not (Test-Path -LiteralPath $RequestPath -PathType Leaf)) {
    throw "Missing request fixture: protocol/request.json"
}

Push-Location $EngineDir
try {
    & opam exec -- dune build
    if ($LASTEXITCODE -ne 0) {
        throw "Dune build failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "Dune build completed but engine executable was not found at $EnginePath"
}

# Unique temp trail per run, cleaned in finally block.
$trailDir = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-demo-$([System.Guid]::NewGuid())"
$trailPath = Join-Path $trailDir "trail.jsonl"
try {
    Push-Location $HostDir
    try {
        $output = & cargo run -- $EnginePath $RequestPath "allow" $trailPath "success"
        if ($LASTEXITCODE -ne 0) {
            throw "Rust reference host demo failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }

    $text = ($output -join "`n").Trim()
    if ($text -eq "") {
        throw "Rust reference host produced no JSON output."
    }

    try {
        $response = $text | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Rust reference host produced invalid JSON: $($_.Exception.Message)"
    }

    if ($response.status -ne "matched") {
        throw "Demo completed but response status was '$($response.status)', expected 'matched'."
    }

    if ($response.execution_status -ne "completed") {
        throw "Demo completed but execution_status was '$($response.execution_status)', expected 'completed'."
    }

    # Verify intent and outcome in the durable Trail.
    $trailText = Get-Content -Raw -LiteralPath $trailPath
    $trailRecords = @($trailText -split "\r?\n" | Where-Object { $_.Trim() -ne "" })
    if ($trailRecords.Count -ne 2) {
        throw "Expected exactly 2 durable records (intent + outcome) in Trail file, found $($trailRecords.Count)."
    }

    $intent = $trailRecords[0] | ConvertFrom-Json -ErrorAction Stop
    if ($intent.execution_id -ne $response.evaluation_id) {
        throw "Intent execution_id '$($intent.execution_id)' does not match response evaluation_id '$($response.evaluation_id)'."
    }
    if ($intent.action_id -ne "action_1") {
        throw "Intent action_id '$($intent.action_id)' expected 'action_1'."
    }
    if ($intent.capability_name -ne "lantern.task.record") {
        throw "Intent capability_name '$($intent.capability_name)' expected 'lantern.task.record'."
    }

    $outcome = $trailRecords[1] | ConvertFrom-Json -ErrorAction Stop
    if ($outcome.execution_id -ne $response.evaluation_id) {
        throw "Outcome execution_id '$($outcome.execution_id)' does not match response evaluation_id '$($response.evaluation_id)'."
    }
    if ($outcome.action_id -ne "action_1") {
        throw "Outcome action_id '$($outcome.action_id)' expected 'action_1'."
    }
    if ($outcome.status -ne "succeeded") {
        throw "Outcome status '$($outcome.status)' expected 'succeeded'."
    }
    if ($outcome.result.status -ne "recorded") {
        throw "Outcome result.status '$($outcome.result.status)' expected 'recorded'."
    }

    $text
}
finally {
    Remove-Item -LiteralPath $trailPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $trailDir -ErrorAction SilentlyContinue
}
