Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$DebugHostPath = Join-Path $HostDir "target\debug\tethers-reference-host.exe"
$ReleaseHostPath = Join-Path $HostDir "target\release\tethers-reference-host.exe"
$EnginePath = Join-Path $RepoRoot "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"
$StandingManifest = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping-standing-allow.json"
$AskManifest = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping.json"
$FixtureProvider = Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1"
$StandingDigest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da"
$AskDigest = "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"
$CargoLockHash = "d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602"

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
    Write-Output "TEST: M$($script:caseCount.ToString('00')) $Name"
    & $Body
    $script:passedCount++
    Write-Output "  PASS"
}

function Invoke-Host {
    param([string]$WorkingDirectory, [string[]]$ArgumentList)
    Push-Location $WorkingDirectory
    try {
        $output = @(& $DebugHostPath @ArgumentList 2>&1)
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

function Invoke-DebugCommand {
    param([string[]]$ArgumentList)
    $output = @(& $DebugHostPath @ArgumentList 2>&1)
    $exitCode = $LASTEXITCODE
    [pscustomobject]@{
        ExitCode = $exitCode
        Stdout   = ($output -join "`n")
    }
}

function Invoke-ReleaseCommand {
    param([string[]]$ArgumentList)
    $output = @(& $ReleaseHostPath @ArgumentList 2>&1)
    $exitCode = $LASTEXITCODE
    [pscustomobject]@{
        ExitCode = $exitCode
        Stdout   = ($output -join "`n")
    }
}

function ConvertFrom-SingleEnvelope {
    param([Parameter(Mandatory = $true)]$Result,
          [string]$ExpectedCommand,
          [AllowNull()][string]$ExpectedStatus,
          [int]$ExpectedExit)
    $lines = @($Result.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    Assert-Equal $lines.Count 1 "stdout must contain exactly one JSON document"
    $envelope = $lines[0] | ConvertFrom-Json -ErrorAction Stop
    Assert-Equal $envelope.schema "tethers.cli/1" "schema mismatch"
    if ($null -ne $ExpectedCommand) {
        Assert-Equal $envelope.command $ExpectedCommand "command mismatch"
    }
    if ($null -ne $ExpectedStatus) {
        Assert-Equal $envelope.status $ExpectedStatus "status mismatch"
    }
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

function Get-TotalMethodCount {
    param([string]$Marker)
    if (-not (Test-Path -LiteralPath $Marker -PathType Leaf)) { return 0 }
    return @((Get-Content -LiteralPath $Marker)).Count
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

function Get-FileHash-SHA256 {
    param([string]$Path)
    (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLower()
}

function New-StandingConfig {
    param([string]$WorkspaceRoot, [string]$ProviderMode, [string]$Digest = $StandingDigest, [string]$ManifestFile = "fixture-ping-standing-allow.json", [string]$Policy = "allow", [string]$TetherFile = "standing.tether")

    $manifestsDir = Join-Path $WorkspaceRoot "manifests"
    $tethersDir = Join-Path $WorkspaceRoot "tethers"
    $scriptsDir = Join-Path $WorkspaceRoot "scripts"
    New-Item -ItemType Directory -Force -Path $manifestsDir, $tethersDir, $scriptsDir | Out-Null

    $manifestSource = if ($ManifestFile -eq "fixture-ping-standing-allow.json") { $StandingManifest } else { $AskManifest }
    Copy-Item -LiteralPath $manifestSource -Destination (Join-Path $manifestsDir $ManifestFile)
    Copy-Item -LiteralPath $FixtureProvider -Destination (Join-Path $scriptsDir "tethers-stdio-fixture.ps1")

    $marker = Join-Path $WorkspaceRoot "provider-methods.txt"
    $providerArgs = @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", $ProviderMode, "-MarkerFile", $marker)

    $capability = [ordered]@{
        name = "fixture.ping"
        version = 1
        manifest_path = "manifests/$ManifestFile"
        pinned_digest = $Digest
    }
    if ($Digest -eq $StandingDigest -and $ManifestFile -eq "fixture-ping-standing-allow.json") {
        $capability.scope_binding = [ordered]@{
            kind = "path_prefix"
            argument_json_pointer = "/path"
        }
    }

    $config = [ordered]@{
        format_version = "0.1"
        tether_set = [ordered]@{
            id = "fixture.j14b"
            version = "1"
            tethers = @([ordered]@{ id = "fixture-j14b"; version = "1"; source_path = "tethers/$TetherFile" })
            capability_requirements = @([ordered]@{ name = "fixture.ping"; version = 1; reason = "J14B negative matrix" })
        }
        providers = @([ordered]@{
            id = "tethers-stdio-fixture"
            display_name = "Tethers Stdio Fixture"
            transport = [ordered]@{
                kind = "stdio"
                command = "pwsh.exe"
                args = $providerArgs
                protocol_version = "2025-11-25"
            }
            capabilities = @($capability)
        })
        policy = [ordered]@{
            default = "deny"
            rules = @([ordered]@{ name = "fixture.ping"; version = 1; decision = $Policy })
        }
    }
    $configPath = Join-Path $WorkspaceRoot "runtime.json"
    $config | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $configPath -Encoding utf8NoBOM

    $source = if ($TetherFile -eq "ask.tether") {
@'
tether "Fixture J14B Ask"

anchor
    coding.task_completed

when
    project.type is "software"

do
    fixture.ping
        message: anchor.task
'@
    } else {
@'
tether "Fixture J14B"

anchor
    coding.task_completed

when
    project.type is "software"
    and task.changed_files greater_than 0

do
    fixture.ping
        message: anchor.task
        path: anchor.path
'@
    }
    Write-Utf8NoBom (Join-Path $tethersDir $TetherFile) $source

    [pscustomobject]@{
        Root = $WorkspaceRoot
        Config = $configPath
        Marker = $marker
    }
}

function Write-RunInput {
    param([string]$Path, [string]$EvaluationId = "eval_j14b_001", [string]$TetherId = "fixture-j14b", [string]$EventId = "evt_j14b_001", [string]$EventName = "coding.task_completed")
    $input = [ordered]@{
        format_version = "1"
        evaluation_id = $EvaluationId
        tether = [ordered]@{ id = $TetherId; version = "1" }
        event = [ordered]@{
            id = $EventId
            name = $EventName
            data = [ordered]@{ project = "lantern-keeper"; task = "LK-39"; path = "projects/LK-39" }
        }
        facts = [ordered]@{ "project.type" = "software"; "task.changed_files" = 3 }
    }
    $input | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $Path -Encoding utf8NoBOM
}

function Invoke-Run {
    param([Parameter(Mandatory = $true)]$Workspace, [string]$InputPath, [string]$TrailPath, [string]$ReplayRoot)
    Invoke-Host -WorkingDirectory $Workspace.Root -ArgumentList @(
        "run", "--config", $Workspace.Config, "--engine", $EnginePath,
        "--input", $InputPath, "--trail", $TrailPath, "--host-data-root", $ReplayRoot
    )
}

function Invoke-Check {
    param([Parameter(Mandatory = $true)]$Workspace)
    Invoke-Host -WorkingDirectory $Workspace.Root -ArgumentList @(
        "check", "--config", $Workspace.Config, "--engine", $EnginePath
    )
}

# ------------------------------------------------------------------
# Pre-flight
# ------------------------------------------------------------------
if (-not (Test-Path -LiteralPath $DebugHostPath -PathType Leaf)) {
    throw "Debug host executable is missing: $DebugHostPath. Run cargo build first."
}
if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "OCaml engine executable is missing: $EnginePath."
}
if (-not (Test-Path -LiteralPath $StandingManifest -PathType Leaf) -or -not (Test-Path -LiteralPath $AskManifest -PathType Leaf)) {
    throw "Required fixture manifest is missing."
}

$cargoLockPath = Join-Path $HostDir "Cargo.lock"
$cargoLockActual = (Get-FileHash -Path $cargoLockPath -Algorithm SHA256).Hash.ToLower()
if ($cargoLockActual -ne $CargoLockHash) {
    throw "Cargo.lock hash mismatch: expected $CargoLockHash, got $cargoLockActual"
}

Push-Location $RepoRoot
try {
    $s = & git status --porcelain=v1 --untracked-files=all 2>&1
    $gitStatusBefore = ($s -join "`n").Trim()
}
finally {
    Pop-Location
}

# ------------------------------------------------------------------
# Build workspace - Unicode + space temp path
# ------------------------------------------------------------------
$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("Tethers J14B caf" + [char]0x00E9 + " " + [guid]::NewGuid().ToString())
Assert-True ($TempRoot -match " ") "temp root must contain at least one space"
Assert-True ($TempRoot -cmatch "[^\x00-\x7F]") "temp root must contain at least one non-ASCII character"
New-Item -ItemType Directory -Path $TempRoot | Out-Null

try {
    # ==================================================================
    # M01: Malformed manifest
    # ==================================================================
    Invoke-Case "M01 malformed manifest" {
        $root = Join-Path $TempRoot "m01"
        $manifestsDir = Join-Path $root "manifests"
        $tethersDir = Join-Path $root "tethers"
        $scriptsDir = Join-Path $root "scripts"
        New-Item -ItemType Directory -Force -Path $manifestsDir, $tethersDir, $scriptsDir | Out-Null

        Copy-Item -LiteralPath $FixtureProvider -Destination (Join-Path $scriptsDir "tethers-stdio-fixture.ps1")

        $source = @'
tether "Fixture J14B M01"

anchor
    coding.task_completed

when
    project.type is "software"

do
    fixture.ping
        message: anchor.task
        path: anchor.path
'@
        Write-Utf8NoBom (Join-Path $tethersDir "m01.tether") $source

        $malformedManifestPath = Join-Path $manifestsDir "malformed.json"
        Write-Utf8NoBom $malformedManifestPath '{not valid json'

        $config = [ordered]@{
            format_version = "0.1"
            tether_set = [ordered]@{
                id = "fixture.j14b.m01"
                version = "1"
                tethers = @([ordered]@{ id = "fixture-j14b-m01"; version = "1"; source_path = "tethers/m01.tether" })
                capability_requirements = @([ordered]@{ name = "fixture.ping"; version = 1; reason = "J14B M01" })
            }
            providers = @([ordered]@{
                id = "tethers-stdio-fixture"
                display_name = "Tethers Stdio Fixture"
                transport = [ordered]@{
                    kind = "stdio"
                    command = "pwsh.exe"
                    args = @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", "run-success")
                    protocol_version = "2025-11-25"
                }
                capabilities = @([ordered]@{
                    name = "fixture.ping"
                    version = 1
                    manifest_path = "manifests/malformed.json"
                    pinned_digest = $StandingDigest
                })
            })
            policy = [ordered]@{
                default = "deny"
                rules = @([ordered]@{ name = "fixture.ping"; version = 1; decision = "allow" })
            }
        }
        $configPath = Join-Path $root "runtime.json"
        $config | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $configPath -Encoding utf8NoBOM

        $result = Invoke-Host -WorkingDirectory $root -ArgumentList @("check", "--config", $configPath, "--engine", $EnginePath)
        $envelope = ConvertFrom-SingleEnvelope $result "check" "invalid_data" 3
        Assert-Equal $envelope.error.code "RUNTIME_PREPARE_FAILED" "M01 machine code"
    }

    # ==================================================================
    # M02: Unavailable provider (missing tool)
    # ==================================================================
    Invoke-Case "M02 unavailable provider" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m02") -ProviderMode "missing-tool" -TetherFile "m02.tether"
        $result = Invoke-Check $workspace
        $envelope = ConvertFrom-SingleEnvelope $result "check" "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_CAPABILITY_UNAVAILABLE" "M02 machine code"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "M02 zero tools/call"
    }

    # ==================================================================
    # M03: Ask
    # ==================================================================
    Invoke-Case "M03 Ask" {
        $root = Join-Path $TempRoot "m03"
        $manifestsDir = Join-Path $root "manifests"
        $tethersDir = Join-Path $root "tethers"
        $scriptsDir = Join-Path $root "scripts"
        New-Item -ItemType Directory -Force -Path $manifestsDir, $tethersDir, $scriptsDir | Out-Null

        Copy-Item -LiteralPath $AskManifest -Destination (Join-Path $manifestsDir "fixture-ping.json")
        Copy-Item -LiteralPath $FixtureProvider -Destination (Join-Path $scriptsDir "tethers-stdio-fixture.ps1")

        $marker = Join-Path $root "provider-methods.txt"

        $source = @'
tether "Fixture J14B M03"

anchor
    coding.task_completed

when
    project.type is "software"

do
    fixture.ping
        message: anchor.task
'@
        Write-Utf8NoBom (Join-Path $tethersDir "ask.tether") $source

        $config = [ordered]@{
            format_version = "0.1"
            tether_set = [ordered]@{
                id = "fixture.j14b.m03"
                version = "1"
                tethers = @([ordered]@{ id = "fixture-j14b-m03"; version = "1"; source_path = "tethers/ask.tether" })
                capability_requirements = @([ordered]@{ name = "fixture.ping"; version = 1; reason = "J14B M03" })
            }
            providers = @([ordered]@{
                id = "tethers-stdio-fixture"
                display_name = "Tethers Stdio Fixture"
                transport = [ordered]@{
                    kind = "stdio"
                    command = "pwsh.exe"
                    args = @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", "record-methods", "-MarkerFile", $marker)
                    protocol_version = "2025-11-25"
                }
                capabilities = @([ordered]@{
                    name = "fixture.ping"
                    version = 1
                    manifest_path = "manifests/fixture-ping.json"
                    pinned_digest = $AskDigest
                })
            })
            policy = [ordered]@{
                default = "deny"
                rules = @([ordered]@{ name = "fixture.ping"; version = 1; decision = "allow" })
            }
        }
        $configPath = Join-Path $root "runtime.json"
        $config | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $configPath -Encoding utf8NoBOM

        $replayRoot = Join-Path $root "replay"
        Provision-ReplayRoot $replayRoot $root
        $inputPath = Join-Path $root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b-m03"
        $trailPath = Join-Path $root "trail.jsonl"

        $result = Invoke-Host -WorkingDirectory $root -ArgumentList @(
            "run", "--config", $configPath, "--engine", $EnginePath,
            "--input", $inputPath, "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $envelope = ConvertFrom-SingleEnvelope $result "run" "approval_required" 5

        Assert-True ($result.Stdout -notmatch 'approval_id') "M03 no public approval ID"
        Assert-True ($null -eq $envelope.data.PSObject.Properties["execution_id"] -or $null -eq $envelope.data.execution_id) "M03 no execution ID"
        Assert-Equal (Get-MethodCount $marker "tools/call") 0 "M03 zero calls"
        Assert-True ($null -eq $envelope.data.PSObject.Properties["result_anchor"] -or $null -eq $envelope.data.result_anchor) "M03 no Result Anchor"
        Assert-True ((Get-Content -Raw -LiteralPath $trailPath) -match '"approval_requested"') "M03 approval_requested in trail"
    }

    # ==================================================================
    # M04: Deny
    # ==================================================================
    Invoke-Case "M04 Deny" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m04") -ProviderMode "run-success" -Policy "deny" -TetherFile "m04.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $envelope = ConvertFrom-SingleEnvelope $result "run" "denied" 0

        Assert-True ($null -eq $envelope.data.PSObject.Properties["execution_id"] -or $null -eq $envelope.data.execution_id) "M04 no execution ID"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "M04 zero calls"
        Assert-True ($null -eq $envelope.data.PSObject.Properties["result_anchor"] -or $null -eq $envelope.data.result_anchor) "M04 no Result Anchor"
    }

    # ==================================================================
    # M05: Stale pinned digest
    # ==================================================================
    Invoke-Case "M05 stale pinned digest" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m05") -ProviderMode "run-success" -TetherFile "m05.tether" -Digest "sha256:0000000000000000000000000000000000000000000000000000000000000001"
        $result = Invoke-Check $workspace
        $envelope = ConvertFrom-SingleEnvelope $result "check" "invalid_data" 3
        Assert-Equal $envelope.error.code "RUNTIME_PREPARE_FAILED" "M05 machine code"
        Assert-Equal (Get-TotalMethodCount $workspace.Marker) 0 "M05 no provider launch"
    }

    # ==================================================================
    # M07: Executor failure
    # ==================================================================
    Invoke-Case "M07 executor failure" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m07") -ProviderMode "run-explicit-error" -TetherFile "m07.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m07"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $envelope = ConvertFrom-SingleEnvelope $result "run" "failed" 6

        Assert-Equal $envelope.error.code "ACTION_FAILED" "M07 machine code"
        Assert-Equal $envelope.data.execution_status "failed" "M07 execution status"
        Assert-True ($null -ne $envelope.data.execution_id) "M07 execution ID present"
        Assert-True ($envelope.data.execution_id -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "M07 execution ID format"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M07 exactly one call"

        Assert-True (Test-Path -LiteralPath $trailPath) "M07 trail file exists"
        $trailContent = Get-Content -Raw -LiteralPath $trailPath
        $trailLines = @(Get-Content -LiteralPath $trailPath | Where-Object { $_.Trim() -ne "" })
        Assert-True ($trailLines.Count -gt 0) "M07 trail has entries"
        Assert-True ($trailContent -match '"failed"') "M07 trail contains failed outcome"
    }

    # ==================================================================
    # M08: Invalid provider output
    # ==================================================================
    Invoke-Case "M08 invalid provider output" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m08") -ProviderMode "run-invalid-output" -TetherFile "m08.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m08"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $envelope = ConvertFrom-SingleEnvelope $result "run" "failed" 6

        Assert-Equal $envelope.error.code "ACTION_FAILED" "M08 machine code"
        Assert-Equal $envelope.data.execution_status "failed" "M08 execution status"
        Assert-True ($null -ne $envelope.data.execution_id) "M08 execution ID present"
        Assert-True ($envelope.data.execution_id -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "M08 execution ID format"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M08 exactly one call"

        Assert-True (Test-Path -LiteralPath $trailPath) "M08 trail file exists"
        $trailContent = Get-Content -Raw -LiteralPath $trailPath
        Assert-True ($trailContent -match '"failed"') "M08 trail contains failed outcome"
    }

    # ==================================================================
    # M09: Timeout (uncertain)
    # ==================================================================
    Invoke-Case "M09 uncertain timeout" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m09") -ProviderMode "run-hang-call" -TetherFile "m09.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m09"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $envelope = ConvertFrom-SingleEnvelope $result "run" "uncertain" 7
        Assert-Equal $envelope.error.code "ACTION_UNCERTAIN" "M09 machine code"
        Assert-Equal $envelope.data.execution_status "uncertain" "M09 execution status"
        Assert-True ($null -ne $envelope.data.execution_id) "M09 execution ID present"
        Assert-True ($envelope.data.execution_id -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "M09 execution ID format"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M09 exactly one call"

        Assert-True (Test-Path -LiteralPath $trailPath) "M09 trail file exists"
        $trailContent = Get-Content -Raw -LiteralPath $trailPath
        Assert-True ($trailContent -match '"uncertain"') "M09 trail contains uncertain outcome"
    }

    # ==================================================================
    # M10: Duplicate replay
    # ==================================================================
    Invoke-Case "M10 duplicate replay" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m10") -ProviderMode "run-success" -TetherFile "m10.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m10"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $run1 = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $env1 = ConvertFrom-SingleEnvelope $run1 "run" "completed" 0
        $execId1 = $env1.data.execution_id
        Assert-True ($null -ne $execId1) "M10 first execution ID present"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M10 first run one call"

        $savedEntries = Get-Content -LiteralPath $trailPath | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.PSObject.Properties["execution_id"] -and $_.execution_id -eq $execId1 } | ConvertTo-Json -Depth 20 -Compress

        $run2 = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $env2 = ConvertFrom-SingleEnvelope $run2 "run" "completed" 0
        Assert-Equal $env2.data.execution_status "replay_blocked_completed_success" "M10 replay status"
        Assert-Equal $env2.data.execution_id $execId1 "M10 same execution ID"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M10 total one call across both runs"

        $replayEntries = Get-Content -LiteralPath $trailPath | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.PSObject.Properties["execution_id"] -and $_.execution_id -eq $execId1 } | ConvertTo-Json -Depth 20 -Compress
        Assert-Equal $replayEntries $savedEntries "M10 structurally identical filtered trail entries"
    }

    # ==================================================================
    # M11: Causal depth beyond eight
    # ==================================================================
    Invoke-Case "M11 causal depth beyond eight" {
        # Build debug binary
        Push-Location $HostDir
        try {
            Write-Host "Building debug host for M11 ..."
            $null = & cargo build 2>&1
            if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
        }
        finally {
            Pop-Location
        }

        $trailPath = Join-Path $TempRoot "m11-trail.jsonl"

        $debugResult = Invoke-DebugCommand @("event-admission-trail-probe", "causal-depth", $trailPath)
        Assert-Equal $debugResult.ExitCode 0 "M11 debug probe exit code"
        Assert-True ($debugResult.Stdout.Trim() -ne "") "M11 debug probe produced output"
        $debugJson = $debugResult.Stdout.Trim() | ConvertFrom-Json

        Assert-Equal $debugJson.kind "event_admission_probe" "M11 probe kind"
        Assert-Equal $debugJson.scenario "causal-depth" "M11 scenario"
        Assert-True ($null -eq $debugJson.PSObject.Properties["follow_up_evaluations"] -or $debugJson.follow_up_evaluations.Count -eq 0) "M11 no follow up evaluations for gen 9"

        $rejection = $debugJson.event_admission_rejection
        Assert-Equal $rejection.kind "causal_depth_exceeded" "M11 rejection kind"
        Assert-Equal $rejection.event_id "evt/deep" "M11 rejection event ID"
        Assert-Equal ([int]$rejection.generation) 9 "M11 generation 9"
        Assert-Equal ([int]$rejection.maximum_generation) 8 "M11 maximum generation 8"
        Assert-Equal $rejection.processing "stopped" "M11 processing stopped"

        $remaining = $debugJson.remaining_queue_event_ids
        Assert-True ($remaining.Count -eq 1) "M11 one later sibling not evaluated"
        Assert-Equal $remaining[0] "evt/later" "M11 later sibling remains"

        Assert-True (Test-Path -LiteralPath $trailPath -PathType Leaf) "M11 trail file exists"
        $trailContent = Get-Content -Raw -LiteralPath $trailPath
        Assert-True ($trailContent -match '"event_admitted".*"evt/root"') "M11 initial generation-zero admission in trail"
        Assert-True ($trailContent -match '"event_rejected".*"evt/deep"') "M11 generation-nine rejection in trail"
    }

    # ==================================================================
    # M11b: Release binary does not expose debug command
    # ==================================================================
    Invoke-Case "M11b release binary rejects debug command" {
        if (-not (Test-Path -LiteralPath $ReleaseHostPath -PathType Leaf)) {
            throw "Release host executable is missing: $ReleaseHostPath."
        }
        $trailPath = Join-Path $TempRoot "m11b-trail.jsonl"
        $releaseResult = Invoke-ReleaseCommand @("event-admission-trail-probe", "causal-depth", $trailPath)
        $releaseEnv = ConvertFrom-SingleEnvelope $releaseResult "event-admission-trail-probe" "unavailable" 4
        Assert-Equal $releaseEnv.error.code "DEBUG_ONLY" "M11b release rejects debug command"
    }

    Write-Output ""
    Write-Output "============================================"
    Write-Output "TOTAL: $script:caseCount cases, $script:passedCount passed, 0 failed"
    Write-Output "ASSERTIONS: $script:assertionCount"
    Write-Output "============================================"

    # Verify repository git status unchanged
    Push-Location $RepoRoot
    try {
        $s = & git status --porcelain=v1 --untracked-files=all 2>&1
        $gitStatusNow = ($s -join "`n").Trim()
    }
    finally {
        Pop-Location
    }
    $gitStatusBeforeNormalized = ($gitStatusBefore -replace "`r`n", "`n").Trim()
    $gitStatusNowNormalized = ($gitStatusNow -replace "`r`n", "`n").Trim()
    if ($gitStatusBeforeNormalized -ne $gitStatusNowNormalized) {
        throw "Repository git status changed during test execution. Before: '$gitStatusBeforeNormalized', After: '$gitStatusNowNormalized'"
    }
}
finally {
    Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
