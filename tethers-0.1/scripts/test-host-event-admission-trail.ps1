Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $Root "host-rust"
$DebugExe = Join-Path $HostDir "target/debug/tethers-reference-host.exe"
$ReleaseExe = Join-Path $HostDir "target/release/tethers-reference-host.exe"

function Assert-Command {
    param([Parameter(Mandatory = $true)][string] $Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

Assert-Command "cargo"

Push-Location $HostDir
try {
    Write-Host "Building debug host ..."
    $null = & cargo build 2>&1
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    Write-Host "Building release host ..."
    $null = & cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $DebugExe -PathType Leaf)) {
    throw "Debug host executable not found at $DebugExe"
}
if (-not (Test-Path -LiteralPath $ReleaseExe -PathType Leaf)) {
    throw "Release host executable not found at $ReleaseExe"
}

function Get-PropertyOrDefault {
    param($Object, [string]$Name)
    if ($Object.PSObject.Properties.Name -contains $Name) {
        return $Object.$Name
    }
    return $null
}

function Invoke-TrailProbe {
    param(
        [string]$Scenario,
        [string]$TrailPath
    )

    $output = & $DebugExe "event-admission-trail-probe" $Scenario $TrailPath 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "event-admission-trail-probe $Scenario failed with exit code ${LASTEXITCODE}: $($output -join "`n")"
    }

    $text = ($output -join "`n").Trim()
    if ($text -eq "") { throw "event-admission-trail-probe $Scenario produced no output" }

    try {
        $json = $text | ConvertFrom-Json
    }
    catch {
        throw "event-admission-trail-probe $Scenario did not produce valid JSON: $text"
    }

    # Read Trail file
    $trailLines = @()
    if (Test-Path -LiteralPath $TrailPath) {
        $rawLines = (Get-Content -Raw -LiteralPath $TrailPath -ErrorAction SilentlyContinue) -split "\r?\n" | Where-Object { $_.Trim() -ne "" }
        foreach ($line in $rawLines) {
            try {
                $trailLines += $line | ConvertFrom-Json
            }
            catch {
                throw "Trail line is not valid JSON: $line"
            }
        }
    }

    return @{
        Response = $json
        Trail    = $trailLines
    }
}

function Assert-TrailRecord {
    param(
        $Record,
        [string]$Kind,
        [string]$EventId,
        [string]$Source,
        [string]$Processing,
        [int]$Generation,
        [string]$ReasonCode,
        [int]$MaxGen
    )

    if ($Record.kind -ne $Kind) {
        throw "Expected kind='$Kind', got '$($Record.kind)'"
    }
    if ($Record.event_id -ne $EventId) {
        throw "Expected event_id='$EventId', got '$($Record.event_id)'"
    }
    if ($Record.source -ne $Source) {
        throw "Expected source='$Source', got '$($Record.source)'"
    }
    if ($Record.processing -ne $Processing) {
        throw "Expected processing='$Processing', got '$($Record.processing)'"
    }
    if ($Record.generation -ne $Generation) {
        throw "Expected generation=$Generation, got $($Record.generation)"
    }
    if ($ReasonCode -ne "") {
        if ($Record.reason_code -ne $ReasonCode) {
            throw "Expected reason_code='$ReasonCode', got '$($Record.reason_code)'"
        }
    }
    else {
        if ($null -ne (Get-PropertyOrDefault $Record 'reason_code')) {
            throw "reason_code must be absent, got '$($Record.reason_code)'"
        }
    }
    if ($MaxGen -ge 0) {
        if ($Record.maximum_generation -ne $MaxGen) {
            throw "Expected maximum_generation=$MaxGen, got $($Record.maximum_generation)"
        }
    }
    else {
        if ($null -ne (Get-PropertyOrDefault $Record 'maximum_generation')) {
            throw "maximum_generation must be absent, got $($Record.maximum_generation)"
        }
    }
    if ($Record.timestamp_unix_ms -isnot [long] -and $Record.timestamp_unix_ms -isnot [int]) {
        throw "timestamp_unix_ms must be numeric, got type $($Record.timestamp_unix_ms.GetType())"
    }
    if ($Record.timestamp_unix_ms -lt 0) {
        throw "timestamp_unix_ms must be non-negative, got $($Record.timestamp_unix_ms)"
    }
}

# -------------------------------------------------------------------
# Scenario 1: duplicate-initial
# -------------------------------------------------------------------
$tmpDir1 = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-j11-trail-dupinit-$([System.Guid]::NewGuid())"
$tmpTrail1 = Join-Path $tmpDir1 "trail.jsonl"
New-Item -ItemType Directory -Path $tmpDir1 -Force -ErrorAction SilentlyContinue | Out-Null
try {
    $result = Invoke-TrailProbe -Scenario "duplicate-initial" -TrailPath $tmpTrail1

    # Response checks
    if ($result.Response.kind -ne "event_admission_probe") { throw "Wrong response kind" }
    if ($result.Response.scenario -ne "duplicate-initial") { throw "Wrong scenario" }
    $rej = Get-PropertyOrDefault $result.Response 'event_admission_rejection'
    if ($null -eq $rej) { throw "Expected event_admission_rejection" }
    if ($rej.kind -ne "duplicate_event_id") { throw "Wrong rejection kind" }
    if ($rej.event_id -ne "evt/root") { throw "Wrong rejection event_id" }
    $rem = $result.Response.remaining_queue_event_ids
    if ($rem.Count -ne 1 -or $rem[0] -ne "evt/later") { throw "Wrong remaining queue" }

    # Trail checks: exactly 2 records
    if ($result.Trail.Count -ne 2) { throw "Expected 2 Trail records, got $($result.Trail.Count)" }
    Assert-TrailRecord -Record $result.Trail[0] -Kind "event_admitted" -EventId "evt/root" -Source "external" -Processing "continued" -Generation 0 -ReasonCode "" -MaxGen (-1)
    Assert-TrailRecord -Record $result.Trail[1] -Kind "event_rejected" -EventId "evt/root" -Source "result_anchor" -Processing "stopped" -Generation 1 -ReasonCode "duplicate_event_id" -MaxGen (-1)

    # evt/later must not appear in Trail
    foreach ($r in $result.Trail) {
        if ($r.event_id -eq "evt/later") { throw "evt/later must not appear in durable Trail" }
    }

    Write-Host "PASS duplicate-initial trail"
}
finally {
    Remove-Item -LiteralPath $tmpTrail1 -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tmpDir1 -Recurse -Force -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------
# Scenario 2: duplicate-sibling
# -------------------------------------------------------------------
$tmpDir2 = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-j11-trail-dupsib-$([System.Guid]::NewGuid())"
$tmpTrail2 = Join-Path $tmpDir2 "trail.jsonl"
New-Item -ItemType Directory -Path $tmpDir2 -Force -ErrorAction SilentlyContinue | Out-Null
try {
    $result = Invoke-TrailProbe -Scenario "duplicate-sibling" -TrailPath $tmpTrail2

    if ($result.Trail.Count -ne 3) { throw "Expected 3 Trail records, got $($result.Trail.Count)" }
    Assert-TrailRecord -Record $result.Trail[0] -Kind "event_admitted" -EventId "evt/root" -Source "external" -Processing "continued" -Generation 0 -ReasonCode "" -MaxGen (-1)
    Assert-TrailRecord -Record $result.Trail[1] -Kind "event_admitted" -EventId "evt/first" -Source "result_anchor" -Processing "continued" -Generation 1 -ReasonCode "" -MaxGen (-1)
    Assert-TrailRecord -Record $result.Trail[2] -Kind "event_rejected" -EventId "evt/first" -Source "result_anchor" -Processing "stopped" -Generation 1 -ReasonCode "duplicate_event_id" -MaxGen (-1)

    foreach ($r in $result.Trail) {
        if ($r.event_id -eq "evt/later") { throw "evt/later must not appear in durable Trail" }
    }

    Write-Host "PASS duplicate-sibling trail"
}
finally {
    Remove-Item -LiteralPath $tmpTrail2 -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tmpDir2 -Recurse -Force -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------
# Scenario 3: causal-depth
# -------------------------------------------------------------------
$tmpDir3 = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-j11-trail-depth-$([System.Guid]::NewGuid())"
$tmpTrail3 = Join-Path $tmpDir3 "trail.jsonl"
New-Item -ItemType Directory -Path $tmpDir3 -Force -ErrorAction SilentlyContinue | Out-Null
try {
    $result = Invoke-TrailProbe -Scenario "causal-depth" -TrailPath $tmpTrail3

    if ($result.Trail.Count -ne 2) { throw "Expected 2 Trail records, got $($result.Trail.Count)" }
    Assert-TrailRecord -Record $result.Trail[0] -Kind "event_admitted" -EventId "evt/root" -Source "external" -Processing "continued" -Generation 0 -ReasonCode "" -MaxGen (-1)
    Assert-TrailRecord -Record $result.Trail[1] -Kind "event_rejected" -EventId "evt/deep" -Source "result_anchor" -Processing "stopped" -Generation 9 -ReasonCode "causal_depth_exceeded" -MaxGen 8

    foreach ($r in $result.Trail) {
        if ($r.event_id -eq "evt/later") { throw "evt/later must not appear in durable Trail" }
    }

    Write-Host "PASS causal-depth trail"
}
finally {
    Remove-Item -LiteralPath $tmpTrail3 -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tmpDir3 -Recurse -Force -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------
# Scenario 4: clean
# -------------------------------------------------------------------
$tmpDir4 = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-j11-trail-clean-$([System.Guid]::NewGuid())"
$tmpTrail4 = Join-Path $tmpDir4 "trail.jsonl"
New-Item -ItemType Directory -Path $tmpDir4 -Force -ErrorAction SilentlyContinue | Out-Null
try {
    $result = Invoke-TrailProbe -Scenario "clean" -TrailPath $tmpTrail4

    if ($result.Trail.Count -ne 3) { throw "Expected 3 Trail records, got $($result.Trail.Count)" }
    Assert-TrailRecord -Record $result.Trail[0] -Kind "event_admitted" -EventId "evt/root" -Source "external" -Processing "continued" -Generation 0 -ReasonCode "" -MaxGen (-1)
    Assert-TrailRecord -Record $result.Trail[1] -Kind "event_admitted" -EventId "evt/a" -Source "result_anchor" -Processing "continued" -Generation 1 -ReasonCode "" -MaxGen (-1)
    Assert-TrailRecord -Record $result.Trail[2] -Kind "event_admitted" -EventId "evt/b" -Source "result_anchor" -Processing "continued" -Generation 8 -ReasonCode "" -MaxGen (-1)

    $rej = Get-PropertyOrDefault $result.Response 'event_admission_rejection'
    if ($null -ne $rej) { throw "Clean scenario must have no rejection" }

    Write-Host "PASS clean trail"
}
finally {
    Remove-Item -LiteralPath $tmpTrail4 -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tmpDir4 -Recurse -Force -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------
# Negative: missing scenario
# -------------------------------------------------------------------
$negOut = & $DebugExe "event-admission-trail-probe" 2>&1
if ($LASTEXITCODE -eq 0) { throw "missing scenario must fail" }
if (($negOut -join "`n") -notmatch "event-admission-trail-probe") { throw "missing scenario must print usage" }
Write-Host "PASS missing scenario"

# -------------------------------------------------------------------
# Negative: unknown scenario
# -------------------------------------------------------------------
$negOut2 = & $DebugExe "event-admission-trail-probe" "nonexistent" "/tmp/trail.jsonl" 2>&1
if ($LASTEXITCODE -eq 0) { throw "unknown scenario must fail" }
if (($negOut2 -join "`n") -notmatch "event-admission-trail-probe") { throw "unknown scenario must print usage" }
Write-Host "PASS unknown scenario"

# -------------------------------------------------------------------
# Negative: missing path
# -------------------------------------------------------------------
$negOut3 = & $DebugExe "event-admission-trail-probe" "clean" 2>&1
if ($LASTEXITCODE -eq 0) { throw "missing path must fail" }
if (($negOut3 -join "`n") -notmatch "event-admission-trail-probe") { throw "missing path must print usage" }
Write-Host "PASS missing path"

# -------------------------------------------------------------------
# Negative: extra argument
# -------------------------------------------------------------------
$negOut4 = & $DebugExe "event-admission-trail-probe" "clean" "/tmp/trail.jsonl" "extra" 2>&1
if ($LASTEXITCODE -eq 0) { throw "extra argument must fail" }
if (($negOut4 -join "`n") -notmatch "event-admission-trail-probe") { throw "extra argument must print usage" }
Write-Host "PASS extra argument"

# -------------------------------------------------------------------
# Negative: relative path
# -------------------------------------------------------------------
$negOut5 = & $DebugExe "event-admission-trail-probe" "clean" "relative/path.jsonl" 2>&1
if ($LASTEXITCODE -eq 0) { throw "relative path must fail" }
if (($negOut5 -join "`n") -notmatch "event-admission-trail-probe") { throw "relative path must print usage" }
Write-Host "PASS relative path"

# -------------------------------------------------------------------
# Release: neither diagnostic command available
# -------------------------------------------------------------------
$relOut1 = & $ReleaseExe "event-admission-probe" "clean" 2>&1
if ($LASTEXITCODE -eq 0) { throw "release must not have event-admission-probe" }
$relOut2 = & $ReleaseExe "event-admission-trail-probe" "clean" "/tmp/trail.jsonl" 2>&1
if ($LASTEXITCODE -eq 0) { throw "release must not have event-admission-trail-probe" }
Write-Host "PASS release diagnostic absence"

# -------------------------------------------------------------------
Write-Host "PASS test-host-event-admission-trail"
