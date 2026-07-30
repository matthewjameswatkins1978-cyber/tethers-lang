Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$HostPath = Join-Path $HostDir "target\debug\tethers-reference-host.exe"
$EnginePath = Join-Path $RepoRoot "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"
$StandingManifest = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping-standing-allow.json"
$FixtureProvider = Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1"
$StandingDigest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da"

$script:caseCount = 0
$script:passedCount = 0
$script:assertionCount = 0

function Assert-True {
    param([bool]$Condition, [string]$Message)
    $script:assertionCount++
    if (-not $Condition) { throw $Message }
}

function Assert-Equal {
    param($Actual, $Expected, [string]$Message)
    $script:assertionCount++
    if ($Actual -ne $Expected) { throw "$Message Expected '$Expected', got '$Actual'." }
}

function Invoke-Case {
    param([string]$Name, [scriptblock]$Body)
    $script:caseCount++
    Write-Output "TEST: $($script:caseCount). $Name"
    & $Body
    $script:passedCount++
    Write-Output "  PASS"
}

function Invoke-Host {
    param([string]$WorkingDirectory, [string[]]$ArgumentList)
    Push-Location $WorkingDirectory
    try {
        $output = @(& $HostPath @ArgumentList 2>&1)
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    [pscustomobject]@{
        ExitCode = $exitCode
        Stdout   = ($output -join "`n")
    }
}

function ConvertFrom-SingleEnvelope {
    param([Parameter(Mandatory = $true)]$Result,
          [string]$ExpectedCommand,
          [string]$ExpectedStatus,
          [int]$ExpectedExit)
    $lines = @($Result.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    Assert-Equal $lines.Count 1 "stdout must contain exactly one JSON document"
    $envelope = $lines[0] | ConvertFrom-Json -ErrorAction Stop
    Assert-Equal $envelope.schema "tethers.cli/1" "schema mismatch"
    Assert-Equal $envelope.command $ExpectedCommand "command mismatch"
    Assert-Equal $envelope.status $ExpectedStatus "status mismatch"
    Assert-Equal ([int]$envelope.exit_code) $ExpectedExit "embedded exit code mismatch"
    Assert-Equal $Result.ExitCode $ExpectedExit "process exit code mismatch"
    return $envelope
}

function Get-FileHash-SHA256 {
    param([string]$Path)
    (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLower()
}

function Write-Text {
    param([string]$Path, [string]$Text)
    [System.IO.File]::WriteAllText($Path, $Text, [System.Text.UTF8Encoding]::new($false))
}

function Get-MethodCount {
    param([string]$Marker, [string]$Method)
    if (-not (Test-Path -LiteralPath $Marker -PathType Leaf)) { return 0 }
    return @((Get-Content -LiteralPath $Marker) | Where-Object { $_ -eq $Method }).Count
}

function Provision-ReplayRoot {
    param([string]$Root, [string]$WorkingDirectory)
    if (-not (Test-Path -LiteralPath $Root -PathType Container)) {
        New-Item -ItemType Directory -Path $Root | Out-Null
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
        $acl = [System.Security.AccessControl.DirectorySecurity]::new()
        $acl.SetAccessRuleProtection($true, $false)
        $inheritance = [System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit
        $propagation = [System.Security.AccessControl.PropagationFlags]::None
        foreach ($trustee in @($identity, "NT AUTHORITY\SYSTEM", "BUILTIN\Administrators")) {
            $rule = [System.Security.AccessControl.FileSystemAccessRule]::new(
                $trustee,
                [System.Security.AccessControl.FileSystemRights]::FullControl,
                $inheritance,
                $propagation,
                [System.Security.AccessControl.AccessControlType]::Allow
            )
            $acl.AddAccessRule($rule)
        }
        Set-Acl -LiteralPath $Root -AclObject $acl
    }
    $result = Invoke-Host -WorkingDirectory $WorkingDirectory -ArgumentList @("provision-replay", $Root)
    Assert-Equal $result.ExitCode 0 "replay provisioning failed: $($result.Stdout)"
}

# ------------------------------------------------------------------
# Pre-flight
# ------------------------------------------------------------------
if (-not (Test-Path -LiteralPath $HostPath -PathType Leaf)) {
    throw "Host executable is missing: $HostPath. Run cargo build first."
}
if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "OCaml engine executable is missing: $EnginePath."
}
if (-not (Test-Path -LiteralPath $StandingManifest -PathType Leaf)) {
    throw "Required reviewed run fixture manifest is missing: $StandingManifest"
}

# ------------------------------------------------------------------
# Build workspace (J13B-style)
# ------------------------------------------------------------------
$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("Tethers J14A scenario " + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempRoot | Out-Null

try {
    $workspace = Join-Path $TempRoot "workspace"
    $manifestsDir = Join-Path $workspace "manifests"
    $tethersDir = Join-Path $workspace "tethers"
    $scriptsDir = Join-Path $workspace "scripts"
    New-Item -ItemType Directory -Force -Path $manifestsDir, $tethersDir, $scriptsDir | Out-Null

    # Copy assets
    Copy-Item -LiteralPath $StandingManifest -Destination (Join-Path $manifestsDir "fixture-ping-standing-allow.json")
    Copy-Item -LiteralPath $FixtureProvider -Destination (Join-Path $scriptsDir "tethers-stdio-fixture.ps1")

    # Copy scenario Tether source
    Copy-Item -LiteralPath (Join-Path $RepoRoot "scenarios\j14-complete-local\tethers\complete.tether") -Destination (Join-Path $tethersDir "complete.tether")

    # Hash committed source for non-mutation check
    $committedTetherPath = Join-Path $RepoRoot "scenarios\j14-complete-local\tethers\complete.tether"
    $committedInputPath = Join-Path $RepoRoot "scenarios\j14-complete-local\input.json"
    $hashTetherStart = Get-FileHash-SHA256 $committedTetherPath
    $hashInputStart  = Get-FileHash-SHA256 $committedInputPath

    $marker = Join-Path $workspace "provider-methods.txt"

    # Runtime config (same shape as J13B but with j14-complete identity)
    $config = [ordered]@{
        format_version = "0.1"
        tether_set = [ordered]@{
            id = "scenario.j14.complete-local"
            version = "1"
            tethers = @([ordered]@{ id = "j14-complete"; version = "1"; source_path = "tethers/complete.tether" })
            capability_requirements = @([ordered]@{ name = "fixture.ping"; version = 1; reason = "J14A complete local scenario" })
        }
        providers = @([ordered]@{
            id = "tethers-stdio-fixture"
            display_name = "Tethers Stdio Fixture"
            transport = [ordered]@{
                kind = "stdio"
                command = "pwsh.exe"
                args = @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", "run-success", "-MarkerFile", $marker)
                protocol_version = "2025-11-25"
            }
            capabilities = @([ordered]@{
                name = "fixture.ping"
                version = 1
                manifest_path = "manifests/fixture-ping-standing-allow.json"
                pinned_digest = $StandingDigest
                scope_binding = [ordered]@{
                    kind = "path_prefix"
                    argument_json_pointer = "/path"
                }
            })
        })
        policy = [ordered]@{
            default = "deny"
            rules = @([ordered]@{ name = "fixture.ping"; version = 1; decision = "allow" })
        }
    }
    $configPath = Join-Path $workspace "runtime.json"
    $config | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $configPath -Encoding utf8NoBOM

    # Input file
    $input = [ordered]@{
        format_version = "1"
        evaluation_id = "eval_j14_complete_001"
        tether = [ordered]@{ id = "j14-complete"; version = "1" }
        event = [ordered]@{
            id = "evt_j14_complete_001"
            name = "coding.task_completed"
            data = [ordered]@{ project = "lantern-keeper"; task = "LK-39"; path = "projects/LK-39" }
        }
        facts = [ordered]@{ "project.type" = "software"; "task.changed_files" = 3 }
    }
    $inputPath = Join-Path $workspace "input.json"
    $input | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $inputPath -Encoding utf8NoBOM

    # Replay provisioning
    $replayRoot = Join-Path $workspace "replay-data"
    $trailPath = Join-Path $workspace "trail.jsonl"

    Provision-ReplayRoot $replayRoot $workspace

    # ------------------------------------------------------------------
    # Phase 2: Check
    # ------------------------------------------------------------------
    Invoke-Case "check validates Tether, provider, and availability" {
        $checkResult = Invoke-Host $workspace @("check", "--config", $configPath, "--engine", $EnginePath)
        $checkEnv = ConvertFrom-SingleEnvelope $checkResult "check" "ok" 0

        Assert-True ($null -ne $checkEnv.data.tethers) "check data missing tethers"
        Assert-Equal $checkEnv.data.tethers.Count 1 "expected one configured Tether"
        Assert-Equal $checkEnv.data.tethers[0].id "j14-complete" "tether ID"
        Assert-Equal $checkEnv.data.tethers[0].status "valid" "tether status"

        Assert-True ($null -ne $checkEnv.data.providers) "check data missing providers"
        Assert-Equal $checkEnv.data.providers.Count 1 "expected one configured provider"
        Assert-Equal $checkEnv.data.providers[0].status "available" "provider status"

        Assert-Equal (Get-MethodCount $marker "initialize") 1 "provider initialize count"
        Assert-Equal (Get-MethodCount $marker "tools/list") 1 "provider tools/list count"
        Assert-Equal (Get-MethodCount $marker "tools/call") 0 "provider tools/call count during check"

        Assert-True (-not (Test-Path -LiteralPath $trailPath)) "Trail must not exist after check"

        Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue
    }

    # ------------------------------------------------------------------
    # Phase 3: First Run
    # ------------------------------------------------------------------
    $script:executionId = $null

    Invoke-Case "first run completes and records Result Anchor" {
        $run1Result = Invoke-Host $workspace @(
            "run", "--config", $configPath, "--engine", $EnginePath,
            "--input", $inputPath, "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $run1Env = ConvertFrom-SingleEnvelope $run1Result "run" "completed" 0

        Assert-Equal $run1Env.data.evaluation_id "eval_j14_complete_001" "evaluation_id"
        Assert-True ($null -ne $run1Env.data.action_id) "action_id is non-empty"

        $executionId = $run1Env.data.execution_id
        Assert-True ($null -ne $executionId) "execution_id must be present"
        Assert-True ($executionId -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "execution_id UUIDv4 format"

        Assert-Equal $run1Env.data.execution_status "completed" "execution_status"

        Assert-Equal (Get-MethodCount $marker "initialize") 1 "first run initialize count"
        Assert-Equal (Get-MethodCount $marker "tools/list") 1 "first run tools/list count"
        Assert-Equal (Get-MethodCount $marker "tools/call") 1 "first run tools/call count"

        # Result Anchor proof
        $anchor = $run1Env.data.result_anchor
        Assert-True ($null -ne $anchor) "result_anchor must be present"
        Assert-Equal $anchor.event_id "$($run1Env.data.evaluation_id)/$($run1Env.data.action_id)/result" "anchor event_id"
        Assert-Equal $anchor.event_name "capability.succeeded" "anchor event_name"
        Assert-Equal $anchor.producer "tethers-reference-host" "anchor producer"
        Assert-Equal $anchor.correlation_id "evt_j14_complete_001" "anchor correlation_id"
        Assert-Equal $anchor.causation_id "evt_j14_complete_001" "anchor causation_id"
        Assert-Equal ([int]$anchor.generation) 1 "anchor generation"

        Assert-Equal $anchor.facts.evaluation_id $run1Env.data.evaluation_id "anchor facts.evaluation_id"
        Assert-Equal $anchor.facts.action_id $run1Env.data.action_id "anchor facts.action_id"
        Assert-Equal $anchor.facts.capability.name "fixture.ping" "anchor facts.capability.name"
        Assert-Equal ([int]$anchor.facts.capability.version) 1 "anchor facts.capability.version"
        Assert-Equal $anchor.facts.manifest_digest $StandingDigest "anchor facts.manifest_digest"
        Assert-Equal $anchor.facts.provider_identity "tethers-stdio-fixture" "anchor facts.provider_identity"
        Assert-Equal $anchor.facts.result.echo "LK-39" "anchor facts.result.echo"

        $anchorStr = $anchor | ConvertTo-Json -Compress
        Assert-True ($anchorStr -notmatch "execution_id") "result_anchor must not contain execution_id"

        $script:executionId = $executionId
    }

    # ------------------------------------------------------------------
    # Phase 4: Public Trail Inspection
    # ------------------------------------------------------------------
    Invoke-Case "public trail inspection returns execution entries in order" {
        $trailResult = Invoke-Host $workspace @(
            "trail", "--trail", $trailPath, "--execution-id", $script:executionId
        )
        $trailEnv = ConvertFrom-SingleEnvelope $trailResult "trail" "ok" 0

        Assert-Equal $trailEnv.data.execution_id $script:executionId "trail execution_id matches"
        Assert-Equal $trailEnv.data.entry_count 2 "trail must have exactly two execution-specific entries"

        $entries = $trailEnv.data.entries
        Assert-Equal $entries.Count 2 "two entries"
        Assert-Equal $entries[0].execution_id $script:executionId "entry 0 execution_id"
        Assert-Equal $entries[1].execution_id $script:executionId "entry 1 execution_id"

        # Intent precedes outcome
        Assert-True ($entries[0].PSObject.Properties["capability_name"] -ne $null) "intent has capability_name"
        Assert-Equal $entries[0].capability_name "fixture.ping" "intent capability_name"
        Assert-Equal ([int]$entries[0].capability_version) 1 "intent capability_version"
        Assert-Equal $entries[0].provider_identity "tethers-stdio-fixture" "intent provider_identity"
        Assert-Equal $entries[0].manifest_digest $StandingDigest "intent manifest_digest"

        $intentStr = $entries[0] | ConvertTo-Json -Compress
        Assert-True ($intentStr -match "LK-39") "intent contains LK-39"
        Assert-True ($intentStr -match "projects/LK-39") "intent contains projects/LK-39"

        Assert-True ($entries[1].PSObject.Properties["status"] -ne $null) "outcome has status"
        Assert-Equal $entries[1].status "succeeded" "outcome status succeeded"

        $script:savedTrail = $trailResult.Stdout
    }

    # ------------------------------------------------------------------
    # Phase 5: Exact Replay
    # ------------------------------------------------------------------
    Invoke-Case "exact replay blocks duplicate effect and returns same execution ID" {
        $replayResult = Invoke-Host $workspace @(
            "run", "--config", $configPath, "--engine", $EnginePath,
            "--input", $inputPath, "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $replayEnv = ConvertFrom-SingleEnvelope $replayResult "run" "completed" 0

        Assert-Equal $replayEnv.data.execution_status "replay_blocked_completed_success" "replay status"
        Assert-Equal $replayEnv.data.execution_id $script:executionId "replay execution_id must match"

        Assert-Equal (Get-MethodCount $marker "tools/call") 1 "replay must not invoke provider again"

        # Trail inspection returns identical entries
        $replayTrailResult = Invoke-Host $workspace @(
            "trail", "--trail", $trailPath, "--execution-id", $script:executionId
        )
        $replayTrailEnv = ConvertFrom-SingleEnvelope $replayTrailResult "trail" "ok" 0
        Assert-Equal $replayTrailEnv.data.entry_count 2 "replay trail entry count unchanged"
    }

    # ------------------------------------------------------------------
    # Phase 6: Non-Mutation
    # ------------------------------------------------------------------
    Invoke-Case "committed scenario sources are unchanged" {
        Assert-Equal (Get-FileHash-SHA256 $committedTetherPath) $hashTetherStart "complete.tether hash unchanged"
        Assert-Equal (Get-FileHash-SHA256 $committedInputPath) $hashInputStart "input.json hash unchanged"
    }

    Write-Output ""
    Write-Output "============================================"
    Write-Output "TOTAL: $caseCount cases, $passedCount passed, 0 failed"
    Write-Output "ASSERTIONS: $assertionCount"
    Write-Output "FIRST EXECUTION ID: $script:executionId"
    Write-Output "============================================"
}
finally {
    Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
