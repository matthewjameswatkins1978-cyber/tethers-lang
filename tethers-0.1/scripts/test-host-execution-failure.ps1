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
    throw "Missing request fixture: $RequestPath"
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

# Unique temp trail path per run, cleaned in finally block.
$trailDir = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-exec-fail-$([System.Guid]::NewGuid())"
$trailPath = Join-Path $trailDir "trail.jsonl"
try {
    Push-Location $HostDir
    try {
        $output = & cargo run -- $EnginePath $RequestPath "allow" $trailPath "fail"
        if ($LASTEXITCODE -ne 0) {
            throw "Rust reference host exited with code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }

    $text = ($output -join "`n").Trim()
    if ($text -eq "") {
        throw "Rust reference host produced no JSON output."
    }

    $response = $text | ConvertFrom-Json -ErrorAction Stop

    # --- Assertions ---

    if ($response.status -ne "matched") {
        throw "Expected engine status 'matched' but got '$($response.status)'."
    }

    # Executor failure path.
    if ($response.execution_status -ne "failed") {
        throw "Expected execution_status 'failed' but got '$($response.execution_status)'."
    }

    $trail = $response.trail
    if ($null -eq $trail -or $trail.Count -eq 0) {
        throw "Response has no Trail."
    }

    # Exactly one action_started.
    $started = @($trail | Where-Object { $_.kind -eq "action_started" })
    if ($started.Count -ne 1) {
        throw "Expected exactly 1 action_started, found $($started.Count)."
    }

    # Exactly one action_failed.
    $failed = @($trail | Where-Object { $_.kind -eq "action_failed" })
    if ($failed.Count -ne 1) {
        throw "Expected exactly 1 action_failed, found $($failed.Count)."
    }

    # Zero action_completed entries.
    $completed = @($trail | Where-Object { $_.kind -eq "action_completed" })
    if ($completed.Count -ne 0) {
        throw "Expected 0 action_completed, found $($completed.Count)."
    }

    # Verify intent and outcome were recorded in the durable Trail.
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
    if ($outcome.status -ne "failed") {
        throw "Outcome status '$($outcome.status)' expected 'failed'."
    }
    if ($outcome.error_message -ne "executor failed as requested") {
        throw "Outcome error_message '$($outcome.error_message)' expected 'executor failed as requested'."
    }
    if ($null -ne $outcome.PSObject.Properties['result']) {
        throw "Failed outcome must not have a result field, got '$($outcome.PSObject.Properties['result'].Value)'."
    }

    # Verify failure entry content.
    if ($failed[0].message -ne "executor failed as requested") {
        throw "action_failed message expected 'executor failed as requested', got '$($failed[0].message)'."
    }
    if ($failed[0].phase -ne "execution") {
        throw "action_failed phase expected 'execution', got '$($failed[0].phase)'."
    }
    if ($failed[0].outcome -ne "failed") {
        throw "action_failed outcome expected 'failed', got '$($failed[0].outcome)'."
    }
    if ($failed[0].action_id -ne "action_1") {
        throw "action_failed action_id expected 'action_1', got '$($failed[0].action_id)'."
    }

    # Plan remains atomic.
    if ($null -eq $response.plan.actions -or $response.plan.actions.Count -eq 0) {
        throw "Plan actions array is missing or empty."
    }

    Write-Output "PASS test-host-execution-failure"
    Write-Output "Engine status: $($response.status)"
    Write-Output "Execution status: $($response.execution_status)"
    Write-Output "Intent execution_id: $($intent.execution_id)"
    Write-Output "Durable records: $($trailRecords.Count)"
    Write-Output "action_started: $($started.Count)"
    Write-Output "action_failed: $($failed.Count)"
    Write-Output "action_completed: $($completed.Count)"
    Write-Output "Failure message: $($failed[0].message)"
}
finally {
    Remove-Item -LiteralPath $trailPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $trailDir -ErrorAction SilentlyContinue
}