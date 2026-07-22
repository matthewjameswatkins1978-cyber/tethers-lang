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
$trailDir = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-denial-$([System.Guid]::NewGuid())"
$trailPath = Join-Path $trailDir "trail.jsonl"
try {
    Push-Location $HostDir
    try {
        $output = & cargo run -- $EnginePath $RequestPath "deny" $trailPath "success"
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

    # Deny policy must result in denied execution status.
    if ($response.execution_status -ne "denied") {
        throw "Expected execution_status 'denied' but got '$($response.execution_status)'."
    }

    $trail = $response.trail
    if ($null -eq $trail -or $trail.Count -eq 0) {
        throw "Response has no Trail."
    }

    $kinds = $trail | ForEach-Object { $_.kind }

    # Denial: zero execution entries.
    if ("action_started" -in $kinds) {
        throw "Trail contains action_started but no Action should have executed."
    }
    if ("action_completed" -in $kinds) {
        throw "Trail contains action_completed but no Action should have executed."
    }
    if ("action_failed" -in $kinds) {
        throw "Trail contains action_failed but no Action should have started."
    }

    # An intent_failed entry must appear (the prepare_and_record Deny path).
    $intentFailed = @($trail | Where-Object { $_.kind -eq "intent_failed" })
    if ($intentFailed.Count -ne 1) {
        throw "Expected exactly 1 intent_failed Trail entry, found $($intentFailed.Count)."
    }
    if ($intentFailed[0].outcome -ne "failed") {
        throw "intent_failed outcome expected 'failed', got '$($intentFailed[0].outcome)'."
    }

    # Plan remains atomic.
    if ($null -eq $response.plan.actions -or $response.plan.actions.Count -eq 0) {
        throw "Plan actions array is missing or empty."
    }

    Write-Output "PASS test-host-denial"
    Write-Output "Engine status: $($response.status)"
    Write-Output "Execution status: $($response.execution_status)"
    Write-Output "Plan actions count: $($response.plan.actions.Count)"
}
finally {
    Remove-Item -LiteralPath $trailPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $trailDir -ErrorAction SilentlyContinue
}