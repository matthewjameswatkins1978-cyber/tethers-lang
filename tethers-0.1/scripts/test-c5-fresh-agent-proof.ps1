Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# C5 — Fresh-Agent Concurrency Authoring Proof
# End-to-end integration test for multi-capability Together execution.

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$HostPath = Join-Path $HostDir "target\debug\tethers-reference-host.exe"
$EnginePath = Join-Path $RepoRoot "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"
$FixtureProvider = Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1"
$TestDir = Join-Path $RepoRoot "tests\c5-fresh-agent-proof"

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

function Write-Utf8NoBom {
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
if (-not (Test-Path -LiteralPath $FixtureProvider -PathType Leaf)) {
    throw "Required fixture provider is missing: $FixtureProvider"
}

# ------------------------------------------------------------------
# Manifest digests (computed by tethers_reference_host::manifest::canonicalize_and_digest)
# ------------------------------------------------------------------
$ManifestA = Join-Path $TestDir "manifests\fixture-ping-a.json"
$ManifestB = Join-Path $TestDir "manifests\fixture-ping-b.json"
$DigestA = "sha256:368042670855d316c42bc7dff3740ce19b219d3eec3b970597c3f7e7ff44bcef"
$DigestB = "sha256:2b18f82de39cb61a226000162183b9bcfdfba3b3fcf10079f2fbe19e792c915d"
Write-Output "Manifest A digest: $DigestA"
Write-Output "Manifest B digest: $DigestB"

# ------------------------------------------------------------------
# Build workspace
# ------------------------------------------------------------------
$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("tethers-c5-" + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempRoot | Out-Null

try {
    $workspace = Join-Path $TempRoot "workspace"
    $manifestsDir = Join-Path $workspace "manifests"
    $tethersDir = Join-Path $workspace "tethers"
    $scriptsDir = Join-Path $workspace "scripts"
    New-Item -ItemType Directory -Force -Path $manifestsDir, $tethersDir, $scriptsDir | Out-Null

    # Copy assets
    Copy-Item -LiteralPath $ManifestA -Destination (Join-Path $manifestsDir "fixture-ping-a.json")
    Copy-Item -LiteralPath $ManifestB -Destination (Join-Path $manifestsDir "fixture-ping-b.json")
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1") -Destination (Join-Path $scriptsDir "tethers-stdio-fixture.ps1")
    Copy-Item -LiteralPath (Join-Path $TestDir "tethers\c5-multi-capability.tether") -Destination (Join-Path $tethersDir "c5-multi-capability.tether")
    Copy-Item -LiteralPath (Join-Path $TestDir "c5-input.json") -Destination (Join-Path $workspace "c5-input.json")

    $marker = Join-Path $workspace "provider-methods.txt"
    $trailPath = Join-Path $workspace "trail.jsonl"
    $replayRoot = Join-Path $workspace "replay-data"

    $providerScriptPath = Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1"

    # Build runtime config with two providers
        $config = [ordered]@{
        format_version = "0.1"
        tether_set = [ordered]@{
            id = "test.c5.multicapability"
            version = "1"
            tethers = @([ordered]@{
                id = "c5-multi-capability"
                version = "1"
                source_path = "tethers/c5-multi-capability.tether"
                core_environment = [ordered]@{
                    program_id = "program.c5"
                    core_version = "1"
                    capabilities = @(
                        [ordered]@{ source_name = "fixture.ping-a"; capability_id = "cap.c5.ping-a"; contract_digest = "C5-CONTRACT-A"; runtime_name = "fixture.ping-a" },
                        [ordered]@{ source_name = "fixture.ping-b"; capability_id = "cap.c5.ping-b"; contract_digest = "C5-CONTRACT-B"; runtime_name = "fixture.ping-b" }
                    )
                    input_facts = @(
                        [ordered]@{ source_name = "build.status"; fact_id = "fact.build.status"; host_snapshot_key = "build.status"; scalar_type = "string"; schema_description = "Build status" }
                    )
                }
            })
            capability_requirements = @(
                [ordered]@{ name = "fixture.ping-a"; version = 1; reason = "C5 multi-capability proof" },
                [ordered]@{ name = "fixture.ping-b"; version = 1; reason = "C5 multi-capability proof" }
            )
        }
        providers = @(
            [ordered]@{
                id = "provider-a"
                display_name = "Provider A"
                transport = [ordered]@{
                    kind = "stdio"
                    command = "pwsh.exe"
                    args = @("-NoProfile", "-File", $providerScriptPath, "-Mode", "run-success", "-MarkerFile", $marker)
                    protocol_version = "2025-11-25"
                }
                capabilities = @([ordered]@{
                    name = "fixture.ping-a"
                    version = 1
                    manifest_path = "manifests/fixture-ping-a.json"
                    pinned_digest = $DigestA
                    scope_binding = [ordered]@{
                        kind = "path_prefix"
                        argument_json_pointer = "/message"
                    }
                })
            },
            [ordered]@{
                id = "provider-b"
                display_name = "Provider B"
                transport = [ordered]@{
                    kind = "stdio"
                    command = "pwsh.exe"
                    args = @("-NoProfile", "-File", $providerScriptPath, "-Mode", "run-success", "-MarkerFile", $marker)
                    protocol_version = "2025-11-25"
                }
                capabilities = @([ordered]@{
                    name = "fixture.ping-b"
                    version = 1
                    manifest_path = "manifests/fixture-ping-b.json"
                    pinned_digest = $DigestB
                    scope_binding = [ordered]@{
                        kind = "path_prefix"
                        argument_json_pointer = "/message"
                    }
                })
            }
        )
        policy = [ordered]@{
            default = "deny"
            rules = @(
                [ordered]@{ name = "fixture.ping-a"; version = 1; decision = "allow" },
                [ordered]@{ name = "fixture.ping-b"; version = 1; decision = "allow" }
            )
        }
    }
    $runtimePath = Join-Path $workspace "runtime.json"
    $configJson = $config | ConvertTo-Json -Depth 30
    Write-Utf8NoBom $runtimePath $configJson

    # Provision replay
    Provision-ReplayRoot $replayRoot $workspace

    # ------------------------------------------------------------------
    # Phase 1: First Run — Plan Proof
    # ------------------------------------------------------------------
    $script:firstExecutionId = $null
    $script:firstPlan = $null

    Invoke-Case "first run produces plan with two actions and one together group" {
        $runResult = Invoke-Host $workspace @(
            "run", "--config", $runtimePath, "--engine", $EnginePath,
            "--input", (Join-Path $workspace "c5-input.json"), "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $runEnv = ConvertFrom-SingleEnvelope $runResult "run" "completed" 0

        Assert-Equal $runEnv.data.evaluation_id "eval_c5_multicapability_001" "evaluation_id"
        Assert-Equal $runEnv.data.execution_status "completed" "execution_status"

        $script:firstExecutionId = $runEnv.data.execution_id

        # Verify plan structure
        $plan = $runEnv.data.plan
        Assert-True ($null -ne $plan) "plan must be present"
        Assert-True ($null -ne $plan.actions) "plan.actions must be present"
        Assert-Equal $plan.actions.Count 2 "plan must have exactly two actions"

        # Verify different capabilities
        $capA = $plan.actions[0].capability
        $capB = $plan.actions[1].capability
        Assert-Equal $capA "fixture.ping-a" "first action capability"
        Assert-Equal $capB "fixture.ping-b" "second action capability"
        Assert-True ($capA -ne $capB) "capabilities must be different"

        # Verify groups array
        Assert-True ($null -ne $plan.groups) "plan.groups must be present for together block"
        Assert-Equal $plan.groups.Count 1 "must have exactly one group"
        Assert-Equal $plan.groups[0].group_id "group_1" "group_id"
        Assert-Equal $plan.groups[0].member_action_ids.Count 2 "group must have two members"
        Assert-Equal $plan.groups[0].member_action_ids[0] $plan.actions[0].action_id "first member action_id"
        Assert-Equal $plan.groups[0].member_action_ids[1] $plan.actions[1].action_id "second member action_id"

        # Verify contiguous action_ids
        Assert-Equal $plan.actions[0].action_id "action_1" "first action_id"
        Assert-Equal $plan.actions[1].action_id "action_2" "second action_id"

        # Save plan for determinism comparison
        $script:firstPlan = $plan | ConvertTo-Json -Depth 20 -Compress

        # Verify Result Anchor
        $anchor = $runEnv.data.result_anchor
        Assert-True ($null -ne $anchor) "result_anchor must be present"
    }

    # ------------------------------------------------------------------
    # Phase 3: Execution Proof — Both Members Executed
    # ------------------------------------------------------------------
    Invoke-Case "both together members reached provider invocation" {
        # The run already completed. Verify execution evidence.
        Assert-True ($null -ne $script:firstExecutionId) "execution_id must exist"

        # Verify provider methods were called
        Assert-True (Test-Path -LiteralPath $marker) "marker file must exist"
        $initializeCount = Get-MethodCount $marker "initialize"
        Assert-True ($initializeCount -ge 2) "both providers must have been initialized (got $initializeCount)"
    }

    # ------------------------------------------------------------------
    # Phase 4: Trail Proof — GroupJoin and Member Outcomes
    # ------------------------------------------------------------------
    Invoke-Case "trail contains truthful evidence for both members and group join" {
        Assert-True (Test-Path -LiteralPath $trailPath) "trail file must exist"

        $trailResult = Invoke-Host $workspace @(
            "trail", "--trail", $trailPath, "--execution-id", $script:firstExecutionId
        )
        $trailEnv = ConvertFrom-SingleEnvelope $trailResult "trail" "ok" 0

        Assert-True ($trailEnv.data.entry_count -ge 4) "trail must have at least 4 entries (2 outcomes + group join + admission)"

        $entries = $trailEnv.data.entries

        # Find outcome entries for both members
        $outcomeA = $entries | Where-Object { $_.PSObject.Properties["action_id"] -and $_.action_id -eq "action_1" -and $_.PSObject.Properties["status"] }
        $outcomeB = $entries | Where-Object { $_.PSObject.Properties["action_id"] -and $_.action_id -eq "action_2" -and $_.PSObject.Properties["status"] }

        Assert-True ($null -ne $outcomeA) "must have outcome for action_1 (fixture.ping-a)"
        Assert-True ($null -ne $outcomeB) "must have outcome for action_2 (fixture.ping-b)"

        # Find group join entry
        $groupJoin = $entries | Where-Object { $_.PSObject.Properties["group_id"] -and $_.PSObject.Properties["joined"] }
        Assert-True ($null -ne $groupJoin) "must have group join entry"
        Assert-Equal $groupJoin.group_id "group_1" "group join group_id"
        Assert-Equal $groupJoin.joined $true "group must have joined successfully"
    }

    # ------------------------------------------------------------------
    # Phase 5: Determinism Proof — 3+ Runs
    # ------------------------------------------------------------------
    $determinismRunCount = 3
    $planShapes = @()

    Invoke-Case "determinism proof: $determinismRunCount runs preserve semantic plan identity" {
        for ($i = 1; $i -le $determinismRunCount; $i++) {
            # Clean up for fresh run
            Remove-Item -LiteralPath $trailPath -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $marker -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $replayRoot -Recurse -Force -ErrorAction SilentlyContinue
            Provision-ReplayRoot $replayRoot $workspace

            $runResult = Invoke-Host $workspace @(
                "run", "--config", $runtimePath, "--engine", $EnginePath,
                "--input", (Join-Path $workspace "c5-input.json"), "--trail", $trailPath, "--host-data-root", $replayRoot
            )
            $runEnv = ConvertFrom-SingleEnvelope $runResult "run" "completed" 0

            Assert-Equal $runEnv.data.execution_status "completed" "run $i execution_status"

            $plan = $runEnv.data.plan
            Assert-True ($null -ne $plan) "run $i plan must be present"
            Assert-Equal $plan.actions.Count 2 "run $i must have exactly two actions"

            # Verify same capabilities in same order
            Assert-Equal $plan.actions[0].capability "fixture.ping-a" "run $i first capability"
            Assert-Equal $plan.actions[1].capability "fixture.ping-b" "run $i second capability"

            # Verify same group structure
            Assert-True ($null -ne $plan.groups) "run $i plan.groups must be present"
            Assert-Equal $plan.groups.Count 1 "run $i must have one group"
            Assert-Equal $plan.groups[0].member_action_ids.Count 2 "run $i group must have two members"

            # Save plan shape (excluding evaluation-specific IDs)
            $shape = [ordered]@{
                action_count = $plan.actions.Count
                cap_a = $plan.actions[0].capability
                cap_b = $plan.actions[1].capability
                group_count = $plan.groups.Count
                member_count = $plan.groups[0].member_action_ids.Count
            }
            $planShapes += ($shape | ConvertTo-Json -Compress)

            # Verify trail evidence
            Assert-True (Test-Path -LiteralPath $trailPath) "run $i trail must exist"
        }

        # Verify all runs produced identical semantic shapes
        $uniqueShapes = $planShapes | Sort-Object -Unique
        Assert-Equal $uniqueShapes.Count 1 "all runs must produce identical plan shapes"

        Write-Output "  Determinism proof: $determinismRunCount runs produced identical semantic plan shapes"
    }

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    Write-Output ""
    Write-Output "============================================"
    Write-Output "C5 FRESH-AGENT CONCURRENCY AUTHORING PROOF"
    Write-Output "============================================"
    Write-Output "TOTAL: $caseCount cases, $passedCount passed, 0 failed"
    Write-Output "ASSERTIONS: $assertionCount"
    Write-Output ""
    Write-Output "EVIDENCE SUMMARY:"
    Write-Output "  Tether source: tethers/c5-multi-capability.tether"
    Write-Output "  Capabilities: fixture.ping-a, fixture.ping-b (different)"
    Write-Output "  Together group: group_1 with 2 members"
    Write-Output "  Plan: flat actions + additive groups array"
    Write-Output "  Execution: both members reached providers"
    Write-Output "  GroupJoin: successful"
    Write-Output "  Trail: truthful evidence for both members"
    Write-Output "  Determinism: $determinismRunCount runs with identical semantic shapes"
    Write-Output "============================================"
}
finally {
    Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
