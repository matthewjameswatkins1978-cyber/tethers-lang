Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$HostPath = Join-Path $HostDir "target\debug\tethers-reference-host.exe"
$EnginePath = Join-Path $RepoRoot "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"
$StandingManifest = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping-standing-allow.json"
$FixtureProvider = Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1"
$StandingDigest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da"
$CargoLockHash = "c72087d25475c82a13e3b57396f57e965dbeca1f76a33b738322523a54fc20a3"

$ScenarioDir = Join-Path $RepoRoot "scenarios\j14-complete-local"
$CommittedTether = Join-Path $ScenarioDir "tethers\complete.tether"
$CommittedInput = Join-Path $ScenarioDir "input.json"
$CommittedTemplate = Join-Path $ScenarioDir "runtime.template.json"
$CommittedReadme = Join-Path $ScenarioDir "README.md"

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

function Write-Utf8NoBom {
    param([string]$Path, [string]$Text)
    [System.IO.File]::WriteAllText($Path, $Text, [System.Text.UTF8Encoding]::new($false))
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

function Get-RepoGitStatus {
    Push-Location $RepoRoot
    try {
        $status = & git status --porcelain=v1 --untracked-files=all 2>&1
        return ($status -join "`n").Trim()
    }
    finally {
        Pop-Location
    }
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
    throw "Required fixture manifest is missing: $StandingManifest"
}
if (-not (Test-Path -LiteralPath $FixtureProvider -PathType Leaf)) {
    throw "Required fixture provider is missing: $FixtureProvider"
}

# Verify Cargo.lock hash
$cargoLockPath = Join-Path $HostDir "Cargo.lock"
$cargoLockActual = (Get-FileHash -Path $cargoLockPath -Algorithm SHA256).Hash.ToLower()
if ($cargoLockActual -ne $CargoLockHash) {
    throw "Cargo.lock hash mismatch: expected $CargoLockHash, got $cargoLockActual"
}

# ------------------------------------------------------------------
# Snapshot committed hashes before any work
# ------------------------------------------------------------------
$hashTetherStart = Get-FileHash-SHA256 $CommittedTether
$hashInputStart = Get-FileHash-SHA256 $CommittedInput
$hashTemplateStart = Get-FileHash-SHA256 $CommittedTemplate
$hashReadmeStart = Get-FileHash-SHA256 $CommittedReadme

# Snapshot repository git status
$gitStatusBefore = Get-RepoGitStatus

# ------------------------------------------------------------------
# Build workspace - Unicode + space temp path
# ------------------------------------------------------------------
$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("Tethers J14A caf" + [char]0x00E9 + " " + [guid]::NewGuid().ToString())
Assert-True ($TempRoot -match " ") "temp root must contain at least one space"
Assert-True ($TempRoot -cmatch "[^\x00-\x7F]") "temp root must contain at least one non-ASCII character"

New-Item -ItemType Directory -Path $TempRoot | Out-Null

try {
    $workspace = Join-Path $TempRoot "workspace"
    $manifestsDir = Join-Path $workspace "manifests"
    $tethersDir = Join-Path $workspace "tethers"
    $scriptsDir = Join-Path $workspace "scripts"
    New-Item -ItemType Directory -Force -Path $manifestsDir, $tethersDir, $scriptsDir | Out-Null

    # Copy assets (fixtures, not scenario sources)
    Copy-Item -LiteralPath $StandingManifest -Destination (Join-Path $manifestsDir "fixture-ping-standing-allow.json")
    Copy-Item -LiteralPath $FixtureProvider -Destination (Join-Path $scriptsDir "tethers-stdio-fixture.ps1")

    # Copy the committed scenario source files exactly
    Copy-Item -LiteralPath $CommittedTether -Destination (Join-Path $tethersDir "complete.tether")
    Copy-Item -LiteralPath $CommittedInput -Destination (Join-Path $workspace "input.json")
    Copy-Item -LiteralPath $CommittedTemplate -Destination (Join-Path $workspace "runtime.template.json")

    $marker = Join-Path $workspace "provider-methods.txt"
    $trailPath = Join-Path $workspace "trail.jsonl"
    $replayRoot = Join-Path $workspace "replay-data"

    # Materialise runtime.json using the committed template's content
    # (hash-protected) and the same programmatic construction pattern the
    # existing regression harnesses use.
    $providerScriptPath = Join-Path $scriptsDir "tethers-stdio-fixture.ps1"

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
                args = @("-NoProfile", "-File", $providerScriptPath, "-Mode", "run-success", "-MarkerFile", $marker)
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
    $runtimePath = Join-Path $workspace "runtime.json"
    $configJson = $config | ConvertTo-Json -Depth 30
    Write-Utf8NoBom $runtimePath $configJson

    # Verify template is byte-identical
    $hashTemplateAfterCopy = Get-FileHash-SHA256 (Join-Path $workspace "runtime.template.json")
    Assert-Equal $hashTemplateAfterCopy $hashTemplateStart "copied runtime.template.json hash unchanged"

    # Provision replay
    Provision-ReplayRoot $replayRoot $workspace

    # Snapshot replay root tree before check
    $replayTreeBefore = Get-ChildItem -Recurse -LiteralPath $replayRoot | ForEach-Object { $_.FullName } | Sort-Object
    $replayTreeBeforeStr = ($replayTreeBefore -join "`n")

    # ------------------------------------------------------------------
    # Phase: Check
    # ------------------------------------------------------------------
    Invoke-Case "check validates Tether, provider, and availability" {
        $checkResult = Invoke-Host $workspace @("check", "--config", $runtimePath, "--engine", $EnginePath)
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

        # Prove replay tree unchanged after check
        $replayTreeAfter = Get-ChildItem -Recurse -LiteralPath $replayRoot | ForEach-Object { $_.FullName } | Sort-Object
        $replayTreeAfterStr = ($replayTreeAfter -join "`n")
        Assert-Equal $replayTreeAfterStr $replayTreeBeforeStr "replay root tree unchanged after check"

        Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue
    }

    # ------------------------------------------------------------------
    # Phase: First Run
    # ------------------------------------------------------------------
    $script:executionId = $null

    Invoke-Case "first run completes and records Result Anchor" {
        $run1Result = Invoke-Host $workspace @(
            "run", "--config", $runtimePath, "--engine", $EnginePath,
            "--input", (Join-Path $workspace "input.json"), "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $run1Env = ConvertFrom-SingleEnvelope $run1Result "run" "completed" 0

        Assert-Equal $run1Env.data.evaluation_id "eval_j14_complete_001" "evaluation_id"
        Assert-True ($null -ne $run1Env.data.action_id) "action_id is non-empty"

        $executionId = $run1Env.data.execution_id
        Assert-True ($null -ne $executionId) "execution_id must be present"
        Assert-True ($executionId -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "execution_id UUIDv4 format"

        Assert-Equal $run1Env.data.execution_status "completed" "execution_status"

        # First run: initialize=1, tools/list=1, tools/call=1
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
    # Phase: Trail Inspection
    # ------------------------------------------------------------------
    Invoke-Case "public trail inspection returns execution entries in order" {
        $trailResult = Invoke-Host $workspace @(
            "trail", "--trail", $trailPath, "--execution-id", $script:executionId
        )
        $trailEnv = ConvertFrom-SingleEnvelope $trailResult "trail" "ok" 0

        Assert-Equal $trailEnv.data.execution_id $script:executionId "trail execution_id matches"

        # Prove returned trail_path is a non-empty absolute path
        Assert-True ([string]::IsNullOrEmpty($trailEnv.data.trail_path) -eq $false) "trail_path is not empty"
        Assert-True ([System.IO.Path]::IsPathRooted($trailEnv.data.trail_path)) "trail_path is absolute"

        Assert-True ($trailEnv.data.entry_count -ge 2) "trail must have at least two execution-specific entries"

        $entries = $trailEnv.data.entries

        # Intent precedes outcome
        Assert-True ($entries[0].PSObject.Properties["capability_name"] -ne $null) "intent has capability_name"
        Assert-Equal $entries[0].capability_name "fixture.ping" "intent capability_name"
        Assert-Equal ([int]$entries[0].capability_version) 1 "intent capability_version"
        Assert-Equal $entries[0].provider_identity "tethers-stdio-fixture" "intent provider_identity"
        Assert-Equal $entries[0].manifest_digest $StandingDigest "intent manifest_digest"

        # Prove the intent arguments contain message = LK-39 and path = projects/LK-39
        $intentStr = $entries[0] | ConvertTo-Json -Compress
        Assert-True ($intentStr -match "LK-39") "intent contains LK-39"
        Assert-True ($intentStr -match "projects/LK-39") "intent contains projects/LK-39"

        # Prove outcome result is structurally exactly {"echo":"LK-39"}
        Assert-True ($entries[1].PSObject.Properties["status"] -ne $null) "outcome has status"
        Assert-Equal $entries[1].status "succeeded" "outcome status succeeded"
        if ($null -ne $entries[1].PSObject.Properties["result"] -and $null -ne $entries[1].result) {
            $outcomeJson = $entries[1].result | ConvertTo-Json -Compress
            Assert-True ($outcomeJson -match '"echo":"LK-39"') "outcome result contains echo:LK-39"
        }

        # Save canonical structural representation of entries
        $script:savedEntries = $entries | ConvertTo-Json -Depth 20 -Compress
    }

    # ------------------------------------------------------------------
    # Phase: Replay
    # ------------------------------------------------------------------
    Invoke-Case "exact replay blocks duplicate effect and returns same execution ID" {
        $replayResult = Invoke-Host $workspace @(
            "run", "--config", $runtimePath, "--engine", $EnginePath,
            "--input", (Join-Path $workspace "input.json"), "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $replayEnv = ConvertFrom-SingleEnvelope $replayResult "run" "completed" 0

        Assert-Equal $replayEnv.data.execution_status "replay_blocked_completed_success" "replay status"
        Assert-Equal $replayEnv.data.execution_id $script:executionId "replay execution_id must match"

        # After replay: initialize=2, tools/list=2, tools/call=1 (no second effect)
        Assert-Equal (Get-MethodCount $marker "initialize") 2 "replay initialize count"
        Assert-Equal (Get-MethodCount $marker "tools/list") 2 "replay tools/list count"
        Assert-Equal (Get-MethodCount $marker "tools/call") 1 "replay tools/call count unchanged"

        # Trail inspection returns structurally identical entries
        $replayTrailResult = Invoke-Host $workspace @(
            "trail", "--trail", $trailPath, "--execution-id", $script:executionId
        )
        $replayTrailEnv = ConvertFrom-SingleEnvelope $replayTrailResult "trail" "ok" 0
        $replayEntries = $replayTrailEnv.data.entries | ConvertTo-Json -Depth 20 -Compress
        Assert-Equal $replayEntries $script:savedEntries "replay trail entries identical"
    }

    # ------------------------------------------------------------------
    # Phase: Non-Mutation
    # ------------------------------------------------------------------
    Invoke-Case "committed scenario sources and Cargo.lock are unchanged" {
        Assert-Equal (Get-FileHash-SHA256 $CommittedTether) $hashTetherStart "complete.tether hash unchanged"
        Assert-Equal (Get-FileHash-SHA256 $CommittedInput) $hashInputStart "input.json hash unchanged"
        Assert-Equal (Get-FileHash-SHA256 $CommittedTemplate) $hashTemplateStart "runtime.template.json hash unchanged"
        Assert-Equal (Get-FileHash-SHA256 $CommittedReadme) $hashReadmeStart "README.md hash unchanged"

        $cargoLockNow = (Get-FileHash -Path $cargoLockPath -Algorithm SHA256).Hash.ToLower()
        Assert-Equal $cargoLockNow $CargoLockHash "Cargo.lock hash unchanged"

        $gitStatusNow = Get-RepoGitStatus
        Assert-Equal $gitStatusNow $gitStatusBefore "repository git status unchanged"
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
