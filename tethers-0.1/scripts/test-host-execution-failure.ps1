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

# Derive a request from the happy path but swap the Capability name and
# Action to a capability the MockExecutor does not implement.  Keep the
# effect as lantern.write so the host policy authorises the Plan.
$happy = Get-Content -Raw -LiteralPath $HappyRequestPath | ConvertFrom-Json -ErrorAction Stop

$happy.capabilities[0].name = "lantern.task.fail"
$happy.capabilities[0].effects = @("lantern.write")

# Rewrite the Tether source so the Action calls the unsupported Capability.
$happy.tether.source = "tether `"Record completed software task`"`n`nanchor`n    coding.task_completed`n`nwhen`n    project.type is `"software`"`n    and task.changed_files greater_than 0`n`ndo`n    lantern.task.fail`n        project: anchor.project`n        task: anchor.task`n"

# Assign distinct identifiers.
$happy.evaluation_id = "eval_exec_fail_001"
$happy.event.id = "evt_exec_fail_001"

$failRequest = $happy | ConvertTo-Json -Depth 100 -Compress

# Write a temporary request file for the Rust host to read.
$tempRequestPath = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-execution-failure-request.json"
$failRequest | Set-Content -LiteralPath $tempRequestPath -NoNewline

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

# 1. Engine must return matched.
if ($response.status -ne "matched") {
    throw "Expected engine status 'matched' but got '$($response.status)'."
}

# 2. required_effects must include lantern.write.
$requiredEffects = $response.plan.required_effects
if ($requiredEffects -notcontains "lantern.write") {
    throw "Expected required_effects to contain 'lantern.write' but got: $($requiredEffects -join ', ')"
}

# 3. execution_status must be failed.
if ($response.execution_status -ne "failed") {
    throw "Expected execution_status 'failed' but got '$($response.execution_status)'."
}

# 4. Trail assertions.
$trail = $response.trail
if ($null -eq $trail -or $trail.Count -eq 0) {
    throw "Response has no Trail."
}

$kinds = $trail | ForEach-Object { $_.kind }

# 5. No plan_denied entry — effect was lantern.write, which is permitted.
if ("plan_denied" -in $kinds) {
    throw "Trail contains plan_denied but the Plan should have been authorised."
}

# 6. plan_authorised must appear.
$authorisedEntries = $trail | Where-Object { $_.kind -eq "plan_authorised" }
if ($authorisedEntries.Count -ne 1) {
    throw "Expected exactly 1 plan_authorised Trail entry, found $($authorisedEntries.Count)."
}

# 7. action_started must appear.
$startedEntries = $trail | Where-Object { $_.kind -eq "action_started" }
if ($startedEntries.Count -ne 1) {
    throw "Expected exactly 1 action_started Trail entry, found $($startedEntries.Count)."
}

# 8. action_failed must appear exactly once and contain the executor error.
$failedEntries = $trail | Where-Object { $_.kind -eq "action_failed" }
if ($failedEntries.Count -ne 1) {
    throw "Expected exactly 1 action_failed Trail entry, found $($failedEntries.Count)."
}

$expectedMessage = "no host executor is installed for lantern.task.fail"
if ($failedEntries[0].message -ne $expectedMessage) {
    throw "action_failed message expected '$expectedMessage', got '$($failedEntries[0].message)'."
}
if ($failedEntries[0].phase -ne "execution") {
    throw "action_failed phase expected 'execution', got '$($failedEntries[0].phase)'."
}
if ($failedEntries[0].outcome -ne "failed") {
    throw "action_failed outcome expected 'failed', got '$($failedEntries[0].outcome)'."
}

# 9. No action_completed entry.
if ("action_completed" -in $kinds) {
    throw "Trail contains action_completed but the Action should have failed."
}

# 10. Host Trail order: plan_authorised → action_started → action_failed.
$hostKinds = @("plan_denied", "plan_authorised", "action_started", "action_completed", "action_failed")
$hostEntries = $trail | Where-Object { $_.kind -in $hostKinds }
$hostEntryNames = $hostEntries | ForEach-Object { $_.kind }

$expectedSequence = @("plan_authorised", "action_started", "action_failed")
for ($i = 0; $i -lt 3; $i++) {
    if ($hostEntryNames[$i] -ne $expectedSequence[$i]) {
        throw "Host Trail order mismatch at position $i`: expected '$($expectedSequence[$i])', got '$($hostEntryNames[$i])'."
    }
}

# 11. Host Trail sequencing continues after the engine Trail.
$engineEntries = $trail | Where-Object { $_.kind -notin $hostKinds }
if ($engineEntries.Count -gt 0 -and $hostEntries.Count -gt 0) {
    $maxEngineSeq = ($engineEntries | ForEach-Object { $_.sequence } | Measure-Object -Maximum).Maximum
    foreach ($entry in $hostEntries) {
        if ($entry.sequence -le $maxEngineSeq) {
            throw "Host Trail entry '$($entry.kind)' has sequence $($entry.sequence), which does not follow engine Trail (max $maxEngineSeq)."
        }
    }
}

# 12. Exactly one Action was planned and tried — execution stopped after failure.
if ($response.plan.actions.Count -ne 1) {
    throw "Expected exactly 1 planned Action, found $($response.plan.actions.Count)."
}

Write-Output "PASS test-host-execution-failure"
Write-Output "Engine status: $($response.status)"
Write-Output "Required effects: $($requiredEffects -join ', ')"
Write-Output "Execution status: $($response.execution_status)"
Write-Output "Planned capability: $($response.plan.actions[0].capability)"
Write-Output "Failure message: $($failedEntries[0].message)"