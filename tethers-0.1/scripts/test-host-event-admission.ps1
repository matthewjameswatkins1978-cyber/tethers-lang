Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $Root "host-rust"
$HostExe = Join-Path $HostDir "target/debug/tethers-reference-host.exe"

function Assert-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

Assert-Command "cargo"

Push-Location $HostDir
try {
    Write-Host "Building debug host ..."
    $build = & cargo build 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE; output: $($build -join "`n")"
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $HostExe -PathType Leaf)) {
    throw "Debug host executable not found at $HostExe"
}

function Invoke-AdmissionProbe {
    param([string] $Scenario)

    $output = & $HostExe "event-admission-probe" $Scenario 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "event-admission-probe $Scenario failed with exit code ${LASTEXITCODE}: $($output -join "`n")"
    }

    $text = ($output -join "`n").Trim()
    if ($text -eq "") {
        throw "event-admission-probe $Scenario produced no output"
    }

    try {
        $json = $text | ConvertFrom-Json
    }
    catch {
        throw "event-admission-probe $Scenario did not produce valid JSON: $text"
    }

    return $json
}

function Get-PropertyOrDefault {
    param($Object, [string]$Name)
    if ($Object.PSObject.Properties.Name -contains $Name) {
        return $Object.$Name
    }
    return $null
}

# -------------------------------------------------------------------
# Scenario A: duplicate-initial
# -------------------------------------------------------------------
$response = Invoke-AdmissionProbe "duplicate-initial"

if ($response.kind -ne "event_admission_probe") {
    throw "duplicate-initial: expected kind=event_admission_probe, got $($response.kind)"
}
if ($response.scenario -ne "duplicate-initial") {
    throw "duplicate-initial: wrong scenario $($response.scenario)"
}
if ($response.initial_event_id -ne "evt/root") {
    throw "duplicate-initial: wrong initial_event_id $($response.initial_event_id)"
}

$fu = (Get-PropertyOrDefault $response 'follow_up_evaluations')
if ($null -ne $fu) {
    throw "duplicate-initial: expected no follow_up_evaluations, got $($fu | ConvertTo-Json -Compress)"
}

$rej = (Get-PropertyOrDefault $response 'event_admission_rejection')
if ($null -eq $rej) {
    throw "duplicate-initial: expected event_admission_rejection"
}
if ($rej.kind -ne "duplicate_event_id") {
    throw "duplicate-initial: expected kind=duplicate_event_id, got $($rej.kind)"
}
if ($rej.event_id -ne "evt/root") {
    throw "duplicate-initial: wrong rejection event_id $($rej.event_id)"
}
if ($rej.generation -ne 1) {
    throw "duplicate-initial: wrong rejection generation $($rej.generation)"
}
if ($rej.processing -ne "stopped") {
    throw "duplicate-initial: wrong processing $($rej.processing)"
}

$rem = $response.remaining_queue_event_ids
if ($rem.Count -ne 1 -or $rem[0] -ne "evt/later") {
    throw "duplicate-initial: expected remaining=['evt/later'], got $($rem | ConvertTo-Json -Compress)"
}

Write-Host "PASS duplicate-initial"

# -------------------------------------------------------------------
# Scenario B: duplicate-sibling
# -------------------------------------------------------------------
$response = Invoke-AdmissionProbe "duplicate-sibling"

$fu = (Get-PropertyOrDefault $response 'follow_up_evaluations')
if ($null -eq $fu -or $fu.Count -ne 1) {
    throw "duplicate-sibling: expected 1 follow_up_evaluation, got $($fu | ConvertTo-Json -Compress)"
}
if ($fu[0].input_event_id -ne "evt/first") {
    throw "duplicate-sibling: wrong input_event_id $($fu[0].input_event_id)"
}
if ($fu[0].generation -ne 1) {
    throw "duplicate-sibling: wrong generation $($fu[0].generation)"
}
if ($fu[0].response.status -ne "evaluated") {
    throw "duplicate-sibling: wrong response status $($fu[0].response.status)"
}
if ($fu[0].response.event_id -ne "evt/first") {
    throw "duplicate-sibling: wrong response event_id $($fu[0].response.event_id)"
}
if ($fu[0].response.generation -ne 1) {
    throw "duplicate-sibling: wrong response generation $($fu[0].response.generation)"
}

$rej = (Get-PropertyOrDefault $response 'event_admission_rejection')
if ($null -eq $rej) {
    throw "duplicate-sibling: expected event_admission_rejection"
}
if ($rej.kind -ne "duplicate_event_id") {
    throw "duplicate-sibling: expected kind=duplicate_event_id, got $($rej.kind)"
}
if ($rej.event_id -ne "evt/first") {
    throw "duplicate-sibling: wrong rejection event_id $($rej.event_id)"
}
if ($rej.generation -ne 1) {
    throw "duplicate-sibling: wrong rejection generation $($rej.generation)"
}

$rem = $response.remaining_queue_event_ids
if ($rem.Count -ne 1 -or $rem[0] -ne "evt/later") {
    throw "duplicate-sibling: expected remaining=['evt/later'], got $($rem | ConvertTo-Json -Compress)"
}

Write-Host "PASS duplicate-sibling"

# -------------------------------------------------------------------
# Scenario C: causal-depth
# -------------------------------------------------------------------
$response = Invoke-AdmissionProbe "causal-depth"

$fu = (Get-PropertyOrDefault $response 'follow_up_evaluations')
if ($null -ne $fu) {
    throw "causal-depth: expected no follow_up_evaluations, got $($fu | ConvertTo-Json -Compress)"
}

$rej = (Get-PropertyOrDefault $response 'event_admission_rejection')
if ($null -eq $rej) {
    throw "causal-depth: expected event_admission_rejection"
}
if ($rej.kind -ne "causal_depth_exceeded") {
    throw "causal-depth: expected kind=causal_depth_exceeded, got $($rej.kind)"
}
if ($rej.event_id -ne "evt/deep") {
    throw "causal-depth: wrong rejection event_id $($rej.event_id)"
}
if ($rej.generation -ne 9) {
    throw "causal-depth: wrong rejection generation $($rej.generation)"
}
if ($rej.maximum_generation -ne 8) {
    throw "causal-depth: wrong maximum_generation $($rej.maximum_generation)"
}
if ($rej.processing -ne "stopped") {
    throw "causal-depth: wrong processing $($rej.processing)"
}

$rem = $response.remaining_queue_event_ids
if ($rem.Count -ne 1 -or $rem[0] -ne "evt/later") {
    throw "causal-depth: expected remaining=['evt/later'], got $($rem | ConvertTo-Json -Compress)"
}

Write-Host "PASS causal-depth"

# -------------------------------------------------------------------
# Scenario D: clean
# -------------------------------------------------------------------
$response = Invoke-AdmissionProbe "clean"

$fu = (Get-PropertyOrDefault $response 'follow_up_evaluations')
if ($null -eq $fu -or $fu.Count -ne 2) {
    throw "clean: expected 2 follow_up_evaluations, got $($fu | ConvertTo-Json -Compress)"
}
if ($fu[0].input_event_id -ne "evt/a") {
    throw "clean: wrong first input_event_id $($fu[0].input_event_id)"
}
if ($fu[0].generation -ne 1) {
    throw "clean: wrong first generation $($fu[0].generation)"
}
if ($fu[0].response.status -ne "evaluated") {
    throw "clean: wrong first response status $($fu[0].response.status)"
}
if ($fu[1].input_event_id -ne "evt/b") {
    throw "clean: wrong second input_event_id $($fu[1].input_event_id)"
}
if ($fu[1].generation -ne 8) {
    throw "clean: wrong second generation $($fu[1].generation)"
}
if ($fu[1].response.status -ne "evaluated") {
    throw "clean: wrong second response status $($fu[1].response.status)"
}

$rej = (Get-PropertyOrDefault $response 'event_admission_rejection')
if ($null -ne $rej) {
    throw "clean: expected no event_admission_rejection, got $($rej | ConvertTo-Json -Compress)"
}

$rem = $response.remaining_queue_event_ids
if ($rem.Count -ne 0) {
    throw "clean: expected empty remaining, got $($rem | ConvertTo-Json -Compress)"
}

Write-Host "PASS clean"

# -------------------------------------------------------------------
Write-Host "PASS test-host-event-admission"
