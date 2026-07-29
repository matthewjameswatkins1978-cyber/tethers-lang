Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$HostPath = Join-Path $HostDir "target\debug\tethers-reference-host.exe"
$EnginePath = Join-Path $RepoRoot "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"
$StandingManifest = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping-standing-allow.json"
$AskManifest = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping.json"
$FixtureProvider = Join-Path $PSScriptRoot "tethers-stdio-fixture.ps1"
$StandingDigest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da"
$AskDigest = "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"
$script:caseCount = 0
$script:passedCount = 0

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function Assert-Equal {
    param($Actual, $Expected, [string]$Message)
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
        Stdout = ($output -join "`n")
    }
}

function ConvertFrom-SingleEnvelope {
    param([Parameter(Mandatory = $true)]$Result, [string]$ExpectedStatus, [int]$ExpectedExit)
    $lines = @($Result.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    Assert-Equal $lines.Count 1 "stdout must contain exactly one JSON document."
    $envelope = $lines[0] | ConvertFrom-Json -ErrorAction Stop
    Assert-Equal $envelope.schema "tethers.cli/1" "schema mismatch."
    Assert-Equal $envelope.command "run" "command mismatch."
    Assert-Equal $envelope.status $ExpectedStatus "status mismatch."
    Assert-Equal ([int]$envelope.exit_code) $ExpectedExit "embedded exit code mismatch."
    Assert-Equal $Result.ExitCode $ExpectedExit "process exit code mismatch."
    return $envelope
}

function Write-Text {
    param([string]$Path, [string]$Text)
    [System.IO.File]::WriteAllText($Path, $Text, [System.Text.UTF8Encoding]::new($false))
}

function New-RunWorkspace {
    param(
        [string]$Name,
        [ValidateSet("allow", "deny")][string]$Policy = "allow",
        [switch]$Ask,
        [string]$ProviderMode = ""
    )
    $root = Join-Path $script:TempRoot $Name
    $manifestDir = Join-Path $root "manifests"
    $tetherDir = Join-Path $root "tethers"
    $scriptDir = Join-Path $root "scripts"
    New-Item -ItemType Directory -Force -Path $manifestDir, $tetherDir, $scriptDir | Out-Null

    $manifestSource = if ($Ask) { $AskManifest } else { $StandingManifest }
    $manifestFile = if ($Ask) { "fixture-ping.json" } else { "fixture-ping-standing-allow.json" }
    $digest = if ($Ask) { $AskDigest } else { $StandingDigest }
    Copy-Item -LiteralPath $manifestSource -Destination (Join-Path $manifestDir $manifestFile)
    Copy-Item -LiteralPath $FixtureProvider -Destination (Join-Path $scriptDir "tethers-stdio-fixture.ps1")

    $source = if ($Ask) {
@'
tether "Fixture public run Ask"

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
tether "Fixture public run"

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
    Write-Text (Join-Path $tetherDir "run.tether") $source

    $marker = Join-Path $root "provider-methods.txt"
    $mode = if ($ProviderMode) { $ProviderMode } elseif ($Ask) { "record-methods" } else { "run-success" }
    $providerArgs = @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", $mode, "-MarkerFile", $marker)
    $capability = [ordered]@{
        name = "fixture.ping"
        version = 1
        manifest_path = "manifests/$manifestFile"
        pinned_digest = $digest
    }
    if (-not $Ask) {
        $capability.scope_binding = [ordered]@{
            kind = "path_prefix"
            argument_json_pointer = "/path"
        }
    }
    $config = [ordered]@{
        format_version = "0.1"
        tether_set = [ordered]@{
            id = "fixture.j13b.run"
            version = "1"
            tethers = @([ordered]@{ id = "fixture-run"; version = "1"; source_path = "tethers/run.tether" })
            capability_requirements = @([ordered]@{ name = "fixture.ping"; version = 1; reason = "J13B public run acceptance" })
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
    $configPath = Join-Path $root "runtime.json"
    $config | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $configPath -Encoding utf8NoBOM
    [pscustomobject]@{ Root = $root; Config = $configPath; Marker = $marker }
}

function Write-RunInput {
    param([string]$Path, [string]$EvaluationId = "eval_demo_001", [string]$EventName = "coding.task_completed")
    $input = [ordered]@{
        format_version = "1"
        evaluation_id = $EvaluationId
        tether = [ordered]@{ id = "fixture-run"; version = "1" }
        event = [ordered]@{
            id = "evt_demo_001"
            name = $EventName
            data = [ordered]@{ project = "lantern-keeper"; task = "LK-39"; path = "projects/LK-39" }
        }
        facts = [ordered]@{ "project.type" = "software"; "task.changed_files" = 3 }
    }
    $input | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $Path -Encoding utf8NoBOM
}

function Invoke-Run {
    param([Parameter(Mandatory = $true)]$Workspace, [string]$InputPath, [string]$TrailPath, [string]$ReplayRoot, [switch]$Reordered)
    $arguments = if ($Reordered) {
        @("run", "--host-data-root", $ReplayRoot, "--trail", $TrailPath, "--input", $InputPath, "--engine", $EnginePath, "--config", $Workspace.Config)
    } else {
        @("run", "--config", $Workspace.Config, "--engine", $EnginePath, "--input", $InputPath, "--trail", $TrailPath, "--host-data-root", $ReplayRoot)
    }
    Invoke-Host -WorkingDirectory $Workspace.Root -ArgumentList $arguments
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

function Invoke-InterruptedRun {
    param([Parameter(Mandatory = $true)]$Workspace, [string]$InputPath, [string]$TrailPath, [string]$ReplayRoot)
    $controllerPath = Join-Path $Workspace.Root "ctrl-c-controller.ps1"
    $resultPath = Join-Path $Workspace.Root "ctrl-c-result.json"
@'
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$source = @"
using System;
using System.Runtime.InteropServices;
public static class J13BRunCtrlController
{
    private delegate bool HandlerRoutine(uint ctrlType);
    private static readonly HandlerRoutine Handler = Ignore;

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetConsoleCtrlHandler(HandlerRoutine handler, bool add);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool FreeConsole();
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool AllocConsole();
    [DllImport("kernel32.dll")]
    private static extern IntPtr GetConsoleWindow();
    [DllImport("user32.dll")]
    private static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool GenerateConsoleCtrlEvent(uint ctrlEvent, uint processGroupId);

    private static bool Ignore(uint ctrlType) { return true; }
    public static bool PrepareIsolatedConsole()
    {
        FreeConsole();
        if (!AllocConsole()) return false;
        IntPtr window = GetConsoleWindow();
        if (window != IntPtr.Zero) ShowWindow(window, 0);
        return SetConsoleCtrlHandler(Handler, true);
    }
}
"@

try {
    Add-Type -TypeDefinition $source
    if (-not [J13BRunCtrlController]::PrepareIsolatedConsole()) {
        throw "Controller could not prepare its isolated Ctrl+C console."
    }
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $env:TETHERS_J13B_CTRL_HOST
    $psi.WorkingDirectory = $env:TETHERS_J13B_CTRL_CWD
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $false
    foreach ($argument in @(
        "run", "--config", $env:TETHERS_J13B_CTRL_CONFIG,
        "--engine", $env:TETHERS_J13B_CTRL_ENGINE,
        "--input", $env:TETHERS_J13B_CTRL_INPUT,
        "--trail", $env:TETHERS_J13B_CTRL_TRAIL,
        "--host-data-root", $env:TETHERS_J13B_CTRL_REPLAY_ROOT
    )) { $null = $psi.ArgumentList.Add($argument) }
    $process = [System.Diagnostics.Process]::Start($psi)
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $wait = [System.Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath $env:TETHERS_J13B_CTRL_PROVIDER_MARKER -PathType Leaf)) {
        if ($wait.ElapsedMilliseconds -ge 8000) { throw "Timed out waiting for the blocked provider." }
        Start-Sleep -Milliseconds 50
    }
    $interrupt = [System.Diagnostics.Stopwatch]::StartNew()
    if (-not [J13BRunCtrlController]::GenerateConsoleCtrlEvent(0, 0)) {
        throw "GenerateConsoleCtrlEvent(CTRL_C_EVENT) failed."
    }
    if (-not $process.WaitForExit(8000)) {
        $process.Kill($true)
        $process.WaitForExit()
        throw "Host did not exit within 8000ms after CTRL_C_EVENT."
    }
    $interrupt.Stop()
    [ordered]@{
        exit_code = $process.ExitCode
        stdout_base64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($stdoutTask.GetAwaiter().GetResult()))
        stderr_base64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($stderrTask.GetAwaiter().GetResult()))
        interrupt_ms = $interrupt.ElapsedMilliseconds
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $env:TETHERS_J13B_CTRL_RESULT -Encoding utf8NoBOM
}
catch {
    [ordered]@{ controller_error = $_.Exception.ToString() } |
        ConvertTo-Json -Compress | Set-Content -LiteralPath $env:TETHERS_J13B_CTRL_RESULT -Encoding utf8NoBOM
    exit 1
}
'@ | Set-Content -LiteralPath $controllerPath -Encoding utf8NoBOM

    $env:TETHERS_J13B_CTRL_HOST = $HostPath
    $env:TETHERS_J13B_CTRL_CWD = $Workspace.Root
    $env:TETHERS_J13B_CTRL_CONFIG = $Workspace.Config
    $env:TETHERS_J13B_CTRL_ENGINE = $EnginePath
    $env:TETHERS_J13B_CTRL_INPUT = $InputPath
    $env:TETHERS_J13B_CTRL_TRAIL = $TrailPath
    $env:TETHERS_J13B_CTRL_REPLAY_ROOT = $ReplayRoot
    $env:TETHERS_J13B_CTRL_PROVIDER_MARKER = $Workspace.Marker
    $env:TETHERS_J13B_CTRL_RESULT = $resultPath
    $controller = Start-Process -FilePath "pwsh.exe" -ArgumentList @("-NoProfile", "-File", ('"' + $controllerPath + '"')) -WindowStyle Hidden -PassThru
    if (-not $controller.WaitForExit(20000)) {
        $controller.Kill($true)
        $controller.WaitForExit()
        throw "Ctrl+C controller exceeded 20000ms."
    }
    if (-not (Test-Path -LiteralPath $resultPath -PathType Leaf)) {
        throw "Ctrl+C controller did not produce a result."
    }
    $result = Get-Content -Raw -LiteralPath $resultPath | ConvertFrom-Json
    if ($null -ne $result.PSObject.Properties["controller_error"]) {
        throw "Ctrl+C controller failed: $($result.controller_error)"
    }
    [pscustomobject]@{
        ExitCode = [int]$result.exit_code
        Stdout = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($result.stdout_base64))
        Stderr = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($result.stderr_base64))
        InterruptMilliseconds = [long]$result.interrupt_ms
    }
}

if (-not (Test-Path -LiteralPath $HostPath -PathType Leaf)) {
    throw "Host executable is missing: $HostPath. Run cargo build first."
}
if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "OCaml engine executable is missing: $EnginePath. Build the current worktree engine first."
}
if (-not (Test-Path -LiteralPath $StandingManifest -PathType Leaf) -or -not (Test-Path -LiteralPath $AskManifest -PathType Leaf)) {
    throw "Required reviewed run fixture manifest is missing."
}

$script:TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("Tethers J13B run " + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempRoot | Out-Null

try {
    $allow = New-RunWorkspace -Name "allow fixture"
    $replayRoot = Join-Path $allow.Root "replay root"
    Provision-ReplayRoot $replayRoot $allow.Root
    $inputPath = Join-Path $allow.Root "input.json"
    $trailPath = Join-Path $allow.Root "trail.jsonl"
    Write-RunInput $inputPath
    $valid = Invoke-Run $allow $inputPath $trailPath $replayRoot

    Invoke-Case "valid completed run admits, executes once, and records a Result Anchor" {
        $envelope = ConvertFrom-SingleEnvelope $valid "completed" 0
        Assert-Equal $envelope.data.evaluation_id "eval_demo_001" "evaluation ID was not preserved."
        Assert-True ($null -ne $envelope.data.result_anchor) "completed run did not expose its Result Anchor."
        Assert-Equal (Get-MethodCount $allow.Marker "tools/call") 1 "expected one effectful provider call."
        $entries = @(Get-Content -LiteralPath $trailPath | ForEach-Object { $_ | ConvertFrom-Json })
        $admissionIndex = [array]::FindIndex($entries, [Predicate[object]]{ param($entry) $entry.PSObject.Properties["kind"] -and $entry.kind -eq "event_admitted" })
        $intentIndex = [array]::FindIndex($entries, [Predicate[object]]{ param($entry) $null -ne $entry.PSObject.Properties["capability_name"] })
        $outcomeIndex = [array]::FindIndex($entries, [Predicate[object]]{ param($entry) $entry.PSObject.Properties["status"] -and $entry.status -eq "succeeded" })
        Assert-True ($admissionIndex -ge 0 -and $admissionIndex -lt $intentIndex -and $intentIndex -lt $outcomeIndex) "admission, intent, and outcome order is incorrect."
    }

    $replay = Invoke-Run $allow $inputPath $trailPath $replayRoot
    Invoke-Case "exact replay is completed but performs no second provider effect" {
        $envelope = ConvertFrom-SingleEnvelope $replay "completed" 0
        Assert-Equal $envelope.data.execution_status "replay_blocked_completed_success" "replay status mismatch."
        Assert-Equal (Get-MethodCount $allow.Marker "tools/call") 1 "replay must not invoke the provider."
    }

    Invoke-Case "not-matched has no effectful provider call" {
        $workspace = New-RunWorkspace -Name "not matched"
        $root = Join-Path $workspace.Root "replay root"
        Provision-ReplayRoot $root $workspace.Root
        $input = Join-Path $workspace.Root "input.json"
        Write-RunInput $input -EventName "coding.other"
        $result = Invoke-Run $workspace $input (Join-Path $workspace.Root "trail.jsonl") $root
        $null = ConvertFrom-SingleEnvelope $result "no_actions" 0
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "not-matched must not call the provider."
    }

    Invoke-Case "Deny has no effectful provider call" {
        $workspace = New-RunWorkspace -Name "deny" -Policy "deny"
        $root = Join-Path $workspace.Root "replay root"
        Provision-ReplayRoot $root $workspace.Root
        $input = Join-Path $workspace.Root "input.json"
        Write-RunInput $input
        $result = Invoke-Run $workspace $input (Join-Path $workspace.Root "trail.jsonl") $root
        $null = ConvertFrom-SingleEnvelope $result "denied" 0
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "Deny must not call the provider."
    }

    Invoke-Case "Ask records approval_requested without a public approval ID" {
        $workspace = New-RunWorkspace -Name "ask" -Ask
        $root = Join-Path $workspace.Root "replay root"
        Provision-ReplayRoot $root $workspace.Root
        $input = Join-Path $workspace.Root "input.json"
        Write-RunInput $input
        $trail = Join-Path $workspace.Root "trail.jsonl"
        $result = Invoke-Run $workspace $input $trail $root
        $envelope = ConvertFrom-SingleEnvelope $result "approval_required" 5
        Assert-True ($result.Stdout -notmatch 'approval_id') "public Ask output exposed an approval ID."
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "Ask must not call the provider."
        Assert-True ((Get-Content -Raw -LiteralPath $trail) -match '"approval_requested"') "Ask Trail entry is missing."
        Assert-True ($null -ne $envelope.data.action_id) "Ask result omitted action ID."
    }

    Invoke-Case "unprovisioned replay root is unavailable before provider effect" {
        $workspace = New-RunWorkspace -Name "unprovisioned"
        $input = Join-Path $workspace.Root "input.json"
        Write-RunInput $input
        $result = Invoke-Run $workspace $input (Join-Path $workspace.Root "trail.jsonl") (Join-Path $workspace.Root "unprovisioned root")
        $envelope = ConvertFrom-SingleEnvelope $result "unavailable" 4
        Assert-Equal $envelope.error.code "REPLAY_PERSISTENCE_UNAVAILABLE" "replay unavailable code mismatch."
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "unprovisioned root must not call provider."
    }

    Invoke-Case "invalid public input stops before engine or provider launch" {
        $workspace = New-RunWorkspace -Name "invalid input"
        $root = Join-Path $workspace.Root "replay root"
        Provision-ReplayRoot $root $workspace.Root
        foreach ($body in @(
            '{bad json',
            '{"format_version":"1","format_version":"1"}',
            '{"format_version":"2","evaluation_id":"e","tether":{"id":"fixture-run","version":"1"},"event":{"id":"evt","name":"coding.task_completed","data":{}},"facts":{}}',
            '{"format_version":"1","evaluation_id":"e","extra":1,"tether":{"id":"fixture-run","version":"1"},"event":{"id":"evt","name":"coding.task_completed","data":{}},"facts":{}}',
            '{"format_version":"1","evaluation_id":"e","tether":{"id":"fixture-run","version":"1"},"event":{"id":"evt","name":"coding.task_completed","data":[]},"facts":{}}',
            '{"format_version":"1","evaluation_id":"e","tether":{"id":"fixture-run","version":"1"},"event":{"id":"evt","name":"coding.task_completed","data":{}},"facts":[]}',
            '{"format_version":"1","evaluation_id":"e","tether":{"id":"unknown","version":"1"},"event":{"id":"evt","name":"coding.task_completed","data":{}},"facts":{}}'
        )) {
            Remove-Item -LiteralPath $workspace.Marker -Force -ErrorAction SilentlyContinue
            $input = Join-Path $workspace.Root "input.json"
            Write-Text $input $body
            $result = Invoke-Run $workspace $input (Join-Path $workspace.Root ("trail " + [guid]::NewGuid().ToString() + ".jsonl")) $root
            $null = ConvertFrom-SingleEnvelope $result "invalid_data" 3
            Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "invalid input must not call provider."
        }
    }

    Invoke-Case "reordered options and invalid CLI usage preserve the frozen envelope" {
        $workspace = New-RunWorkspace -Name "reordered"
        $root = Join-Path $workspace.Root "replay root"
        Provision-ReplayRoot $root $workspace.Root
        $input = Join-Path $workspace.Root "input.json"
        Write-RunInput $input -EvaluationId "eval_reordered"
        $result = Invoke-Run $workspace $input (Join-Path $workspace.Root "trail.jsonl") $root -Reordered
        $envelope = ConvertFrom-SingleEnvelope $result "completed" 0
        Assert-Equal $envelope.data.evaluation_id "eval_reordered" "reordered options changed evaluation ID."
        foreach ($arguments in @(
            @("run", "--config", $workspace.Config),
            @("run", "--config", $workspace.Config, "--config", $workspace.Config, "--engine", $EnginePath, "--input", $input, "--trail", (Join-Path $workspace.Root "x"), "--host-data-root", $root),
            @("run", "--config", $workspace.Config, "--engine", $EnginePath, "--input", $input, "--trail", (Join-Path $workspace.Root "x"), "--host-data-root", $root, "--unknown")
        )) {
            $bad = Invoke-Host -WorkingDirectory $workspace.Root -ArgumentList $arguments
            $lines = @($bad.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
            Assert-Equal $lines.Count 1 "invalid CLI output must be one JSON document."
            $badEnvelope = $lines[0] | ConvertFrom-Json
            Assert-Equal $badEnvelope.status "invalid_cli_usage" "invalid CLI status mismatch."
            Assert-Equal $bad.ExitCode 2 "invalid CLI exit mismatch."
            Assert-Equal ([int]$badEnvelope.exit_code) $bad.ExitCode "invalid CLI embedded exit mismatch."
        }
    }

    Invoke-Case "Ctrl+C during a blocked public run returns interrupted" {
        $workspace = New-RunWorkspace -Name "interrupted" -ProviderMode "run-hang-initialize"
        $root = Join-Path $workspace.Root "replay root"
        Provision-ReplayRoot $root $workspace.Root
        $input = Join-Path $workspace.Root "input.json"
        $trail = Join-Path $workspace.Root "trail.jsonl"
        Write-RunInput $input -EvaluationId "eval_interrupted"
        $result = Invoke-InterruptedRun $workspace $input $trail $root
        $envelope = ConvertFrom-SingleEnvelope $result "interrupted" 10
        Assert-Equal $envelope.error.code "INTERRUPTED" "interruption machine code mismatch."
        Assert-True ($result.InterruptMilliseconds -le 5000) "interruption exceeded 5 seconds."
        Assert-Equal (Get-MethodCount $workspace.Marker "tools/call") 0 "interrupted run must not call the provider."
    }

    Write-Output "J13B public run acceptance: $script:passedCount passed, 0 failed"
}
finally {
    Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
