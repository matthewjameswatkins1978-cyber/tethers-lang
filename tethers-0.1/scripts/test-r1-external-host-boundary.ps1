[CmdletBinding()]
param(
    [string]$OcamlSwitchPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$HostRoot = Join-Path $RepoRoot 'host-rust'
$HostPath = Join-Path $HostRoot 'target/debug/tethers-reference-host.exe'
$EngineRoot = Join-Path $RepoRoot 'engine-ocaml'
$EnginePath = Join-Path $EngineRoot '_build/default/bin/tethers_mcp_main.exe'
$ScenarioRoot = Join-Path $RepoRoot 'scenarios/j14-complete-local'
$ManifestPath = Join-Path $RepoRoot 'protocol/capability-manifests/fixture-ping-standing-allow.json'
$ProviderScript = Join-Path $RepoRoot 'scripts/tethers-stdio-fixture.ps1'

$script:Assertions = 0
$script:Cases = 0

function Assert-True {
    param([bool]$Condition, [string]$Message)
    $script:Assertions++
    if (-not $Condition) { throw $Message }
}

function Assert-Equal {
    param($Actual, $Expected, [string]$Message)
    $script:Assertions++
    if ($Actual -ne $Expected) {
        throw "$Message Expected '$Expected', got '$Actual'."
    }
}

function Invoke-Case {
    param([string]$Name, [scriptblock]$Body)
    $script:Cases++
    Write-Output "TEST: $($script:Cases). $Name"
    & $Body
    Write-Output '  PASS'
}

function Invoke-Checked {
    param(
        [string]$WorkingDirectory,
        [string]$Program,
        [string[]]$Arguments,
        [string]$FailureMessage
    )
    Push-Location -LiteralPath $WorkingDirectory
    try {
        $output = @(& $Program @Arguments 2>&1 | ForEach-Object { "$($_)" })
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    if ($exitCode -ne 0) {
        $first = ($output | Select-Object -First 1)
        throw "$FailureMessage Exit code $exitCode. First output: $first"
    }
    return $output
}

function ConvertTo-CanonicalValue {
    param($Value)
    if ($null -eq $Value) { return $null }
    if ($Value -is [System.Collections.IEnumerable] -and
        $Value -isnot [string] -and
        $Value -isnot [System.Collections.IDictionary]) {
        $items = [System.Collections.Generic.List[object]]::new()
        foreach ($item in $Value) {
            [void]$items.Add((ConvertTo-CanonicalValue $item))
        }
        return ,$items.ToArray()
    }
    if ($Value -is [System.Collections.IDictionary] -or $Value -is [pscustomobject]) {
        $result = [ordered]@{}
        foreach ($property in ($Value.PSObject.Properties | Sort-Object Name)) {
            $result[$property.Name] = ConvertTo-CanonicalValue $property.Value
        }
        return $result
    }
    return $Value
}

function ConvertTo-CanonicalJson {
    param($Value)
    (ConvertTo-CanonicalValue $Value | ConvertTo-Json -Compress -Depth 50)
}

function Invoke-TethersPlan {
    param([string]$Workspace, [string]$RuntimePath, [string]$InputPath, [string]$HostData)
    $arguments = @(
        'plan',
        '--config', $RuntimePath,
        '--engine', $EnginePath,
        '--input', $InputPath,
        '--host-data-root', $HostData
    )
    Push-Location -LiteralPath $Workspace
    try {
        $lines = @(& $HostPath @arguments 2>&1 | ForEach-Object { "$($_)" })
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    Assert-Equal $exitCode 0 "Tethers plan failed: $($lines -join "`n")"
    $nonEmpty = @($lines | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    Assert-Equal $nonEmpty.Count 1 'Tethers plan must emit one JSON document'
    $envelope = $nonEmpty[0] | ConvertFrom-Json -ErrorAction Stop
    Assert-Equal $envelope.schema 'tethers.cli/1' 'CLI schema'
    Assert-Equal $envelope.command 'plan' 'CLI command'
    Assert-Equal $envelope.status 'completed' 'CLI status'
    Assert-Equal $envelope.data.schema 'tethers.plan/1' 'Plan schema'
    return $envelope
}

function Assert-PlanIsReadOnly {
    param($Envelope, [string]$Workspace)
    Assert-Equal $Envelope.data.authority.granted $false 'Plan must not grant authority'
    Assert-Equal $Envelope.data.authority.status 'not_requested' 'Plan authority status'
    Assert-Equal $Envelope.data.execution.performed $false 'Plan must not execute'
    Assert-Equal ([int]$Envelope.data.execution.provider_invocations) 0 'Tethers provider calls'
    Assert-Equal $Envelope.data.execution.policy_evaluated $false 'Plan policy state'
    Assert-Equal $Envelope.data.execution.replay_mutated $false 'Plan replay mutation'
    Assert-Equal ([int]$Envelope.data.execution.trail_execution_entries) 0 'Plan Trail entries'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $Workspace 'provider-methods.txt'))) 'Tethers provider marker exists'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $Workspace 'trail.jsonl'))) 'Tethers execution Trail exists'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $Workspace 'plan-no-trail'))) 'Plan created its no-Trail path'
}

function Invoke-ResolveOwnedExecutor {
    param(
        $PlanEnvelope,
        [ValidateSet('ALLOW', 'ASK', 'DENY')][string]$Decision,
        [ValidateSet('not_required', 'pending', 'approved')][string]$ApprovalState,
        [string]$ProviderMarker,
        [Parameter(Mandatory = $true)][string]$ExpectedCanonicalPlan
    )

    Assert-Equal $PlanEnvelope.schema 'tethers.cli/1' 'Resolve CLI envelope schema'
    Assert-Equal $PlanEnvelope.command 'plan' 'Resolve command'
    Assert-Equal $PlanEnvelope.status 'completed' 'Resolve Plan status'
    Assert-Equal $PlanEnvelope.data.schema 'tethers.plan/1' 'Resolve Plan schema'
    Assert-Equal (ConvertTo-CanonicalJson $PlanEnvelope.data) $ExpectedCanonicalPlan 'Resolve Plan evidence changed'
    if ($Decision -eq 'ALLOW' -and $ApprovalState -ne 'approved') {
        throw 'Resolve refused ALLOW without explicit approval.'
    }
    if ($Decision -eq 'ASK' -and $ApprovalState -ne 'pending') {
        throw 'Resolve ASK requires pending approval state.'
    }

    $plan = $PlanEnvelope.data.plan
    if ($null -eq $plan -or $null -eq $plan.actions -or @($plan.actions).Count -ne 1) {
        throw 'Resolve rejected malformed Plan: exactly one action is required.'
    }
    $action = @($plan.actions)[0]
    if ([string]::IsNullOrWhiteSpace([string]$action.action_id) -or
        [string]$action.capability -ne 'fixture.ping') {
        throw 'Resolve rejected malformed Plan: action identity or capability is invalid.'
    }

    $executionId = 'resolve-execution-' + [guid]::NewGuid().ToString('N')
    $result = [ordered]@{
        decision = $Decision
        action_id = [string]$action.action_id
        execution_id = $executionId
        effect_identity = 'resolve-effect-' + [guid]::NewGuid().ToString('N')
        provider_calls = 0
        outcome = 'not_executed'
    }
    if ($Decision -eq 'ALLOW') {
        Add-Content -LiteralPath $ProviderMarker -Value "$executionId|$($action.action_id)"
        $result.provider_calls = 1
        $result.outcome = 'success'
    } elseif ($Decision -eq 'ASK') {
        $result.outcome = 'awaiting_approval'
    } else {
        $result.outcome = 'denied'
    }
    return [pscustomobject]$result
}

foreach ($path in @($ScenarioRoot, $ManifestPath, $ProviderScript)) {
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Required R1 asset is missing: $path"
    }
}

if (-not (Test-Path -LiteralPath $HostPath -PathType Leaf)) {
    Invoke-Checked -WorkingDirectory $HostRoot -Program 'cargo' -Arguments @('build', '--locked') -FailureMessage 'Could not build the current reference Host'
}

if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    $prepareArgs = @('-NoProfile', '-File', (Join-Path $RepoRoot '..\scripts\prepare-current-engine.ps1'))
    if (-not [string]::IsNullOrWhiteSpace($OcamlSwitchPath)) {
        $prepareArgs += @('-OcamlSwitchPath', $OcamlSwitchPath)
    }
    Invoke-Checked -WorkingDirectory $RepoRoot -Program 'pwsh.exe' -Arguments $prepareArgs -FailureMessage 'Could not prepare the current OCaml engine'
}

$temp = Join-Path ([IO.Path]::GetTempPath()) ('tethers-r1-external-host-' + [guid]::NewGuid().ToString('N'))
$workspace = Join-Path $temp 'workspace'
$manifests = Join-Path $workspace 'manifests'
$tethers = Join-Path $workspace 'tethers'
$scripts = Join-Path $workspace 'scripts'
$hostData = Join-Path $workspace 'host-data'
$resolveMarker = Join-Path $workspace 'resolve-provider-methods.txt'

try {
    New-Item -ItemType Directory -Force -Path $manifests, $tethers, $scripts, $hostData | Out-Null
    Copy-Item -LiteralPath (Join-Path $ScenarioRoot 'tethers/complete.tether') -Destination (Join-Path $tethers 'complete.tether')
    Copy-Item -LiteralPath (Join-Path $ScenarioRoot 'input.json') -Destination (Join-Path $workspace 'input.json')
    Copy-Item -LiteralPath $ManifestPath -Destination (Join-Path $manifests 'fixture-ping-standing-allow.json')
    Copy-Item -LiteralPath $ProviderScript -Destination (Join-Path $scripts 'tethers-stdio-fixture.ps1')

    $runtime = Get-Content -LiteralPath (Join-Path $ScenarioRoot 'runtime.template.json') -Raw | ConvertFrom-Json
    $runtime.providers[0].transport.args[2] = Join-Path $scripts 'tethers-stdio-fixture.ps1'
    $runtime.providers[0].transport.args[5] = Join-Path $workspace 'provider-methods.txt'
    $runtime.providers[0].capabilities[0].manifest_path = 'manifests/fixture-ping-standing-allow.json'
    $runtimePath = Join-Path $workspace 'runtime.json'
    [IO.File]::WriteAllText($runtimePath, ($runtime | ConvertTo-Json -Depth 30), [Text.UTF8Encoding]::new($false))
    $inputPath = Join-Path $workspace 'input.json'

    Invoke-Case 'identical external-host inputs produce identical Plan evidence' {
        $first = Invoke-TethersPlan -Workspace $workspace -RuntimePath $runtimePath -InputPath $inputPath -HostData $hostData
        Assert-PlanIsReadOnly -Envelope $first -Workspace $workspace
        $firstCanonical = ConvertTo-CanonicalJson $first.data

        # Resolve-owned job/lease state is intentionally local to the harness;
        # it is not supplied as a Tethers Fact and cannot change the Plan.
        $jobState = [ordered]@{ state = 'leased'; approval = 'not_requested' }
        Assert-Equal $jobState.state 'leased' 'Resolve-only job state setup'
        $second = Invoke-TethersPlan -Workspace $workspace -RuntimePath $runtimePath -InputPath $inputPath -HostData $hostData
        Assert-PlanIsReadOnly -Envelope $second -Workspace $workspace
        Assert-Equal (ConvertTo-CanonicalJson $second.data) $firstCanonical 'Plan changed after Resolve-only state change'
    }

    $planEnvelope = Invoke-TethersPlan -Workspace $workspace -RuntimePath $runtimePath -InputPath $inputPath -HostData $hostData

    Invoke-Case 'Resolve ALLOW owns the only provider invocation' {
        $expectedCanonicalPlan = ConvertTo-CanonicalJson $planEnvelope.data
        $result = Invoke-ResolveOwnedExecutor -PlanEnvelope $planEnvelope -Decision ALLOW -ApprovalState approved -ProviderMarker $resolveMarker -ExpectedCanonicalPlan $expectedCanonicalPlan
        Assert-Equal $result.provider_calls 1 'Resolve ALLOW provider call count'
        Assert-Equal $result.outcome 'success' 'Resolve ALLOW outcome'
        Assert-True ($result.execution_id -ne $result.action_id) 'Resolve execution identity must be distinct from Tethers ActionId'
        Assert-True ($result.effect_identity -ne $result.action_id) 'Resolve effect identity must be distinct from Tethers ActionId'
        Assert-True ($result.effect_identity -ne $result.execution_id) 'Resolve effect identity must be distinct from execution identity'
        Assert-Equal @((Get-Content -LiteralPath $resolveMarker)).Count 1 'Resolve provider marker count after ALLOW'
    }

    Invoke-Case 'Resolve ASK before approval performs no provider call' {
        $askMarker = Join-Path $workspace 'resolve-ask-marker.txt'
        $result = Invoke-ResolveOwnedExecutor -PlanEnvelope $planEnvelope -Decision ASK -ApprovalState pending -ProviderMarker $askMarker -ExpectedCanonicalPlan (ConvertTo-CanonicalJson $planEnvelope.data)
        Assert-Equal $result.provider_calls 0 'Resolve ASK provider call count'
        Assert-Equal $result.outcome 'awaiting_approval' 'Resolve ASK outcome'
        Assert-True (-not (Test-Path -LiteralPath $askMarker)) 'Resolve ASK invoked provider before approval'
    }

    Invoke-Case 'Resolve ASK resumes only after explicit approval' {
        $askApprovalMarker = Join-Path $workspace 'resolve-ask-approved-marker.txt'
        $expectedCanonicalPlan = ConvertTo-CanonicalJson $planEnvelope.data
        $pending = Invoke-ResolveOwnedExecutor -PlanEnvelope $planEnvelope -Decision ASK -ApprovalState pending -ProviderMarker $askApprovalMarker -ExpectedCanonicalPlan $expectedCanonicalPlan
        $approved = Invoke-ResolveOwnedExecutor -PlanEnvelope $planEnvelope -Decision ALLOW -ApprovalState approved -ProviderMarker $askApprovalMarker -ExpectedCanonicalPlan $expectedCanonicalPlan
        Assert-Equal $pending.provider_calls 0 'Pending ASK provider call count'
        Assert-Equal $approved.provider_calls 1 'Approved ASK provider call count'
        Assert-Equal @((Get-Content -LiteralPath $askApprovalMarker)).Count 1 'Approved ASK provider marker count'
    }

    Invoke-Case 'Resolve DENY performs no provider call' {
        $denyMarker = Join-Path $workspace 'resolve-deny-marker.txt'
        $result = Invoke-ResolveOwnedExecutor -PlanEnvelope $planEnvelope -Decision DENY -ApprovalState not_required -ProviderMarker $denyMarker -ExpectedCanonicalPlan (ConvertTo-CanonicalJson $planEnvelope.data)
        Assert-Equal $result.provider_calls 0 'Resolve DENY provider call count'
        Assert-Equal $result.outcome 'denied' 'Resolve DENY outcome'
        Assert-True (-not (Test-Path -LiteralPath $denyMarker)) 'Resolve DENY invoked provider'
    }

    Invoke-Case 'malformed Plan is rejected before external execution' {
        $malformed = $planEnvelope | ConvertTo-Json -Depth 50 | ConvertFrom-Json
        $malformed.data.plan.actions[0].action_id = $null
        $malformed.data.plan.actions[0].arguments.message = 'tampered'
        $malformed.data.evaluation_id = 'tampered-evaluation'
        $malformedMarker = Join-Path $workspace 'resolve-malformed-marker.txt'
        $expectedCanonicalPlan = ConvertTo-CanonicalJson $planEnvelope.data
        $rejected = $false
        try {
            Invoke-ResolveOwnedExecutor -PlanEnvelope $malformed -Decision ALLOW -ApprovalState approved -ProviderMarker $malformedMarker -ExpectedCanonicalPlan $expectedCanonicalPlan | Out-Null
        } catch {
            $rejected = $true
        }
        Assert-True $rejected 'Malformed Plan was accepted'
        Assert-True (-not (Test-Path -LiteralPath $malformedMarker)) 'Malformed Plan reached provider'
    }

    [ordered]@{
        schema = 'tethers.external-host-proof/1'
        plan_schema = $planEnvelope.data.schema
        plan_action_count = @($planEnvelope.data.plan.actions).Count
        tethers_provider_calls = [int]$planEnvelope.data.execution.provider_invocations
        tethers_execution_performed = [bool]$planEnvelope.data.execution.performed
        allow_provider_calls = 1
        ask_provider_calls = 0
        deny_provider_calls = 0
        malformed_plan = 'rejected_before_provider'
        action_identity = [string]$planEnvelope.data.plan.actions[0].action_id
        execution_identity_owner = 'external_host'
    } | ConvertTo-Json -Compress
}
finally {
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Output "TOTAL: $script:Cases cases, $script:Cases passed, 0 failed"
Write-Output "ASSERTIONS: $script:Assertions"
