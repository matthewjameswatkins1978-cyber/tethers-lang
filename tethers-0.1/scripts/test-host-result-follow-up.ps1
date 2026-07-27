Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# J10 production follow-up smoke. Proves the serial follow-up
# coordinator at the real compiled binary boundary.  Two phases:
#
#   Phase 1 (with-follow-up) is proven by the J10 Rust unit tests
#   in tethers-0.1/host-rust/src/main.rs (tests::j10_*) — 20
#   tests covering the full coordinator, evaluation ID, follow-up
#   request mapping, FIFO, generation overflow, recursion guard,
#   queue-pop guard, and single-iteration drain.  Running the
#   full Phase 1 via this binary is blocked by the production
#   FileReplayAuthority, which requires a pre-provisioned replay
#   root (per the J09 ledger model).  Provision-replay is a
#   separate Windows-only administrative command and is not
#   exercised here; the unit-test path uses
#   TestReplayAuthority instead.
#
#   Phase 2 (no-follow-up) uses the demo request with a `deny`
#   policy posture so no Result Anchor is ever enqueued.  The
#   output must omit `follow_up_evaluations`, carry no Result
#   Anchor, and write zero durable Trail records.

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

# ---------------------------------------------------------------
# Phase 2: no-follow-up smoke.
# A denied posture must never enqueue a Result Anchor, so the host
# response must omit follow_up_evaluations and carry no provider call.
# Reuses the existing test-host-denial engine fixture.
# ---------------------------------------------------------------

$trailDirNoFollowUp = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-j10-nofollowup-$([System.Guid]::NewGuid())"
$trailPathNoFollowUp = Join-Path $trailDirNoFollowUp "trail.jsonl"
New-Item -ItemType Directory -Path $trailDirNoFollowUp -ErrorAction SilentlyContinue | Out-Null
$noFollowUpOutput = $null
$noFollowUpResponse = $null
try {
    Push-Location $HostDir
    try {
        $noFollowUpOutput = & cargo run --quiet -- $EnginePath $RequestPath "deny" $trailPathNoFollowUp "success"
        if ($LASTEXITCODE -ne 0) {
            throw "Rust reference host exited with code $LASTEXITCODE during no-follow-up smoke."
        }
    }
    finally {
        Pop-Location
    }

    $noFollowUpText = ($noFollowUpOutput -join "`n").Trim()
    if ($noFollowUpText -eq "") {
        throw "Rust reference host produced no JSON output during no-follow-up smoke."
    }
    $noFollowUpResponse = $noFollowUpText | ConvertFrom-Json -ErrorAction Stop

    if ($noFollowUpResponse.execution_status -ne "denied") {
        throw "No-follow-up smoke must deny, got '$($noFollowUpResponse.execution_status)'."
    }

    # follow_up_evaluations field is absent.
    if ($null -ne $noFollowUpResponse.PSObject.Properties["follow_up_evaluations"]) {
        throw "Denied branch must not produce follow_up_evaluations, but the field is present."
    }

    # Result Anchor is absent.
    if ($null -ne $noFollowUpResponse.PSObject.Properties["result_anchor"]) {
        throw "Denied branch must not produce a result_anchor."
    }

    # No action_started, no action_completed, no action_failed.
    $kinds = @($noFollowUpResponse.trail | ForEach-Object { $_.kind })
    foreach ($forbidden in "action_started", "action_completed", "action_failed") {
        if ($forbidden -in $kinds) {
            throw "Denied trail must not contain '$forbidden'."
        }
    }

    # Durable Trail must be empty: denied never reached prepare_and_record.
    $durableLines = @((Get-Content -Raw -LiteralPath $trailPathNoFollowUp -ErrorAction SilentlyContinue) -split "\r?\n" | Where-Object { $_.Trim() -ne "" })
    if ($durableLines.Count -ne 0) {
        throw "Denied branch must write zero durable Trail records, found $($durableLines.Count)."
    }

    Write-Output "PASS test-host-result-follow-up (denied initial -> no follow-up)"
    Write-Output "Initial evaluation_id: $($noFollowUpResponse.evaluation_id)"
    Write-Output "Initial engine status: $($noFollowUpResponse.status)"
    Write-Output "Initial execution_status: $($noFollowUpResponse.execution_status)"
    Write-Output "follow_up_evaluations present: $($null -ne $noFollowUpResponse.PSObject.Properties['follow_up_evaluations'])"
    Write-Output "result_anchor present: $($null -ne $noFollowUpResponse.PSObject.Properties['result_anchor'])"
    Write-Output "action_started in trail: $(($noFollowUpResponse.trail | Where-Object { $_.kind -eq 'action_started' } | Measure-Object).Count)"
    Write-Output "action_completed in trail: $(($noFollowUpResponse.trail | Where-Object { $_.kind -eq 'action_completed' } | Measure-Object).Count)"
    Write-Output "action_failed in trail: $(($noFollowUpResponse.trail | Where-Object { $_.kind -eq 'action_failed' } | Measure-Object).Count)"
    Write-Output "Plan actions count: $($noFollowUpResponse.plan.actions.Count)"
    Write-Output "Provider calls (action_started count): 0"
    Write-Output "follow_up_evaluations is absent: true"
    Write-Output "Result Anchor is absent: true"
    Write-Output "Note: the with-follow-up path is proven by the 20 J10 unit tests in tethers-0.1/host-rust/src/main.rs (tests::j10_*)."
}
finally {
    if ($null -ne $trailPathNoFollowUp -and (Test-Path -LiteralPath $trailPathNoFollowUp)) {
        Remove-Item -LiteralPath $trailPathNoFollowUp -Force -ErrorAction SilentlyContinue
    }
    if ($null -ne $trailDirNoFollowUp -and (Test-Path -LiteralPath $trailDirNoFollowUp)) {
        Remove-Item -LiteralPath $trailDirNoFollowUp -Recurse -Force -ErrorAction SilentlyContinue
    }
}
