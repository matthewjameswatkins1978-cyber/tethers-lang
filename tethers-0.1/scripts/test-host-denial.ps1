Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$HostDir = Join-Path $Root "host-rust"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
$HappyRequestPath = Join-Path $Root "protocol/cases/happy-path/request.json"

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

if (-not (Test-Path -LiteralPath $HappyRequestPath -PathType Leaf)) {
    throw "Missing happy-path fixture: $HappyRequestPath"
}

# Build the engine.
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

# Derive a request from the happy path but swap the Capability effect to
# network.write, which the reference host policy does not allow.
$happy = Get-Content -Raw -LiteralPath $HappyRequestPath | ConvertFrom-Json -ErrorAction Stop

# Change effect to one the host does not allow.
$happy.capabilities[0].effects = @("network.write")

# Assign distinct identifiers so this evaluation is traceable.
$happy.evaluation_id = "eval_denial_test_001"
$happy.event.id = "evt_denial_test_001"

$denialRequest = $happy | ConvertTo-Json -Depth 100 -Compress

# Write a temporary request file for the Rust host to read.
$tempRequestPath = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-denial-request.json"
$denialRequest | Set-Content -LiteralPath $tempRequestPath -NoNewline

# Run the host with the modified request.
Push-Location $HostDir
try {
    $output = & cargo run -- $EnginePath $tempRequestPath
    if ($LASTEXITCODE -ne 0) {
        throw "Rust reference host exited with code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}

Remove-Item -LiteralPath $tempRequestPath -ErrorAction SilentlyContinue

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

# --- Assertions ---

# 1. Engine must return matched — Conditions pass, Action is valid.
if ($response.status -ne "matched") {
    throw "Expected engine status 'matched' but got '$($response.status)'."
}

# 2. required_effects must include network.write.
$requiredEffects = $response.plan.required_effects
if ($requiredEffects -notcontains "network.write") {
    throw "Expected required_effects to contain 'network.write' but got: $($requiredEffects -join ', ')"
}

# 3. execution_status must be denied.
if ($response.execution_status -ne "denied") {
    throw "Expected execution_status 'denied' but got '$($response.execution_status)'."
}

# 4. Trail assertions.
$trail = $response.trail
if ($null -eq $trail -or $trail.Count -eq 0) {
    throw "Response has no Trail."
}

# Collect trail kinds for assertion.
$kinds = $trail | ForEach-Object { $_.kind }

# 5. Exactly one plan_denied entry.
$deniedEntries = $trail | Where-Object { $_.kind -eq "plan_denied" }
if ($deniedEntries.Count -ne 1) {
    throw "Expected exactly 1 plan_denied Trail entry, found $($deniedEntries.Count)."
}

# 6. Denial message identifies network.write.
if ($deniedEntries[0].message -notmatch "network\.write") {
    throw "Denial message did not reference network.write. Message: $($deniedEntries[0].message)"
}

# 7. Denial entry has correct phase and outcome.
if ($deniedEntries[0].phase -ne "authorisation") {
    throw "plan_denied phase expected 'authorisation', got '$($deniedEntries[0].phase)'."
}
if ($deniedEntries[0].outcome -ne "denied") {
    throw "plan_denied outcome expected 'denied', got '$($deniedEntries[0].outcome)'."
}

# 8. No plan_authorised entry.
if ("plan_authorised" -in $kinds) {
    throw "Trail contains plan_authorised but the Plan should have been denied."
}

# 9. No action_started entry.
if ("action_started" -in $kinds) {
    throw "Trail contains action_started but no Action should have executed."
}

# 10. No action_completed entry.
if ("action_completed" -in $kinds) {
    throw "Trail contains action_completed but no Action should have executed."
}

# 11. No action_failed entry.
if ("action_failed" -in $kinds) {
    throw "Trail contains action_failed but no Action should have started."
}

# 12. Host Trail sequencing continues after the engine Trail.
#    The engine Trail entries should have lower sequence numbers than the host entries.
$hostKinds = @("plan_denied", "plan_authorised", "action_started", "action_completed", "action_failed")
$hostEntries = $trail | Where-Object { $_.kind -in $hostKinds }
$engineEntries = $trail | Where-Object { $_.kind -notin $hostKinds }

if ($engineEntries.Count -gt 0 -and $hostEntries.Count -gt 0) {
    $maxEngineSeq = ($engineEntries | ForEach-Object { $_.sequence } | Measure-Object -Maximum).Maximum
    $minHostSeq = ($hostEntries | ForEach-Object { $_.sequence } | Measure-Object -Minimum).Minimum
    if ($maxEngineSeq -ge $minHostSeq) {
        throw "Host Trail sequence ($minHostSeq) does not follow engine Trail sequence ($maxEngineSeq)."
    }
}

# 13. Plan remains atomic — the actions array in the plan should still be present.
if ($null -eq $response.plan.actions -or $response.plan.actions.Count -eq 0) {
    throw "Plan actions array is missing or empty."
}

Write-Output "PASS test-host-denial"
Write-Output "Engine status: $($response.status)"
Write-Output "Required effects: $($requiredEffects -join ', ')"
Write-Output "Execution status: $($response.execution_status)"
Write-Output "Plan actions count: $($response.plan.actions.Count)"