param(
    [string] $EnginePath = ""
)

# J13A public-boundary acceptance: real OCaml engine, J12 runtime config,
# reviewed manifest, stdio provider, bounded failures, and process cleanup.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$EngineDir = Join-Path $RepoRoot "engine-ocaml"
$HostDir = Join-Path $RepoRoot "host-rust"
$FixtureManifestPath = Join-Path $RepoRoot "protocol\capability-manifests\fixture-ping.json"
$FixtureScriptPath = Join-Path $RepoRoot "scripts\tethers-stdio-fixture.ps1"
$ReviewedDigest = "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"
$ProtocolVersion = "2025-11-25"
$Passed = 0
$CaseResults = [System.Collections.Generic.List[string]]::new()
$ActiveProcesses = [System.Collections.Generic.List[System.Diagnostics.Process]]::new()

function Assert-Command {
    param([Parameter(Mandatory = $true)][string] $Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

function Assert-True {
    param(
        [Parameter(Mandatory = $true)][bool] $Condition,
        [Parameter(Mandatory = $true)][string] $Message
    )
    if (-not $Condition) {
        throw $Message
    }
}

function Assert-Equal {
    param(
        [AllowNull()] $Actual,
        [AllowNull()] $Expected,
        [Parameter(Mandatory = $true)][string] $Message
    )
    if ($Actual -ne $Expected) {
        throw "$Message (expected '$Expected', got '$Actual')"
    }
}

function Invoke-Case {
    param(
        [Parameter(Mandatory = $true)][string] $Name,
        [Parameter(Mandatory = $true)][scriptblock] $Test
    )
    Write-Output "TEST: $Name"
    try {
        & $Test
        $script:Passed++
        $script:CaseResults.Add("PASS: $Name")
        Write-Output "  PASS"
    }
    catch {
        $script:CaseResults.Add("FAIL: $Name - $($_.Exception.Message)")
        Write-Output "  FAIL: $($_.Exception.Message)"
        throw
    }
}

function Get-PathBoundOpamSwitch {
    $switches = @(& opam switch list --short)
    if ($LASTEXITCODE -ne 0) {
        throw "opam switch list failed with exit code $LASTEXITCODE."
    }
    foreach ($candidate in $switches) {
        if ([System.IO.Path]::IsPathRooted($candidate)) {
            $dune = Join-Path $candidate "_opam\bin\dune.exe"
            if (Test-Path -LiteralPath $dune -PathType Leaf) {
                return $candidate
            }
        }
    }
    throw "No existing path-bound opam switch with dune.exe was found."
}

function Resolve-RealEngine {
    param([string] $RequestedPath)

    if (-not [string]::IsNullOrWhiteSpace($RequestedPath)) {
        return (Resolve-Path -LiteralPath $RequestedPath).Path
    }

    Assert-Command "opam"
    $opamSwitch = Get-PathBoundOpamSwitch
    & opam env --switch $opamSwitch --set-switch | Invoke-Expression
    if ($LASTEXITCODE -ne 0) {
        throw "opam env failed with exit code $LASTEXITCODE."
    }

    Push-Location $EngineDir
    try {
        & opam exec --switch $opamSwitch -- dune build
        if ($LASTEXITCODE -ne 0) {
            throw "Dune build failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }

    $resolved = Join-Path $EngineDir "_build\default\bin\tethers_mcp_main.exe"
    return (Resolve-Path -LiteralPath $resolved).Path
}

function Start-HostProcess {
    param(
        [Parameter(Mandatory = $true)][string[]] $ArgumentList,
        [Parameter(Mandatory = $true)][string] $WorkingDirectory,
        [switch] $NewProcessGroup
    )

    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $script:HostBinary
    $psi.WorkingDirectory = $WorkingDirectory
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    if ($NewProcessGroup) {
        $psi.CreateNewProcessGroup = $true
    }
    foreach ($argument in $ArgumentList) {
        $null = $psi.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::Start($psi)
    $script:ActiveProcesses.Add($process)
    [pscustomobject]@{
        Process = $process
        StdoutTask = $process.StandardOutput.ReadToEndAsync()
        StderrTask = $process.StandardError.ReadToEndAsync()
        Command = ('"{0}" {1}' -f $script:HostBinary, (($ArgumentList | ForEach-Object { '"{0}"' -f $_ }) -join " "))
    }
}

function Complete-HostProcess {
    param(
        [Parameter(Mandatory = $true)] $Started,
        [int] $TimeoutMs = 20000
    )

    if (-not $Started.Process.WaitForExit($TimeoutMs)) {
        $Started.Process.Kill($true)
        $Started.Process.WaitForExit()
        throw "Host exceeded harness timeout ${TimeoutMs}ms: $($Started.Command)"
    }
    $Started.Process.WaitForExit()
    [pscustomobject]@{
        Command = $Started.Command
        ExitCode = $Started.Process.ExitCode
        Stdout = $Started.StdoutTask.GetAwaiter().GetResult()
        Stderr = $Started.StderrTask.GetAwaiter().GetResult()
    }
}

function Invoke-Host {
    param(
        [Parameter(Mandatory = $true)][string[]] $ArgumentList,
        [string] $WorkingDirectory = $script:CallerDirectory,
        [int] $TimeoutMs = 20000
    )
    $started = Start-HostProcess -ArgumentList $ArgumentList -WorkingDirectory $WorkingDirectory
    Complete-HostProcess -Started $started -TimeoutMs $TimeoutMs
}

function ConvertFrom-SingleJsonDocument {
    param([Parameter(Mandatory = $true)][string] $Text)
    $trimmed = $Text.Trim()
    Assert-True ($trimmed.Length -gt 0) "Host stdout was empty."
    $lines = @($trimmed -split "\r?\n" | Where-Object { $_.Trim().Length -gt 0 })
    Assert-Equal $lines.Count 1 "Host stdout must contain exactly one non-empty JSON line."
    try {
        return $trimmed | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Host stdout was not exactly one JSON document: $trimmed"
    }
}

function Assert-Envelope {
    param(
        [Parameter(Mandatory = $true)] $Result,
        [Parameter(Mandatory = $true)][string] $Status,
        [Parameter(Mandatory = $true)][int] $ExitCode
    )
    Assert-Equal $Result.ExitCode $ExitCode "Process exit code mismatch."
    $envelope = ConvertFrom-SingleJsonDocument $Result.Stdout
    Assert-Equal $envelope.schema "tethers.cli/1" "Envelope schema mismatch."
    Assert-Equal $envelope.status $Status "Envelope status mismatch."
    Assert-Equal $envelope.exit_code $ExitCode "Embedded exit code mismatch."
    return $envelope
}

function New-ProviderConfig {
    param(
        [Parameter(Mandatory = $true)][string] $Identity,
        [Parameter(Mandatory = $true)][string] $CapabilityName,
        [Parameter(Mandatory = $true)][string] $ManifestFile,
        [Parameter(Mandatory = $true)][string] $Digest,
        [Parameter(Mandatory = $true)][string[]] $TransportArgs
    )
    [ordered]@{
        id = $Identity
        display_name = "J13A $Identity"
        transport = [ordered]@{
            kind = "stdio"
            command = "pwsh.exe"
            args = $TransportArgs
            protocol_version = $script:ProtocolVersion
        }
        capabilities = @(
            [ordered]@{
                name = $CapabilityName
                version = 1
                manifest_path = "manifests/$ManifestFile"
                pinned_digest = $Digest
            }
        )
    }
}

function New-FixtureProvider {
    param(
        [string] $Identity = "tethers-stdio-fixture",
        [string] $CapabilityName = "fixture.ping",
        [string] $ManifestFile = "fixture-ping.json",
        [string] $Digest = $script:ReviewedDigest,
        [string] $Mode = "valid",
        [string] $MarkerFile = "",
        [string] $CwdMarkerFile = ""
    )
    $transportArgs = [System.Collections.Generic.List[string]]::new()
    foreach ($argument in @("-NoProfile", "-File", "scripts/tethers-stdio-fixture.ps1", "-Mode", $Mode)) {
        $transportArgs.Add($argument)
    }
    if (-not [string]::IsNullOrWhiteSpace($MarkerFile)) {
        $transportArgs.Add("-MarkerFile")
        $transportArgs.Add($MarkerFile)
    }
    if (-not [string]::IsNullOrWhiteSpace($CwdMarkerFile)) {
        $transportArgs.Add("-CwdMarkerFile")
        $transportArgs.Add($CwdMarkerFile)
    }
    New-ProviderConfig `
        -Identity $Identity `
        -CapabilityName $CapabilityName `
        -ManifestFile $ManifestFile `
        -Digest $Digest `
        -TransportArgs $transportArgs.ToArray()
}

function Write-RuntimeConfig {
    param(
        [Parameter(Mandatory = $true)][string] $Name,
        [Parameter(Mandatory = $true)][object[]] $Tethers,
        [Parameter(Mandatory = $true)][object[]] $Providers
    )
    $requirements = @(
        foreach ($provider in $Providers) {
            foreach ($capability in $provider.capabilities) {
                [ordered]@{
                    name = $capability.name
                    version = $capability.version
                    reason = "J13A public-boundary acceptance"
                }
            }
        }
    )
    $rules = @(
        foreach ($requirement in $requirements) {
            [ordered]@{
                name = $requirement.name
                version = $requirement.version
                decision = "allow"
            }
        }
    )
    $config = [ordered]@{
        format_version = "0.1"
        tether_set = [ordered]@{
            id = "fixture.acceptance"
            version = "1"
            tethers = $Tethers
            capability_requirements = $requirements
        }
        providers = $Providers
        policy = [ordered]@{
            default = "deny"
            rules = $rules
        }
    }
    $path = Join-Path $script:RuntimeDirectory "$Name config.json"
    $config | ConvertTo-Json -Depth 40 | Set-Content -LiteralPath $path -Encoding utf8NoBOM
    return $path
}

function New-TetherReference {
    param(
        [Parameter(Mandatory = $true)][string] $Id,
        [Parameter(Mandatory = $true)][string] $FileName,
        [Parameter(Mandatory = $true)][string] $Source
    )
    $path = Join-Path $script:TetherDirectory $FileName
    $Source | Set-Content -LiteralPath $path -Encoding utf8NoBOM
    [ordered]@{
        id = $Id
        version = "1"
        source_path = "tethers/$FileName"
    }
}

function Invoke-Check {
    param(
        [Parameter(Mandatory = $true)][string] $ConfigPath,
        [string] $MarkerPath = "",
        [string] $EngineMode = "",
        [int] $TimeoutMs = 20000
    )
    $env:TETHERS_J13A_ENGINE_TARGET = $script:RealEnginePath
    $env:TETHERS_J13A_ENGINE_MARKER = $MarkerPath
    $env:TETHERS_J13A_ENGINE_MODE = $EngineMode
    Invoke-Host `
        -ArgumentList @("check", "--config", $ConfigPath, "--engine", $script:EngineProxyPath) `
        -WorkingDirectory $script:CallerDirectory `
        -TimeoutMs $TimeoutMs
}

function Wait-ForFile {
    param(
        [Parameter(Mandatory = $true)][string] $Path,
        [int] $TimeoutMs = 5000
    )
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        if ($stopwatch.ElapsedMilliseconds -ge $TimeoutMs) {
            throw "Timed out waiting for marker file: $Path"
        }
        Start-Sleep -Milliseconds 50
    }
}

function Wait-ForProcessExit {
    param(
        [Parameter(Mandatory = $true)][int] $ProcessId,
        [int] $TimeoutMs = 5000
    )
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    while (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue) {
        if ($stopwatch.ElapsedMilliseconds -ge $TimeoutMs) {
            return $false
        }
        Start-Sleep -Milliseconds 50
    }
    return $true
}

Assert-Command "cargo"
Assert-Command "pwsh"

# Build before resolving the host executable.
Push-Location $HostDir
try {
    & cargo build
    if ($LASTEXITCODE -ne 0) {
        throw "Rust host build failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}
$HostBinary = (Resolve-Path -LiteralPath (Join-Path $HostDir "target\debug\tethers-reference-host.exe")).Path
$RealEnginePath = Resolve-RealEngine -RequestedPath $EnginePath

$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "Tethers J13A Ω acceptance with spaces $([guid]::NewGuid())"
$RuntimeDirectory = Join-Path $TempRoot "runtime config with spaces"
$ManifestDirectory = Join-Path $RuntimeDirectory "manifests"
$TetherDirectory = Join-Path $RuntimeDirectory "tethers"
$RuntimeScriptDirectory = Join-Path $RuntimeDirectory "scripts"
$CallerDirectory = Join-Path $TempRoot "different caller working directory"
$EngineProxyDirectory = Join-Path $TempRoot "engine proxy with spaces"
$EngineProxyPath = Join-Path $EngineProxyDirectory "tethers engine proxy.exe"

$NativeSource = @'
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;

public static class J13AAcceptanceNative
{
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool GenerateConsoleCtrlEvent(uint ctrlEvent, uint processGroupId);

    public static string ComputeManifestDigest(string path)
    {
        using JsonDocument document = JsonDocument.Parse(File.ReadAllText(path));
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream, new JsonWriterOptions {
            Encoder = JavaScriptEncoder.Default,
            Indented = false
        }))
        {
            WriteCanonical(writer, document.RootElement, true);
        }
        byte[] digest = SHA256.HashData(stream.ToArray());
        return "sha256:" + Convert.ToHexString(digest).ToLowerInvariant();
    }

    private static void WriteCanonical(Utf8JsonWriter writer, JsonElement element, bool root)
    {
        switch (element.ValueKind)
        {
            case JsonValueKind.Object:
                writer.WriteStartObject();
                IEnumerable<JsonProperty> properties = element.EnumerateObject()
                    .Where(p => !root || (p.Name != "digest" && p.Name != "title" && p.Name != "description"))
                    .OrderBy(p => p.Name, StringComparer.Ordinal);
                foreach (JsonProperty property in properties)
                {
                    writer.WritePropertyName(property.Name);
                    WriteCanonical(writer, property.Value, false);
                }
                writer.WriteEndObject();
                break;
            case JsonValueKind.Array:
                writer.WriteStartArray();
                foreach (JsonElement item in element.EnumerateArray())
                    WriteCanonical(writer, item, false);
                writer.WriteEndArray();
                break;
            case JsonValueKind.String:
                writer.WriteStringValue(element.GetString());
                break;
            case JsonValueKind.Number:
                writer.WriteRawValue(element.GetRawText(), true);
                break;
            case JsonValueKind.True:
                writer.WriteBooleanValue(true);
                break;
            case JsonValueKind.False:
                writer.WriteBooleanValue(false);
                break;
            case JsonValueKind.Null:
                writer.WriteNullValue();
                break;
            default:
                throw new InvalidDataException("Unsupported JSON value kind.");
        }
    }
}
'@

$EngineProxySource = @'
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

public static class J13AEngineProxy
{
    public static int Main()
    {
        if (Environment.GetEnvironmentVariable("TETHERS_J13A_ENGINE_MODE") == "hang-initialize")
        {
            Thread.Sleep(Timeout.Infinite);
            return 0;
        }

        string target = Environment.GetEnvironmentVariable("TETHERS_J13A_ENGINE_TARGET");
        if (String.IsNullOrWhiteSpace(target))
            throw new InvalidOperationException("TETHERS_J13A_ENGINE_TARGET is missing.");
        string marker = Environment.GetEnvironmentVariable("TETHERS_J13A_ENGINE_MARKER");
        if (marker == null)
            marker = "";

        ProcessStartInfo start = new ProcessStartInfo {
            FileName = target,
            UseShellExecute = false,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true
        };
        using (Process child = Process.Start(start))
        {
            if (child == null)
                throw new InvalidOperationException("Failed to start real OCaml engine.");

            Task stdout = child.StandardOutput.BaseStream.CopyToAsync(Console.OpenStandardOutput());
            Task stderr = child.StandardError.BaseStream.CopyToAsync(Console.OpenStandardError());

            string line;
            while ((line = Console.ReadLine()) != null)
            {
                RecordMethod(marker, line);
                child.StandardInput.WriteLine(line);
                child.StandardInput.Flush();
            }
            child.StandardInput.Close();
            child.WaitForExit();
            Task.WaitAll(stdout, stderr);
            return child.ExitCode;
        }
    }

    private static void RecordMethod(string marker, string line)
    {
        if (String.IsNullOrWhiteSpace(marker))
            return;
        string compact = line.Replace(" ", "");
        if (compact.IndexOf("\"method\":\"initialize\"", StringComparison.Ordinal) >= 0)
            File.AppendAllText(marker, "initialize" + Environment.NewLine);
        else if (compact.IndexOf("\"method\":\"tools/call\"", StringComparison.Ordinal) >= 0
            && compact.IndexOf("\"name\":\"tethers.validate\"", StringComparison.Ordinal) >= 0)
            File.AppendAllText(marker, "tools/call:tethers.validate" + Environment.NewLine);
        else if (compact.IndexOf("\"method\":\"tools/call\"", StringComparison.Ordinal) >= 0
            && compact.IndexOf("\"name\":\"tethers.evaluate\"", StringComparison.Ordinal) >= 0)
            File.AppendAllText(marker, "tools/call:tethers.evaluate" + Environment.NewLine);
        else if (compact.IndexOf("\"method\":\"tools/call\"", StringComparison.Ordinal) >= 0)
            File.AppendAllText(marker, "tools/call:other" + Environment.NewLine);
    }
}
'@

try {
    New-Item -ItemType Directory -Path `
        $ManifestDirectory, `
        $TetherDirectory, `
        $RuntimeScriptDirectory, `
        $CallerDirectory, `
        $EngineProxyDirectory `
        -Force | Out-Null

    Copy-Item -LiteralPath $FixtureManifestPath -Destination (Join-Path $ManifestDirectory "fixture-ping.json")
    Copy-Item -LiteralPath $FixtureScriptPath -Destination (Join-Path $RuntimeScriptDirectory "tethers-stdio-fixture.ps1")

    Add-Type -TypeDefinition $NativeSource
    $engineProxySourcePath = Join-Path $EngineProxyDirectory "tethers engine proxy.cs"
    $EngineProxySource | Set-Content -LiteralPath $engineProxySourcePath -Encoding utf8NoBOM
    $cscPath = Join-Path $env:WINDIR "Microsoft.NET\Framework64\v4.0.30319\csc.exe"
    if (-not (Test-Path -LiteralPath $cscPath -PathType Leaf)) {
        $cscPath = Join-Path $env:WINDIR "Microsoft.NET\Framework\v4.0.30319\csc.exe"
    }
    Assert-True (Test-Path -LiteralPath $cscPath -PathType Leaf) "Windows C# compiler was not found."
    & $cscPath /nologo /target:exe "/out:$EngineProxyPath" $engineProxySourcePath
    if ($LASTEXITCODE -ne 0) {
        throw "Engine proxy compilation failed with exit code $LASTEXITCODE."
    }
    Assert-True (Test-Path -LiteralPath $EngineProxyPath -PathType Leaf) "Engine proxy executable was not created."

    $validSource = @'
tether "Fixture public check"

anchor
    coding.task_completed

when
    project.type is "software"

do
    lantern.task.record
        project: anchor.project
'@
    $invalidSource = "garbage syntax {{{"
    $validTether = New-TetherReference -Id "fixture-valid" -FileName "fixture valid.tether" -Source $validSource
    $invalidTether = New-TetherReference -Id "fixture-invalid" -FileName "fixture invalid.tether" -Source $invalidSource

    # A second temporary reviewed-fixture derivative gives the second provider
    # a unique exact identity/capability while retaining the same MCP tool schema.
    $secondManifestPath = Join-Path $ManifestDirectory "fixture-ping-second.json"
    $secondManifest = Get-Content -Raw -LiteralPath $FixtureManifestPath | ConvertFrom-Json
    $secondManifest.capability_name = "fixture.ping.second"
    $secondManifest.provider.identity = "tethers-stdio-fixture-second"
    $secondManifest.provider.display_name = "Tethers Stdio Fixture Second"
    $secondManifest.digest = "sha256:$("0" * 64)"
    $secondManifest | ConvertTo-Json -Depth 40 | Set-Content -LiteralPath $secondManifestPath -Encoding utf8NoBOM
    $secondDigest = [J13AAcceptanceNative]::ComputeManifestDigest($secondManifestPath)
    $secondManifest.digest = $secondDigest
    $secondManifest | ConvertTo-Json -Depth 40 | Set-Content -LiteralPath $secondManifestPath -Encoding utf8NoBOM

    # Confirm the PowerShell canonicalizer against the reviewed fixed digest.
    $computedReviewedDigest = [J13AAcceptanceNative]::ComputeManifestDigest(
        (Join-Path $ManifestDirectory "fixture-ping.json")
    )
    Assert-Equal $computedReviewedDigest $ReviewedDigest "Reviewed fixture manifest digest mismatch."

    $engineMarker = Join-Path $TempRoot "engine methods marker.txt"
    $providerMarker = Join-Path $TempRoot "provider methods marker.txt"
    $validProvider = New-FixtureProvider -Mode "record-methods" -MarkerFile $providerMarker
    $validConfig = Write-RuntimeConfig -Name "valid primary" -Tethers @($validTether) -Providers @($validProvider)
    $validResult = Invoke-Check -ConfigPath $validConfig -MarkerPath $engineMarker
    $validEnvelope = $null

    Invoke-Case "1. valid check returns ok and exit 0" {
        $script:validEnvelope = Assert-Envelope $validResult "ok" 0
        Assert-Equal $script:validEnvelope.data.tethers[0].status "valid" "Tether was not valid."
        Assert-Equal $script:validEnvelope.data.providers[0].status "available" "Provider was not available."
    }
    Invoke-Case "2. valid output is exactly one JSON document" {
        $null = ConvertFrom-SingleJsonDocument $validResult.Stdout
    }
    Invoke-Case "3. config and engine paths containing spaces work" {
        Assert-True ($validConfig -match " ") "Config path did not contain spaces."
        Assert-True ($EngineProxyPath -match " ") "Engine path did not contain spaces."
        Assert-Equal $validResult.ExitCode 0 "Paths containing spaces failed."
    }
    Invoke-Case "4. caller CWD may differ from config directory" {
        Assert-True ($CallerDirectory -ne $RuntimeDirectory) "Caller and config directories unexpectedly match."
        Assert-Equal $validResult.ExitCode 0 "Differing caller CWD failed."
    }

    $cwdMarker = Join-Path $TempRoot "provider cwd marker.txt"
    $cwdProvider = New-FixtureProvider -Mode "record-cwd" -CwdMarkerFile $cwdMarker
    $cwdConfig = Write-RuntimeConfig -Name "provider cwd" -Tethers @($validTether) -Providers @($cwdProvider)
    $cwdResult = Invoke-Check -ConfigPath $cwdConfig
    Invoke-Case "5. provider CWD equals canonical config directory" {
        $null = Assert-Envelope $cwdResult "ok" 0
        $recordedCwd = (Get-Content -Raw -LiteralPath $cwdMarker).Trim()
        $canonicalRuntime = (Resolve-Path -LiteralPath $RuntimeDirectory).Path
        Assert-Equal $recordedCwd $canonicalRuntime "Provider CWD was not the canonical config directory."
    }

    $engineMethods = @(Get-Content -LiteralPath $engineMarker)
    $providerMethods = @(Get-Content -LiteralPath $providerMarker)
    Invoke-Case "6. provider initialize marker exactly once" {
        Assert-Equal @($providerMethods | Where-Object { $_ -eq "initialize" }).Count 1 "Provider initialize count mismatch."
    }
    Invoke-Case "7. provider tools/list marker exactly once" {
        Assert-Equal @($providerMethods | Where-Object { $_ -eq "tools/list" }).Count 1 "Provider tools/list count mismatch."
    }
    Invoke-Case "8. provider marker contains no tools/call" {
        Assert-Equal @($providerMethods | Where-Object { $_ -eq "tools/call" }).Count 0 "Provider received tools/call."
    }
    Invoke-Case "9. engine marker records one validate per Tether and no evaluate" {
        Assert-Equal @($engineMethods | Where-Object { $_ -eq "initialize" }).Count 1 "Engine initialize count mismatch."
        Assert-Equal @($engineMethods | Where-Object { $_ -eq "tools/call:tethers.validate" }).Count 1 "Engine validate count mismatch."
        Assert-Equal @($engineMethods | Where-Object { $_ -eq "tools/call:tethers.evaluate" }).Count 0 "Engine received tethers.evaluate."
    }

    $invalidProviderMarker = Join-Path $TempRoot "invalid tether provider marker.txt"
    $invalidProvider = New-FixtureProvider -Mode "record-methods" -MarkerFile $invalidProviderMarker
    $invalidConfig = Write-RuntimeConfig -Name "invalid tether" -Tethers @($invalidTether) -Providers @($invalidProvider)
    $invalidResult = Invoke-Check -ConfigPath $invalidConfig
    Invoke-Case "10. invalid Tether returns invalid_data and launches no provider" {
        $envelope = Assert-Envelope $invalidResult "invalid_data" 3
        Assert-Equal $envelope.error.code "TETHER_INVALID" "Invalid Tether machine code mismatch."
        Assert-Equal $envelope.data.tethers[0].status "invalid" "Invalid Tether evidence missing."
        Assert-Equal @($envelope.data.providers).Count 0 "Invalid Tether unexpectedly has provider evidence."
        Assert-True (-not (Test-Path -LiteralPath $invalidProviderMarker)) "Provider launched for invalid Tether."
    }

    $missingProvider = New-FixtureProvider -Mode "missing-tool"
    $missingConfig = Write-RuntimeConfig -Name "missing configured tool" -Tethers @($validTether) -Providers @($missingProvider)
    $missingResult = Invoke-Check -ConfigPath $missingConfig
    Invoke-Case "11. missing tool returns unavailable with prior valid-Tether evidence" {
        $envelope = Assert-Envelope $missingResult "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_CAPABILITY_UNAVAILABLE" "Missing-tool machine code mismatch."
        Assert-Equal $envelope.data.tethers[0].status "valid" "Prior valid-Tether evidence was lost."
        Assert-Equal $envelope.data.providers[0].status "unavailable" "Failed provider evidence missing."
        Assert-Equal $envelope.data.providers[0].capabilities[0].status "unavailable" "Missing capability evidence missing."
    }

    $firstProvider = New-FixtureProvider -Mode "valid"
    $secondProvider = New-FixtureProvider `
        -Identity "tethers-stdio-fixture-second" `
        -CapabilityName "fixture.ping.second" `
        -ManifestFile "fixture-ping-second.json" `
        -Digest $secondDigest `
        -Mode "initialization-error"
    $laterConfig = Write-RuntimeConfig -Name "later provider failure" -Tethers @($validTether) -Providers @($firstProvider, $secondProvider)
    $laterResult = Invoke-Check -ConfigPath $laterConfig
    Invoke-Case "12. later provider failure preserves earlier successful provider" {
        $envelope = Assert-Envelope $laterResult "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_INITIALIZE_FAILED" "Later-provider machine code mismatch."
        Assert-Equal $envelope.error.field "/providers/1" "Later-provider field pointer mismatch."
        Assert-Equal $envelope.data.providers[0].status "available" "Earlier successful provider was lost."
        Assert-Equal $envelope.data.providers[1].status "initialize_failed" "Later failed provider evidence missing."
    }

    $engineHangStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $engineHangResult = Invoke-Check -ConfigPath $validConfig -EngineMode "hang-initialize" -TimeoutMs 20000
    $engineHangStopwatch.Stop()
    Invoke-Case "13. engine initialize hang is bounded" {
        $envelope = Assert-Envelope $engineHangResult "unavailable" 4
        Assert-Equal $envelope.error.code "ENGINE_LAUNCH_FAILED" "Engine hang machine code mismatch."
        Assert-True ($engineHangStopwatch.ElapsedMilliseconds -le 16000) "Engine hang exceeded 16 seconds."
    }

    $providerInitHang = New-FixtureProvider -Mode "hang-initialize"
    $providerInitHangConfig = Write-RuntimeConfig -Name "provider initialize hang" -Tethers @($validTether) -Providers @($providerInitHang)
    $providerInitStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $providerInitResult = Invoke-Check -ConfigPath $providerInitHangConfig -TimeoutMs 20000
    $providerInitStopwatch.Stop()
    Invoke-Case "14. provider initialize hang is bounded" {
        $envelope = Assert-Envelope $providerInitResult "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_INITIALIZE_FAILED" "Provider initialize hang code mismatch."
        Assert-Equal $envelope.data.providers[0].status "initialize_failed" "Provider initialize hang evidence missing."
        Assert-True ($providerInitStopwatch.ElapsedMilliseconds -le 16000) "Provider initialize hang exceeded 16 seconds."
    }

    $providerListHang = New-FixtureProvider -Mode "hang-tools-list"
    $providerListHangConfig = Write-RuntimeConfig -Name "provider tools list hang" -Tethers @($validTether) -Providers @($providerListHang)
    $providerListStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $providerListResult = Invoke-Check -ConfigPath $providerListHangConfig -TimeoutMs 20000
    $providerListStopwatch.Stop()
    Invoke-Case "15. provider tools/list hang is bounded" {
        $envelope = Assert-Envelope $providerListResult "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_TOOLS_LIST_FAILED" "Provider tools/list hang code mismatch."
        Assert-Equal $envelope.data.providers[0].status "tools_list_failed" "Provider tools/list hang evidence missing."
        Assert-True ($providerListStopwatch.ElapsedMilliseconds -le 16000) "Provider tools/list hang exceeded 16 seconds."
    }

    $stdoutProvider = New-FixtureProvider -Mode "stdout-log-text"
    $stdoutConfig = Write-RuntimeConfig -Name "stdout contamination" -Tethers @($validTether) -Providers @($stdoutProvider)
    $stdoutResult = Invoke-Check -ConfigPath $stdoutConfig
    Invoke-Case "16. provider stdout contamination fails closed" {
        $envelope = Assert-Envelope $stdoutResult "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_INITIALIZE_FAILED" "Stdout contamination code mismatch."
        Assert-Equal $envelope.data.providers[0].status "initialize_failed" "Stdout contamination evidence missing."
    }

    $oversizedProvider = New-FixtureProvider -Mode "oversized-line"
    $oversizedConfig = Write-RuntimeConfig -Name "oversized provider line" -Tethers @($validTether) -Providers @($oversizedProvider)
    $oversizedResult = Invoke-Check -ConfigPath $oversizedConfig
    Invoke-Case "17. oversized provider line fails closed" {
        $envelope = Assert-Envelope $oversizedResult "unavailable" 4
        Assert-Equal $envelope.error.code "PROVIDER_INITIALIZE_FAILED" "Oversized-line code mismatch."
        Assert-Equal $envelope.data.providers[0].status "initialize_failed" "Oversized-line evidence missing."
    }

    $blockingProviderScript = Join-Path $RuntimeScriptDirectory "blocking provider with descendant.ps1"
    @'
param([Parameter(Mandatory = $true)][string] $PidMarker)
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$descendant = Start-Process `
    -FilePath "pwsh.exe" `
    -ArgumentList "-NoProfile", "-Command", "Start-Sleep -Seconds 300" `
    -PassThru `
    -WindowStyle Hidden
@("direct=$PID", "descendant=$($descendant.Id)") | Set-Content -LiteralPath $PidMarker -Encoding utf8NoBOM
while ($true) { Start-Sleep -Seconds 60 }
'@ | Set-Content -LiteralPath $blockingProviderScript -Encoding utf8NoBOM

    $pidMarker = Join-Path $TempRoot "blocked provider pid marker.txt"
    $blockingArgs = @(
        "-NoProfile",
        "-File",
        "scripts/blocking provider with descendant.ps1",
        "-PidMarker",
        $pidMarker
    )
    $blockingProvider = New-ProviderConfig `
        -Identity "tethers-stdio-fixture" `
        -CapabilityName "fixture.ping" `
        -ManifestFile "fixture-ping.json" `
        -Digest $ReviewedDigest `
        -TransportArgs $blockingArgs
    $blockingConfig = Write-RuntimeConfig -Name "ctrl c blocked provider" -Tethers @($validTether) -Providers @($blockingProvider)

    $ctrlControllerPath = Join-Path $RuntimeScriptDirectory "ctrl c controller.ps1"
    $ctrlResultPath = Join-Path $TempRoot "ctrl c controller result.json"
    @'
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$source = @"
using System;
using System.Runtime.InteropServices;
public static class J13ACtrlController
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
        if (!AllocConsole())
            return false;
        IntPtr window = GetConsoleWindow();
        if (window != IntPtr.Zero)
            ShowWindow(window, 0);
        return SetConsoleCtrlHandler(Handler, true);
    }
}
"@

try {
    Add-Type -TypeDefinition $source
    if (-not [J13ACtrlController]::PrepareIsolatedConsole()) {
        throw "Controller could not prepare its isolated Ctrl+C console."
    }

    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $env:TETHERS_J13A_CTRL_HOST
    $psi.WorkingDirectory = $env:TETHERS_J13A_CTRL_CWD
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $false
    foreach ($argument in @(
        "check",
        "--config",
        $env:TETHERS_J13A_CTRL_CONFIG,
        "--engine",
        $env:TETHERS_J13A_CTRL_ENGINE
    )) {
        $null = $psi.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::Start($psi)
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()

    $wait = [System.Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath $env:TETHERS_J13A_CTRL_PID_MARKER -PathType Leaf)) {
        if ($wait.ElapsedMilliseconds -ge 8000) {
            throw "Timed out waiting for the blocked-provider PID marker."
        }
        Start-Sleep -Milliseconds 50
    }

    Start-Sleep -Milliseconds 100
    $interrupt = [System.Diagnostics.Stopwatch]::StartNew()
    if (-not [J13ACtrlController]::GenerateConsoleCtrlEvent(0, 0)) {
        throw "GenerateConsoleCtrlEvent(CTRL_C_EVENT) failed."
    }
    if (-not $process.WaitForExit(8000)) {
        $process.Kill($true)
        $process.WaitForExit()
        throw "Host did not exit within 8000ms after CTRL_C_EVENT."
    }
    $process.WaitForExit()
    $interrupt.Stop()

    [ordered]@{
        exit_code = $process.ExitCode
        stdout_base64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($stdoutTask.GetAwaiter().GetResult()))
        stderr_base64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($stderrTask.GetAwaiter().GetResult()))
        interrupt_ms = $interrupt.ElapsedMilliseconds
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $env:TETHERS_J13A_CTRL_RESULT -Encoding utf8NoBOM
}
catch {
    [ordered]@{
        controller_error = $_.Exception.ToString()
    } | ConvertTo-Json -Compress | Set-Content -LiteralPath $env:TETHERS_J13A_CTRL_RESULT -Encoding utf8NoBOM
    exit 1
}
'@ | Set-Content -LiteralPath $ctrlControllerPath -Encoding utf8NoBOM

    $env:TETHERS_J13A_ENGINE_TARGET = $RealEnginePath
    $env:TETHERS_J13A_ENGINE_MARKER = ""
    $env:TETHERS_J13A_ENGINE_MODE = ""
    $env:TETHERS_J13A_CTRL_HOST = $HostBinary
    $env:TETHERS_J13A_CTRL_CWD = $CallerDirectory
    $env:TETHERS_J13A_CTRL_CONFIG = $blockingConfig
    $env:TETHERS_J13A_CTRL_ENGINE = $EngineProxyPath
    $env:TETHERS_J13A_CTRL_PID_MARKER = $pidMarker
    $env:TETHERS_J13A_CTRL_RESULT = $ctrlResultPath

    $quotedControllerPath = '"' + $ctrlControllerPath + '"'
    $ctrlController = Start-Process `
        -FilePath "pwsh.exe" `
        -ArgumentList @("-NoProfile", "-File", $quotedControllerPath) `
        -WindowStyle Hidden `
        -PassThru
    if (-not $ctrlController.WaitForExit(20000)) {
        $ctrlController.Kill($true)
        $ctrlController.WaitForExit()
        throw "Ctrl+C controller exceeded 20000ms."
    }
    $ctrlController.WaitForExit()
    Wait-ForFile -Path $ctrlResultPath -TimeoutMs 1000
    $ctrlResult = Get-Content -Raw -LiteralPath $ctrlResultPath | ConvertFrom-Json
    if ($null -ne $ctrlResult.PSObject.Properties["controller_error"]) {
        throw "Ctrl+C controller failed: $($ctrlResult.controller_error)"
    }

    $pidLines = @(Get-Content -LiteralPath $pidMarker)
    $directPid = [int](($pidLines | Where-Object { $_ -like "direct=*" }) -replace "^direct=", "")
    $descendantPid = [int](($pidLines | Where-Object { $_ -like "descendant=*" }) -replace "^descendant=", "")
    $blockedResult = [pscustomobject]@{
        ExitCode = [int]$ctrlResult.exit_code
        Stdout = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($ctrlResult.stdout_base64))
        Stderr = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($ctrlResult.stderr_base64))
    }
    $interruptMilliseconds = [long]$ctrlResult.interrupt_ms
    $directGone = Wait-ForProcessExit -ProcessId $directPid
    $descendantGone = Wait-ForProcessExit -ProcessId $descendantPid

    Invoke-Case "18. Ctrl+C during blocked reading returns interrupted and exit 10" {
        $envelope = Assert-Envelope $blockedResult "interrupted" 10
        Assert-Equal $envelope.error.code "INTERRUPTED" "Interruption machine code mismatch."
        Assert-True ($interruptMilliseconds -le 5000) "Interruption exceeded 5 seconds."
    }
    Invoke-Case "19. direct child is gone after interruption" {
        Assert-True $directGone "Direct provider child $directPid survived interruption."
    }
    Invoke-Case "20. descendant child is gone after interruption" {
        Assert-True $descendantGone "Provider descendant $descendantPid survived interruption."
    }

    $effectsProvider = New-FixtureProvider -Mode "valid"
    $effectsConfig = Write-RuntimeConfig -Name "no trail replay effects" -Tethers @($validTether) -Providers @($effectsProvider)
    $beforeEffects = @(
        Get-ChildItem -LiteralPath $RuntimeDirectory -Recurse -Force |
            ForEach-Object { [System.IO.Path]::GetRelativePath($RuntimeDirectory, $_.FullName) } |
            Sort-Object
    )
    $effectsResult = Invoke-Check -ConfigPath $effectsConfig
    $afterEffects = @(
        Get-ChildItem -LiteralPath $RuntimeDirectory -Recurse -Force |
            ForEach-Object { [System.IO.Path]::GetRelativePath($RuntimeDirectory, $_.FullName) } |
            Sort-Object
    )
    Invoke-Case "21. check creates no Trail or replay state" {
        $null = Assert-Envelope $effectsResult "ok" 0
        $difference = @(Compare-Object -ReferenceObject $beforeEffects -DifferenceObject $afterEffects)
        Assert-Equal $difference.Count 0 "Check changed runtime filesystem state: $($difference | Out-String)"
        Assert-Equal @(Get-ChildItem -LiteralPath $TempRoot -Recurse -Force | Where-Object { $_.Name -match "^(trail|replay)" }).Count 0 "Trail or replay path was created."
    }

    $unknownResult = Invoke-Host -ArgumentList @("unknown-command")
    Invoke-Case "22. unknown command returns exit 2" {
        $null = Assert-Envelope $unknownResult "invalid_cli_usage" 2
    }
    $runnResult = Invoke-Host -ArgumentList @("runn", "engine.exe", "request.json")
    Invoke-Case "23. misspelled runn returns exit 2" {
        $null = Assert-Envelope $runnResult "invalid_cli_usage" 2
    }
    $legacyResult = Invoke-Host -ArgumentList @("__legacy")
    Invoke-Case "24. hidden __legacy remains reachable" {
        $null = Assert-Envelope $legacyResult "failed" 6
    }
    Invoke-Case "25. envelope contains no timestamp" {
        Assert-True (-not $unknownResult.Stdout.Contains("timestamp")) "Envelope contained a timestamp."
    }

    Write-Output ""
    Write-Output "J13A public acceptance: $Passed passed, 0 failed"
    foreach ($case in $CaseResults) {
        Write-Output $case
    }
    Write-Output "EVIDENCE real_engine=$RealEnginePath"
    Write-Output "EVIDENCE engine_proxy=$EngineProxyPath"
    Write-Output "EVIDENCE reviewed_manifest=$FixtureManifestPath"
    Write-Output "EVIDENCE reviewed_digest=$ReviewedDigest"
    Write-Output "EVIDENCE missing_tool_stdout=$($missingResult.Stdout.Trim())"
    Write-Output "EVIDENCE engine_marker=$($engineMethods -join ',')"
    Write-Output "EVIDENCE provider_marker=$($providerMethods -join ',')"
    Write-Output "EVIDENCE engine_hang_ms=$($engineHangStopwatch.ElapsedMilliseconds)"
    Write-Output "EVIDENCE provider_initialize_hang_ms=$($providerInitStopwatch.ElapsedMilliseconds)"
    Write-Output "EVIDENCE provider_tools_list_hang_ms=$($providerListStopwatch.ElapsedMilliseconds)"
    Write-Output "EVIDENCE interrupt_ms=$interruptMilliseconds"
    Write-Output "EVIDENCE direct_pid=$directPid direct_gone=$directGone"
    Write-Output "EVIDENCE descendant_pid=$descendantPid descendant_gone=$descendantGone"
    Write-Output "EVIDENCE trail_replay_changes=0"
    Write-Output "PASS test-j13a-check"
}
finally {
    foreach ($process in $ActiveProcesses) {
        try {
            if (-not $process.HasExited) {
                $process.Kill($true)
                $process.WaitForExit()
            }
            $process.Dispose()
        }
        catch {
        }
    }
    Remove-Item Env:TETHERS_J13A_ENGINE_TARGET -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_ENGINE_MARKER -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_ENGINE_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_CTRL_HOST -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_CTRL_CWD -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_CTRL_CONFIG -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_CTRL_ENGINE -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_CTRL_PID_MARKER -ErrorAction SilentlyContinue
    Remove-Item Env:TETHERS_J13A_CTRL_RESULT -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $TempRoot) {
        Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
