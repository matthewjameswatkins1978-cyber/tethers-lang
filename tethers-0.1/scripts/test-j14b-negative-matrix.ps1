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
$CargoLockHash = "46dd99a4287976d9fe0d2327619a9b389f46aa6b00b7993d49345843508ca023"

$script:caseCount = 0
$script:passedCount = 0
$script:assertionCount = 0
$script:observedRows = [System.Collections.Generic.List[string]]::new()
$script:rustTestNames = [System.Collections.Generic.List[string]]::new()

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

function Assert-Contains {
    param([string]$Haystack, [string]$Needle, [string]$Message)
    $script:assertionCount++
    if (-not $Haystack.Contains($Needle)) { throw "$Message Expected '$Needle' in '$Haystack'." }
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

function Get-CargoLockHash {
    # Cargo.lock is tracked with LF. Normalize only its checkout conversion so
    # this content-integrity sentinel remains stable under Windows core.autocrlf.
    $path = Join-Path $HostDir "Cargo.lock"
    $text = [System.Text.Encoding]::UTF8.GetString([System.IO.File]::ReadAllBytes($path))
    $canonical = $text.Replace("`r`n", "`n").Replace("`r", "`n")
    $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes($canonical)
    ([System.Security.Cryptography.SHA256]::HashData($bytes) | ForEach-Object { $_.ToString("x2") }) -join ""
}

function Assert-RepositoryIntegrity {
    param([string]$Label)
    Assert-Equal (Get-CargoLockHash) $CargoLockHash "$Label Cargo.lock hash unchanged"
    $statusNow = Get-RepoGitStatus
    Assert-Equal $statusNow $script:gitStatusBefore "$Label repository git status unchanged"
}

function Invoke-MatrixRow {
    param([Parameter(Mandatory = $true)][string]$Id, [Parameter(Mandatory = $true)][string]$Name, [scriptblock]$Body)
    $script:caseCount++
    $script:observedRows.Add($Id)
    Write-Output "TEST: $Id $Name"
    & $Body
    $script:passedCount++
    Write-Output "  PASS"
    Assert-RepositoryIntegrity "after $Id"
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

function Invoke-HostWithTimeout {
    param(
        [string]$WorkingDirectory,
        [string[]]$ArgumentList,
        [int]$TimeoutMs = 15000
    )
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $DebugHostPath
    $psi.WorkingDirectory = $WorkingDirectory
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    foreach ($argument in $ArgumentList) {
        $null = $psi.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::Start($psi)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $exited = $process.WaitForExit($TimeoutMs)
    $sw.Stop()
    if (-not $exited) {
        try { $process.Kill($true) } catch {}
        $process.WaitForExit()
        throw "host process exceeded harness timeout ${TimeoutMs}ms"
    }

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    [pscustomobject]@{
        ExitCode = $process.ExitCode
        Stdout   = $stdout
        Stderr   = $stderr
        ElapsedMs = $sw.ElapsedMilliseconds
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
    if ($Result.PSObject.Properties["ExitCode"]) {
        Assert-Equal $Result.ExitCode $ExpectedExit "process exit code mismatch"
    }
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

function Get-WorkspaceTreeSnapshot {
    param([string]$Root)
    if (-not (Test-Path -LiteralPath $Root)) { return @() }
    Get-ChildItem -Recurse -LiteralPath $Root | ForEach-Object { $_.FullName } | Sort-Object
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

function Assert-TrailExecutionOutcome {
    param([string]$TrailPath, [string]$ExecutionId, [string]$ExpectedTerminalStatus)
    Assert-True (Test-Path -LiteralPath $TrailPath) "Trail file must exist"
    $lines = @(Get-Content -LiteralPath $TrailPath | Where-Object { $_.Trim() -ne "" })
    $entries = $lines | ForEach-Object { $_ | ConvertFrom-Json }
    $filtered = @($entries | Where-Object { $_.PSObject.Properties["execution_id"] -and $_.execution_id -eq $ExecutionId })
    Assert-True ($filtered.Count -ge 2) "expected at least intent and terminal outcome for $ExecutionId"

        $intent = $filtered[0]
        Assert-True ($null -ne $intent.PSObject.Properties["capability_name"]) "intent must have capability_name"
    Assert-Equal $intent.capability_name "fixture.ping" "intent capability_name"
    Assert-Equal ([int]$intent.capability_version) 1 "intent capability_version"

        $terminal = $filtered | Where-Object { $null -ne $_.PSObject.Properties["status"] } | Select-Object -Last 1
        Assert-True ($null -ne $terminal) "terminal outcome record required"
    Assert-Equal $terminal.status $ExpectedTerminalStatus "terminal outcome status"

    $terminalCount = @($filtered | Where-Object { $_.PSObject.Properties["status"] }).Count
    Assert-Equal $terminalCount 1 "exactly one terminal outcome for $ExecutionId"
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

$script:gitStatusBefore = Get-RepoGitStatus
if ((Get-CargoLockHash) -ne $CargoLockHash) {
    throw "Cargo.lock hash mismatch before start"
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
    # Internal focused Rust proofs (M06 and Result Anchor kinds)
    # ==================================================================
    $rustEnvBefore = $env:RUSTUP_AUTO_INSTALL
    $env:RUSTUP_AUTO_INSTALL = 0
    try {
        $rustOutput = @(& rustup run 1.89.0 cargo test `
          --manifest-path (Join-Path $HostDir "Cargo.toml") `
          --locked `
          j14b_ `
          -- `
          --nocapture 2>&1)
        $rustExit = $LASTEXITCODE
    }
    finally {
        if ($null -ne $rustEnvBefore) {
            $env:RUSTUP_AUTO_INSTALL = $rustEnvBefore
        } else {
            Remove-Item Env:RUSTUP_AUTO_INSTALL -ErrorAction SilentlyContinue
        }
    }
    $rustText = $rustOutput -join "`n"
    Assert-Equal $rustExit 0 "focused Rust j14b_ tests failed`n$rustText"
    $script:rustTestNames.Add("j14b_post_admission_intent_failure_retains_id")
    $script:rustTestNames.Add("j14b_failed_and_uncertain_result_anchor_kinds")
    foreach ($name in $script:rustTestNames) {
        Assert-Contains $rustText $name "focused Rust test name missing: $name"
    }
    Assert-Contains $rustText "0 failed" "focused Rust tests reported failures"
    # The P1 application seam moved the shared test owner from the binary
    # crate root to the library's application module. Preserve the exact two
    # named proofs while accepting their library-owned test path.
    $rustTestCount = ([regex]::Matches($rustText, "test application::tests::j14b_")).Count
    Assert-Equal $rustTestCount 2 "focused Rust j14b_ test count"
    Write-Output "INTERNAL: focused Rust j14b_ tests passed ($rustTestCount tests)"
    Assert-RepositoryIntegrity "after internal Rust proofs"

    # ==================================================================
    # M01: Malformed manifest
    # ==================================================================
    Invoke-MatrixRow -Id "M01" -Name "malformed manifest" {
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

        $marker = Join-Path $root "provider-methods.txt"
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
                    args = @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", "run-success", "-MarkerFile", $marker)
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

        $treeBefore = Get-WorkspaceTreeSnapshot $root
        $treeBeforeStr = ($treeBefore -join "`n")

        $result = Invoke-Host -WorkingDirectory $root -ArgumentList @("check", "--config", $configPath, "--engine", $EnginePath)
        $envelope = ConvertFrom-SingleEnvelope $result "check" "invalid_data" 3
        Assert-Equal $envelope.error.code "RUNTIME_PREPARE_FAILED" "M01 machine code"

        Assert-Equal (Get-MethodCount $marker "initialize") 0 "M01 zero initialize"
        Assert-Equal (Get-MethodCount $marker "tools/list") 0 "M01 zero tools/list"
        Assert-Equal (Get-MethodCount $marker "tools/call") 0 "M01 zero tools/call"
        Assert-True (-not (Test-Path -LiteralPath $marker)) "M01 provider marker not created"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $root "trail.jsonl") -PathType Leaf)) "M01 no Trail file"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $root "replay") -PathType Container)) "M01 no replay directory"
        $treeAfter = Get-WorkspaceTreeSnapshot $root
        Assert-Equal ($treeAfter -join "`n") $treeBeforeStr "M01 workspace tree unchanged"
    }

    # ==================================================================
    # M02: Unavailable provider (missing tool)
    # ==================================================================
    Invoke-MatrixRow -Id "M02" -Name "unavailable provider" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m02") -ProviderMode "missing-tool" -TetherFile "m02.tether"
        $result = Invoke-Check $workspace
        $envelope = ConvertFrom-SingleEnvelope $result "check" "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_CAPABILITY_UNAVAILABLE" "M02 machine code"

        Assert-True ($null -eq $envelope.data.PSObject.Properties["execution_id"] -or $null -eq $envelope.data.execution_id) "M02 no execution ID"
        $anchorProp = $envelope.data.PSObject.Properties["result_anchor"]
        Assert-True ($null -eq $anchorProp -or $null -eq $anchorProp.Value) "M02 no Result Anchor"

        Assert-Equal @($envelope.data.providers).Count 1 "M02 exactly one provider evidence object"
        Assert-Equal $envelope.data.providers[0].status "unavailable" "M02 provider status unavailable"
        Assert-Equal @($envelope.data.providers[0].capabilities).Count 1 "M02 exactly one capability evidence object"
        Assert-Equal $envelope.data.providers[0].capabilities[0].status "unavailable" "M02 capability status unavailable"

        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M02 initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M02 tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "M02 zero tools/call"
    }

    # ==================================================================
    # M03: Ask
    # ==================================================================
    Invoke-MatrixRow -Id "M03" -Name "Ask" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m03") -ProviderMode "record-methods" -ManifestFile "fixture-ping.json" -Digest $AskDigest -TetherFile "ask.tether" -Policy "allow"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m03"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-Host -WorkingDirectory $workspace.Root -ArgumentList @(
            "run", "--config", $workspace.Config, "--engine", $EnginePath,
            "--input", $inputPath, "--trail", $trailPath, "--host-data-root", $replayRoot
        )
        $envelope = ConvertFrom-SingleEnvelope $result "run" "approval_required" 5

        Assert-True ($result.Stdout -notmatch 'approval_id') "M03 no public approval ID"
        Assert-True ($null -eq $envelope.data.PSObject.Properties["execution_id"] -or $null -eq $envelope.data.execution_id) "M03 no execution ID"
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M03 initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M03 tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "M03 zero tools/call"
        Assert-True ($null -eq $envelope.data.PSObject.Properties["result_anchor"] -or $null -eq $envelope.data.result_anchor) "M03 no Result Anchor"
        Assert-True ((Get-Content -Raw -LiteralPath $trailPath) -match '"approval_requested"') "M03 approval_requested in trail"
    }

    # ==================================================================
    # M04: Deny
    # ==================================================================
    Invoke-MatrixRow -Id "M04" -Name "Deny" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m04") -ProviderMode "run-success" -Policy "deny" -TetherFile "m04.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m04"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $envelope = ConvertFrom-SingleEnvelope $result "run" "denied" 0

        Assert-True ($null -eq $envelope.data.PSObject.Properties["execution_id"] -or $null -eq $envelope.data.execution_id) "M04 no execution ID"
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M04 initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M04 tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "M04 zero tools/call"
        Assert-True ($null -eq $envelope.data.PSObject.Properties["result_anchor"] -or $null -eq $envelope.data.result_anchor) "M04 no Result Anchor"
    }

    # ==================================================================
    # M05: Stale pinned digest
    # ==================================================================
    Invoke-MatrixRow -Id "M05" -Name "stale pinned digest" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m05") -ProviderMode "run-success" -TetherFile "m05.tether" -Digest "sha256:0000000000000000000000000000000000000000000000000000000000000001"
        $treeBefore = Get-WorkspaceTreeSnapshot $workspace.Root
        $treeBeforeStr = ($treeBefore -join "`n")

        $result = Invoke-Check $workspace
        $envelope = ConvertFrom-SingleEnvelope $result "check" "invalid_data" 3
        Assert-Equal $envelope.error.code "RUNTIME_PREPARE_FAILED" "M05 machine code"
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 0 "M05 zero initialize"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 0 "M05 zero tools/list"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "M05 zero tools/call"
        Assert-True (-not (Test-Path -LiteralPath $workspace.Marker)) "M05 provider marker not created"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $workspace.Root "trail.jsonl") -PathType Leaf)) "M05 no Trail file"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $workspace.Root "replay") -PathType Container)) "M05 no replay directory"
        $treeAfter = Get-WorkspaceTreeSnapshot $workspace.Root
        Assert-Equal ($treeAfter -join "`n") $treeBeforeStr "M05 workspace tree unchanged"
    }

    # ==================================================================
    # M06: Post-admission durable intent failure (internal Rust proof)
    # ==================================================================
    Invoke-MatrixRow -Id "M06" -Name "post-admission durable intent failure" {
        # M06 is proved by the focused Rust test j14b_post_admission_intent_failure_retains_id.
        # The internal proof run above succeeded, so this public row asserts it was observed.
        Assert-Contains ($script:rustTestNames -join "`n") "j14b_post_admission_intent_failure_retains_id" "M06 internal proof missing"
    }

    # ==================================================================
    # M07: Executor failure
    # ==================================================================
    Invoke-MatrixRow -Id "M07" -Name "executor failure" {
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
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M07 initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M07 tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M07 exactly one call"

        Assert-TrailExecutionOutcome $trailPath $envelope.data.execution_id "failed"
    }

    # ==================================================================
    # M08: Invalid provider output
    # ==================================================================
    Invoke-MatrixRow -Id "M08" -Name "invalid provider output" {
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
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M08 initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M08 tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M08 exactly one call"

        Assert-TrailExecutionOutcome $trailPath $envelope.data.execution_id "failed"
    }

    # ==================================================================
    # M09: Uncertain timeout
    # ==================================================================
    Invoke-MatrixRow -Id "M09" -Name "uncertain timeout" {
        $workspace = New-StandingConfig -WorkspaceRoot (Join-Path $TempRoot "m09") -ProviderMode "run-hang-call" -TetherFile "m09.tether"
        $replayRoot = Join-Path $workspace.Root "replay"
        Provision-ReplayRoot $replayRoot $workspace.Root
        $inputPath = Join-Path $workspace.Root "input.json"
        Write-RunInput $inputPath -TetherId "fixture-j14b" -EvaluationId "eval_j14b_m09"
        $trailPath = Join-Path $workspace.Root "trail.jsonl"

        $result = Invoke-HostWithTimeout -WorkingDirectory $workspace.Root -ArgumentList @(
            "run", "--config", $workspace.Config, "--engine", $EnginePath,
            "--input", $inputPath, "--trail", $trailPath, "--host-data-root", $replayRoot
        ) -TimeoutMs 15000

        $elapsed = $result.ElapsedMs
        Assert-True ($elapsed -ge 4000) "M09 elapsed >= 4000ms (got $elapsed ms)"
        Assert-True ($elapsed -le 12000) "M09 elapsed <= 12000ms (got $elapsed ms)"

        $envelope = ConvertFrom-SingleEnvelope $result "run" "uncertain" 7
        Assert-Equal $envelope.error.code "ACTION_UNCERTAIN" "M09 machine code"
        Assert-Equal $envelope.data.execution_status "uncertain" "M09 execution status"
        Assert-True ($null -ne $envelope.data.execution_id) "M09 execution ID present"
        Assert-True ($envelope.data.execution_id -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "M09 execution ID format"
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M09 initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M09 tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M09 exactly one call"

        Assert-TrailExecutionOutcome $trailPath $envelope.data.execution_id "uncertain"
    }

    # ==================================================================
    # M10: Duplicate replay
    # ==================================================================
    Invoke-MatrixRow -Id "M10" -Name "duplicate replay" {
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
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 1 "M10 first run initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 1 "M10 first run tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M10 first run one call"

        $savedEntries = Get-Content -LiteralPath $trailPath | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.PSObject.Properties["execution_id"] -and $_.execution_id -eq $execId1 } | ConvertTo-Json -Depth 20 -Compress

        $run2 = Invoke-Run $workspace $inputPath $trailPath $replayRoot
        $env2 = ConvertFrom-SingleEnvelope $run2 "run" "completed" 0
        Assert-Equal $env2.data.execution_status "replay_blocked_completed_success" "M10 replay status"
        Assert-Equal $env2.data.execution_id $execId1 "M10 same execution ID"
        Assert-Equal (Get-MethodCount $workspace.Marker "initialize") 2 "M10 replay initialize count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/list") 2 "M10 replay tools/list count"
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 1 "M10 total one call across both runs"

        $replayEntries = Get-Content -LiteralPath $trailPath | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.PSObject.Properties["execution_id"] -and $_.execution_id -eq $execId1 } | ConvertTo-Json -Depth 20 -Compress
        Assert-Equal $replayEntries $savedEntries "M10 structurally identical filtered trail entries"
    }

    # ==================================================================
    # M11: Causal depth beyond eight
    # ==================================================================
    Invoke-MatrixRow -Id "M11" -Name "causal depth beyond eight" {
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
        $trailLines = @(Get-Content -LiteralPath $trailPath | Where-Object { $_.Trim() -ne "" })
        Assert-Equal $trailLines.Count 2 "M11 exactly two Trail records"
        $trailRecords = $trailLines | ForEach-Object { $_ | ConvertFrom-Json }

        $record1 = $trailRecords[0]
        Assert-Equal $record1.kind "event_admitted" "M11 record1 kind"
        Assert-Equal $record1.event_id "evt/root" "M11 record1 event_id"
        Assert-Equal $record1.source "external" "M11 record1 source"
        Assert-Equal ([int]$record1.generation) 0 "M11 record1 generation"
        Assert-Equal $record1.processing "continued" "M11 record1 processing"
        Assert-True ($null -eq $record1.PSObject.Properties["reason_code"]) "M11 record1 no reason_code"
        Assert-True ($null -eq $record1.PSObject.Properties["maximum_generation"]) "M11 record1 no maximum_generation"

        $record2 = $trailRecords[1]
        Assert-Equal $record2.kind "event_rejected" "M11 record2 kind"
        Assert-Equal $record2.event_id "evt/deep" "M11 record2 event_id"
        Assert-Equal $record2.source "result_anchor" "M11 record2 source"
        Assert-Equal ([int]$record2.generation) 9 "M11 record2 generation"
        Assert-Equal $record2.processing "stopped" "M11 record2 processing"
        Assert-Equal $record2.reason_code "causal_depth_exceeded" "M11 record2 reason_code"
        Assert-Equal ([int]$record2.maximum_generation) 8 "M11 record2 maximum_generation"

        foreach ($record in $trailRecords) {
            Assert-True ($record.event_id -ne "evt/later") "M11 no evt/later Trail record"
        }

        # Release binary rejects the debug command
        if (-not (Test-Path -LiteralPath $ReleaseHostPath -PathType Leaf)) {
            Push-Location $HostDir
            try {
                Write-Host "Building release host for M11 ..."
                $null = & cargo build --release 2>&1
                if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }
            }
            finally {
                Pop-Location
            }
        }
        $m11Trail = Join-Path $TempRoot "m11-release-trail.jsonl"
        $releaseResult = Invoke-ReleaseCommand @("event-admission-trail-probe", "causal-depth", $m11Trail)
        $releaseEnv = ConvertFrom-SingleEnvelope $releaseResult "event-admission-trail-probe" "unavailable" 4
        Assert-Equal $releaseEnv.error.code "DEBUG_ONLY" "M11 release rejects debug command"
    }

    # ------------------------------------------------------------------
    # Final row and assertion totals
    # ------------------------------------------------------------------
    Assert-Equal $script:observedRows.Count 11 "row count"
    $expectedRows = @("M01", "M02", "M03", "M04", "M05", "M06", "M07", "M08", "M09", "M10", "M11")
    for ($i = 0; $i -lt $expectedRows.Count; $i++) {
        Assert-Equal $script:observedRows[$i] $expectedRows[$i] "row sequence at index $i"
    }
    $uniqueRows = $script:observedRows | Sort-Object -Unique
    Assert-Equal $uniqueRows.Count 11 "no duplicate rows"

    Write-Output ""
    Write-Output "============================================"
    Write-Output "TOTAL: 11 rows, 11 passed, 0 failed"
    Write-Output "ASSERTIONS: $script:assertionCount"
    Write-Output "============================================"

    Assert-RepositoryIntegrity "final"
}
finally {
    Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
