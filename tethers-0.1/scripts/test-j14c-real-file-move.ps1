Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostPath = Join-Path $RepoRoot "host-rust\target\debug\tethers-reference-host.exe"
$EnginePath = Join-Path $RepoRoot "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"
$ManifestPath = Join-Path $RepoRoot "protocol\capability-manifests\file-move-local.json"
$ProviderScript = Join-Path $RepoRoot "providers\tethers-local-file-provider.ps1"
$Digest = "sha256:6a99459d4f01bca270ae7453757bcab9ce6b8fd4634f0be185a07ae13a34ac4e"

$CommittedTether = Join-Path $RepoRoot "scenarios\j14c-real-file-move\tethers\sort-invoice.tether"
$CommittedInput = Join-Path $RepoRoot "scenarios\j14c-real-file-move\input.invoice.json"
$CommittedPhoto = Join-Path $RepoRoot "scenarios\j14c-real-file-move\input.photo.json"
$CommittedTemplate = Join-Path $RepoRoot "scenarios\j14c-real-file-move\runtime.template.json"

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
    if ($Actual -ne $Expected) { throw "$Message`nExpected: $Expected`nGot:    $Actual" }
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
    [pscustomobject]@{ ExitCode = $exitCode; Stdout = ($output -join "`n") }
}
function ConvertFrom-SingleEnvelope {
    param([Parameter(Mandatory = $true)]$Result, [string]$ExpectedCommand, [string]$ExpectedStatus, [int]$ExpectedExit)
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
function Write-Utf8NoBom {
    param([string]$Path, [string]$Text)
    [System.IO.File]::WriteAllText($Path, $Text, [System.Text.UTF8Encoding]::new($false))
}
function Write-InputJson {
    param([string]$Path, [string]$SourcePath, [string]$DestinationPath, [string]$FileType, [string]$FileName)
    $input = [ordered]@{
        format_version = "1"
        evaluation_id = "eval_j14c_custom_001"
        tether = [ordered]@{ id = "j14c-sort-invoice"; version = "1" }
        event = [ordered]@{
            id = "evt_j14c_custom_001"
            name = "folder.received_file"
            data = [ordered]@{ source_path = $SourcePath; destination_path = $DestinationPath }
        }
        facts = [ordered]@{ "file.type" = $FileType; "file.name" = $FileName }
    }
    Write-Utf8NoBom $Path ($input | ConvertTo-Json -Depth 10)
}
function Get-MethodCount {
    param([string]$Marker, [string]$Method)
    if (-not (Test-Path -LiteralPath $Marker -PathType Leaf)) { return 0 }
    return @((Get-Content -LiteralPath $marker) | Where-Object { $_ -eq $Method }).Count
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
            $rule = [System.Security.AccessControl.FileSystemAccessRule]::new($trustee, [System.Security.AccessControl.FileSystemRights]::FullControl, $inheritance, $propagation, [System.Security.AccessControl.AccessControlType]::Allow)
            $acl.AddAccessRule($rule)
        }
        Set-Acl -LiteralPath $Root -AclObject $acl
    }
    $result = Invoke-Host -WorkingDirectory $WorkingDirectory -ArgumentList @("provision-replay", $Root)
    Assert-Equal $result.ExitCode 0 "replay provisioning failed: $($result.Stdout)"
}

if (-not (Test-Path -LiteralPath $HostPath -PathType Leaf)) { throw "Host executable missing: $HostPath. Build first." }
if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) { throw "OCaml engine missing: $EnginePath." }
if (-not (Test-Path -LiteralPath $ManifestPath -PathType Leaf)) { throw "Manifest missing: $ManifestPath" }
if (-not (Test-Path -LiteralPath $ProviderScript -PathType Leaf)) { throw "Provider script missing: $ProviderScript" }

$cargoLockPath = Join-Path $RepoRoot "host-rust\Cargo.lock"
$cargoLockHash = (Get-FileHash -Path $cargoLockPath -Algorithm SHA256).Hash.ToLower()
$ExpectedCargoLockHash = "d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602"
if ($cargoLockHash -ne $ExpectedCargoLockHash) { throw "Cargo.lock hash mismatch: $cargoLockHash" }

$hashTetherStart = Get-FileHash-SHA256 $CommittedTether
$hashInputStart = Get-FileHash-SHA256 $CommittedInput
$hashTemplateStart = Get-FileHash-SHA256 $CommittedTemplate

$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("Tethers J14C " + [char]0x00E9 + " " + [guid]::NewGuid().ToString())
Assert-True ($TempRoot -match " ") "temp root must contain a space"
Assert-True ($TempRoot -cmatch "[^\x00-\x7F]") "temp root must contain a non-ASCII character"
New-Item -ItemType Directory -Path $TempRoot | Out-Null

$SourcePrefix = "workspace/inbox/"
$DestinationPrefix = "workspace/invoices/"

function New-Workspace {
    param([string]$Name, [string]$InputJson, [string]$SourcePath, [string]$DestinationPath)
    $ws = Join-Path $TempRoot $Name
    New-Item -ItemType Directory -Force -Path $ws, (Join-Path $ws "tethers"), (Join-Path $ws "manifests"), (Join-Path $ws "scripts"), (Join-Path $ws "workspace") | Out-Null
    Copy-Item -LiteralPath $ManifestPath -Destination (Join-Path $ws "manifests\file-move-local.json")
    Copy-Item -LiteralPath $ProviderScript -Destination (Join-Path $ws "scripts\tethers-local-file-provider.ps1")
    Copy-Item -LiteralPath $CommittedTether -Destination (Join-Path $ws "tethers\sort-invoice.tether")

    $marker = Join-Path $ws "provider-methods.txt"
    $trailPath = Join-Path $ws "trail.jsonl"
    $replayRoot = Join-Path $ws "replay-data"

    $config = [ordered]@{
        format_version = "0.1"
        tether_set = [ordered]@{
            id = "scenario.j14c.real-file-move"
            version = "1"
            tethers = @([ordered]@{ id = "j14c-sort-invoice"; version = "1"; source_path = "tethers/sort-invoice.tether" })
            capability_requirements = @([ordered]@{ name = "file.move"; version = 1; reason = "J14C real local file move proof" })
        }
        providers = @([ordered]@{
            id = "tethers-local-file-provider"
            display_name = "Tethers Local File Provider"
            transport = [ordered]@{
                kind = "stdio"
                command = "pwsh.exe"
                args = @("-NoProfile", "-File", (Join-Path $ws "scripts\tethers-local-file-provider.ps1"), "-ProviderRoot", $ws, "-SourcePrefix", $SourcePrefix, "-DestinationPrefix", $DestinationPrefix, "-MarkerFile", $marker)
                protocol_version = "2025-11-25"
            }
            capabilities = @([ordered]@{
                name = "file.move"
                version = 1
                manifest_path = "manifests/file-move-local.json"
                pinned_digest = $Digest
                scope_binding = [ordered]@{ kind = "path_prefix"; argument_json_pointer = "/source_path" }
            })
        })
        policy = [ordered]@{
            default = "deny"
            rules = @([ordered]@{ name = "file.move"; version = 1; decision = "allow" })
        }
    }
    $runtimePath = Join-Path $ws "runtime.json"
    Write-Utf8NoBom $runtimePath ($config | ConvertTo-Json -Depth 30)
    Copy-Item -LiteralPath $InputJson -Destination (Join-Path $ws "input.json")

    [pscustomobject]@{ ws = $ws; marker = $marker; trailPath = $trailPath; replayRoot = $replayRoot; sourceFull = (Join-Path $ws $SourcePath.Replace("/", [System.IO.Path]::DirectorySeparatorChar)); destinationFull = (Join-Path $ws $DestinationPath.Replace("/", [System.IO.Path]::DirectorySeparatorChar)); runtimePath = (Join-Path $ws "runtime.json"); inputPath = (Join-Path $ws "input.json") }
}

function Get-ExecutionId {
    param([Parameter(Mandatory = $true)]$RunResult)
    $lines = @($RunResult.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $envObj = $lines[0] | ConvertFrom-Json
    return $envObj.data.execution_id
}

function Invoke-Case {
    param([string]$Name, [scriptblock]$Body)
    $script:caseCount++
    Write-Output "ROW: $($script:caseCount). $Name"
    & $Body
    $script:passedCount++
    Write-Output "  PASS"
}

# ------------------------------------------------------------------
# F01 check admits real file provider
# ------------------------------------------------------------------
Invoke-Case "F01 public check admits real file provider" {
    $w = New-Workspace "F01" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/invoice-july.pdf"
    $checkResult = Invoke-Host $w.ws @("check", "--config", $w.runtimePath, "--engine", $EnginePath)
    $env = ConvertFrom-SingleEnvelope $checkResult "check" "ok" 0

    Assert-True ($null -ne $env.data.tethers) "check data missing tethers"
    Assert-Equal $env.data.tethers.Count 1 "expected one configured Tether"
    Assert-Equal $env.data.tethers[0].id "j14c-sort-invoice" "tether ID"
    Assert-Equal $env.data.tethers[0].status "valid" "tether status"
    Assert-True ($null -ne $env.data.providers) "check data missing providers"
    Assert-Equal $env.data.providers.Count 1 "expected one configured provider"
    Assert-Equal $env.data.providers[0].status "available" "provider status"
    $m = Get-Content -Raw $ManifestPath | ConvertFrom-Json
    Assert-Equal $m.capability_name "file.move" "capability name"
    Assert-Equal ([int]$m.capability_version) 1 "capability version"
    Assert-Equal $m.provider.identity "tethers-local-file-provider" "provider identity"
    Assert-Equal $m.digest $Digest "manifest digest"

    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "check initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "check tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 0 "check tools/call count"

    Assert-True (-not (Test-Path -LiteralPath $w.trailPath)) "Trail must not exist after check"
    Assert-True ((Get-ChildItem -LiteralPath (Join-Path $w.ws "workspace") -Recurse -ErrorAction SilentlyContinue | Measure-Object).Count -eq 0) "no file effect during check"
}

# ------------------------------------------------------------------
# F02 non-matching photo remains untouched
# ------------------------------------------------------------------
Invoke-Case "F02 non-matching photo remains untouched" {
    $w = New-Workspace "F02" $CommittedPhoto "workspace/inbox/holiday-photo.jpg" "workspace/invoices/holiday-photo.jpg"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    [System.IO.File]::WriteAllText((Join-Path $w.ws "workspace/inbox/holiday-photo.jpg"), "holiday-photo-bytes")
    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "no_actions" 0

    Assert-Equal $env.data.execution_status "no_actions" "execution status"
    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "run initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "run tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 0 "run tools/call count"

    $photoInbox = Join-Path $w.ws "workspace/inbox/holiday-photo.jpg"
    $photoDest = Join-Path $w.ws "workspace/invoices/holiday-photo.jpg"
    Assert-True (Test-Path -LiteralPath $photoInbox) "photo still in inbox"
    Assert-True (-not (Test-Path -LiteralPath $photoDest)) "no destination photo"
}

# ------------------------------------------------------------------
# F03 matching invoice moves exactly once
# ------------------------------------------------------------------
Invoke-Case "F03 matching invoice moves exactly once" {
    $w = New-Workspace "F03" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/invoice-july.pdf"
    $srcBefore = Join-Path $w.ws "workspace/inbox/invoice-july.pdf"
    $dstBefore = Join-Path $w.ws "workspace/invoices/invoice-july.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    [System.IO.File]::WriteAllText($srcBefore, "invoice-july-pdf-bytes")
    [System.IO.File]::WriteAllText((Join-Path $w.ws "workspace/inbox/holiday-photo.jpg"), "holiday-photo-bytes")
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/invoices") | Out-Null

    $hashBefore = (Get-FileHash -Path $srcBefore -Algorithm SHA256).Hash.ToLower()

    Provision-ReplayRoot $w.replayRoot $w.ws
    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "completed" 0

    $executionId = $env.data.execution_id
    Assert-True ($null -ne $executionId) "execution_id must be present"
    Assert-True ($executionId -match '^exec_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') "execution_id UUIDv4 format"

    Assert-Equal $env.data.execution_status "completed" "execution status"
    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "run initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "run tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 1 "run tools/call count"

    $anchor = $env.data.result_anchor
    Assert-True ($null -ne $anchor) "result_anchor must be present"
    Assert-Equal $anchor.event_name "capability.succeeded" "anchor event_name"
    Assert-Equal $anchor.facts.capability.name "file.move" "anchor capability name"
    Assert-Equal ([int]$anchor.facts.capability.version) 1 "anchor capability version"
    Assert-Equal $anchor.facts.manifest_digest $Digest "anchor manifest digest"
    Assert-Equal $anchor.facts.provider_identity "tethers-local-file-provider" "anchor provider identity"

    $sourceMoved = -not (Test-Path -LiteralPath $srcBefore)
    $destExists = Test-Path -LiteralPath $dstBefore
    Assert-True $sourceMoved "source invoice absent after move"
    Assert-True $destExists "destination invoice exists"
    $hashAfter = (Get-FileHash -Path $dstBefore -Algorithm SHA256).Hash.ToLower()
    Assert-Equal $hashAfter $hashBefore "destination preserves source bytes"

    $photoStays = Test-Path -LiteralPath (Join-Path $w.ws "workspace/inbox/holiday-photo.jpg")
    Assert-True $photoStays "unrelated photo untouched in inbox"

    $anchorStr = $env.data.result_anchor | ConvertTo-Json -Compress
    Assert-True ($anchorStr -notmatch '"execution_id"') "result_anchor must not contain execution_id"
}

# ------------------------------------------------------------------
# F04 public Trail explains the move
# ------------------------------------------------------------------
Invoke-Case "F04 public Trail explains the move" {
    $w = New-Workspace "F04" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/invoice-july.pdf"
    $src = Join-Path $w.ws "workspace/inbox/invoice-july.pdf"
    $dst = Join-Path $w.ws "workspace/invoices/invoice-july.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    [System.IO.File]::WriteAllText($src, "invoice-july-pdf-bytes")
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/invoices") | Out-Null

    Provision-ReplayRoot $w.replayRoot $w.ws
    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "completed" 0
    $executionId = $env.data.execution_id

    $trailResult = Invoke-Host $w.ws @("trail", "--trail", $w.trailPath, "--execution-id", $executionId)
    $trailEnv = ConvertFrom-SingleEnvelope $trailResult "trail" "ok" 0
    Assert-Equal $trailEnv.data.execution_id $executionId "trail execution_id matches"

    $entries = $trailEnv.data.entries
    Assert-True ($entries[0].PSObject.Properties["capability_name"] -ne $null) "intent capability_name"
    Assert-Equal $entries[0].capability_name "file.move" "intent capability_name"
    Assert-Equal ([int]$entries[0].capability_version) 1 "intent capability_version"
    Assert-Equal $entries[0].provider_identity "tethers-local-file-provider" "intent provider_identity"
    Assert-Equal $entries[0].manifest_digest $Digest "intent manifest_digest"
    Assert-True ($entries[1].PSObject.Properties["status"] -ne $null) "outcome status"
    Assert-Equal $entries[1].status "succeeded" "outcome status succeeded"
}

# ------------------------------------------------------------------
# F05 exact replay causes no second move
# ------------------------------------------------------------------
Invoke-Case "F05 exact replay causes no second move" {
    $w = New-Workspace "F05" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/invoice-july.pdf"
    $src = Join-Path $w.ws "workspace/inbox/invoice-july.pdf"
    $dst = Join-Path $w.ws "workspace/invoices/invoice-july.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    [System.IO.File]::WriteAllText($src, "invoice-july-pdf-bytes")
    [System.IO.File]::WriteAllText((Join-Path $w.ws "workspace/inbox/holiday-photo.jpg"), "holiday-photo-bytes")
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/invoices") | Out-Null

    Provision-ReplayRoot $w.replayRoot $w.ws
    $run1 = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env1 = ConvertFrom-SingleEnvelope $run1 "run" "completed" 0
    $firstId = $env1.data.execution_id

    $replay = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env2 = ConvertFrom-SingleEnvelope $replay "run" "completed" 0

    Assert-Equal $env2.data.execution_status "replay_blocked_completed_success" "replay status"
    Assert-Equal $env2.data.execution_id $firstId "replay execution_id matches"

    Assert-Equal (Get-MethodCount $w.marker "initialize") 2 "replay initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 2 "replay tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 1 "replay tools/call count"

    Assert-True (-not (Test-Path -LiteralPath $src)) "source still absent"
    Assert-True (Test-Path -LiteralPath $dst) "destination still present"
    $photoStays = Test-Path -LiteralPath (Join-Path $w.ws "workspace/inbox/holiday-photo.jpg")
    Assert-True $photoStays "photo untouched"
}

# ------------------------------------------------------------------
# F06 out-of-scope source is denied
# ------------------------------------------------------------------
Invoke-Case "F06 out-of-scope source is denied" {
    $w = New-Workspace "F06" $CommittedInput "workspace/outsider/invoice-secret.pdf" "workspace/invoices/invoice-secret.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/outsider") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/invoices") | Out-Null
    Write-InputJson $w.inputPath "workspace/outsider/invoice-secret.pdf" "workspace/invoices/invoice-secret.pdf" "pdf" "invoice-secret.pdf"
    [System.IO.File]::WriteAllText((Join-Path $w.ws "workspace/outsider/invoice-secret.pdf"), "secret-bytes")

    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "denied" 0
    Assert-Equal $env.data.execution_status "denied" "execution status"
    Assert-True ($null -eq $env.data.PSObject.Properties["execution_id"] -or $null -eq $env.data.execution_id) "no execution_id for denied"

    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "f06 initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "f06 tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 0 "f06 tools/call count"

    Assert-True (Test-Path -LiteralPath (Join-Path $w.ws "workspace/outsider/invoice-secret.pdf")) "outside source untouched"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $w.ws "workspace/invoices/invoice-secret.pdf"))) "denied destination absent"
}

# ------------------------------------------------------------------
# F07 traversal destination fails safely
# ------------------------------------------------------------------
Invoke-Case "F07 traversal destination fails safely" {
    $w = New-Workspace "F07" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/../invoices/invoice-july.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/invoices") | Out-Null
    Write-InputJson $w.inputPath "workspace/inbox/invoice-july.pdf" "workspace/invoices/../invoices/invoice-july.pdf" "pdf" "invoice-july.pdf"
    $src = Join-Path $w.ws "workspace/inbox/invoice-july.pdf"
    [System.IO.File]::WriteAllText($src, "invoice-july-pdf-bytes")

    Provision-ReplayRoot $w.replayRoot $w.ws
    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "failed" 6
    Assert-Equal $env.data.execution_status "failed" "execution status"

    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "f07 initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "f07 tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 1 "f07 tools/call count"

    Assert-True (Test-Path -LiteralPath $src) "f07 source untouched"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $w.ws "workspace/invoices/../invoices/invoice-july.pdf"))) "no traversal destination created"
}

# ------------------------------------------------------------------
# F08 existing destination is never overwritten
# ------------------------------------------------------------------
Invoke-Case "F08 existing destination is never overwritten" {
    $w = New-Workspace "F08" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/invoice-july.pdf"
    $src = Join-Path $w.ws "workspace/inbox/invoice-july.pdf"
    $dst = Join-Path $w.ws "workspace/invoices/invoice-july.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/invoices") | Out-Null
    [System.IO.File]::WriteAllText($src, "invoice-july-pdf-bytes")
    [System.IO.File]::WriteAllText($dst, "pre-existing-different-bytes")

    Provision-ReplayRoot $w.replayRoot $w.ws
    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "failed" 6
    Assert-Equal $env.data.execution_status "failed" "execution status"

    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "f08 initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "f08 tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 1 "f08 tools/call count"

    Assert-True (Test-Path -LiteralPath $src) "f08 source untouched"
    $dstContent = [System.IO.File]::ReadAllText($dst)
    Assert-Equal $dstContent "pre-existing-different-bytes" "destination keeps original content"
}

# ------------------------------------------------------------------
# F09 junction escape fails safely
# ------------------------------------------------------------------
Invoke-Case "F09 junction escape fails safely" {
    $w = New-Workspace "F09" $CommittedInput "workspace/inbox/invoice-july.pdf" "workspace/invoices/escape-target/invoice-july.pdf"
    New-Item -ItemType Directory -Force -Path (Join-Path $w.ws "workspace/inbox") | Out-Null
    $junctionDir = Join-Path $w.ws "workspace/invoices"
    New-Item -ItemType Directory -Force -Path $junctionDir | Out-Null
    $outside = Join-Path $w.ws "outside"
    New-Item -ItemType Directory -Force -Path $outside | Out-Null
    Write-InputJson $w.inputPath "workspace/inbox/invoice-july.pdf" "workspace/invoices/escape-target/invoice-july.pdf" "pdf" "invoice-july.pdf"
    $src = Join-Path $w.ws "workspace/inbox/invoice-july.pdf"
    [System.IO.File]::WriteAllText($src, "invoice-july-pdf-bytes")
    cmd.exe /c "mklink /J `"$(Join-Path $junctionDir 'escape-target')`" `"$outside`"" | Out-Null

    Provision-ReplayRoot $w.replayRoot $w.ws
    $runResult = Invoke-Host $w.ws @("run", "--config", $w.runtimePath, "--engine", $EnginePath, "--input", $w.inputPath, "--trail", $w.trailPath, "--host-data-root", $w.replayRoot)
    $env = ConvertFrom-SingleEnvelope $runResult "run" "failed" 6
    Assert-Equal $env.data.execution_status "failed" "execution status"

    Assert-Equal (Get-MethodCount $w.marker "initialize") 1 "f09 initialize count"
    Assert-Equal (Get-MethodCount $w.marker "tools/list") 1 "f09 tools/list count"
    Assert-Equal (Get-MethodCount $w.marker "tools/call") 1 "f09 tools/call count"

    Assert-True (Test-Path -LiteralPath $src) "f09 source untouched"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $w.ws "workspace/invoices/escape-target/invoice-july.pdf"))) "no junction-escaped file created"
}

# ------------------------------------------------------------------
# Non-mutation proof
# ------------------------------------------------------------------
Invoke-Case "committed scenario sources and Cargo.lock unchanged" {
    Assert-Equal (Get-FileHash-SHA256 $CommittedTether) $hashTetherStart "tether hash unchanged"
    Assert-Equal (Get-FileHash-SHA256 $CommittedInput) $hashInputStart "input hash unchanged"
    Assert-Equal (Get-FileHash-SHA256 $CommittedTemplate) $hashTemplateStart "template hash unchanged"
    $cargoLockNow = (Get-FileHash -Path $cargoLockPath -Algorithm SHA256).Hash.ToLower()
    Assert-Equal $cargoLockNow $ExpectedCargoLockHash "Cargo.lock hash unchanged"
}

Write-Output ""
Write-Output "============================================"
Write-Output "TOTAL: $($script:caseCount) rows, $($script:passedCount) passed, 0 failed"
Write-Output "ASSERTIONS: $($script:assertionCount)"
Write-Output "============================================"
