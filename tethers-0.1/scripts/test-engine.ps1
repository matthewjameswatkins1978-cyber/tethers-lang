Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
$McpServerPath = Join-Path $EngineDir "_build/default/bin/tethers_mcp_main.exe"
$ProtocolDir = Join-Path $Root "protocol"

function Assert-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

function Assert-True {
    param(
        [Parameter(Mandatory = $true)]
        [bool] $Condition,

        [Parameter(Mandatory = $true)]
        [string] $Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function ConvertTo-CanonicalJson {
    param(
        [Parameter(Mandatory = $true)]
        $Value
    )

    function Convert-Node {
        param($Node)

        if ($null -eq $Node) {
            return $null
        }

        if ($Node -is [System.Collections.IEnumerable] -and
            $Node -isnot [string] -and
            $Node -isnot [System.Collections.IDictionary] -and
            $Node -isnot [pscustomobject]) {
            return @($Node | ForEach-Object { Convert-Node $_ })
        }

        if ($Node -is [pscustomobject]) {
            $ordered = [ordered]@{}
            foreach ($property in ($Node.PSObject.Properties | Sort-Object Name)) {
                $ordered[$property.Name] = Convert-Node $property.Value
            }
            return [pscustomobject]$ordered
        }

        if ($Node -is [System.Collections.IDictionary]) {
            $ordered = [ordered]@{}
            foreach ($key in ($Node.Keys | Sort-Object)) {
                $ordered[$key] = Convert-Node $Node[$key]
            }
            return [pscustomobject]$ordered
        }

        return $Node
    }

    Convert-Node $Value | ConvertTo-Json -Depth 100 -Compress
}

Assert-Command "opam"

Push-Location $EngineDir
try {
    & opam exec -- dune build
    if ($LASTEXITCODE -ne 0) {
        throw "Dune build failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "Dune build completed but engine executable was not found at $EnginePath"
}

if (-not (Test-Path -LiteralPath $McpServerPath -PathType Leaf)) {
    throw "Dune build completed but MCP server executable was not found at $McpServerPath"
}

function Invoke-EngineCase {
    param(
        [Parameter(Mandatory = $true)]
        [string] $CaseName,

        [Parameter(Mandatory = $true)]
        [string] $RequestPath,

        [Parameter(Mandatory = $true)]
        [string] $ExpectedPath
    )

    if (-not (Test-Path -LiteralPath $RequestPath -PathType Leaf)) {
        throw "Missing request fixture for ${CaseName}: $RequestPath"
    }

    if (-not (Test-Path -LiteralPath $ExpectedPath -PathType Leaf)) {
        throw "Missing expected response fixture for ${CaseName}: $ExpectedPath"
    }

    $request = Get-Content -Raw -LiteralPath $RequestPath | ConvertFrom-Json -ErrorAction Stop
    $requestLine = $request | ConvertTo-Json -Depth 100 -Compress
    $engineOutput = $requestLine | & $EnginePath
    if ($LASTEXITCODE -ne 0) {
        throw "Engine exited with code $LASTEXITCODE for case '$CaseName'."
    }

    $actualText = ($engineOutput -join "`n").Trim()
    if ($actualText -eq "") {
        throw "Engine produced no JSON output for case '$CaseName'."
    }

    try {
        $actual = $actualText | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Engine produced invalid JSON for case '$CaseName': $($_.Exception.Message)"
    }

    $expected = Get-Content -Raw -LiteralPath $ExpectedPath | ConvertFrom-Json -ErrorAction Stop

    $actualCanonical = ConvertTo-CanonicalJson $actual
    $expectedCanonical = ConvertTo-CanonicalJson $expected

    if ($actualCanonical -ne $expectedCanonical) {
        throw "Engine response did not semantically match expected fixture for case '$CaseName'.`nExpected: $expectedCanonical`nActual:   $actualCanonical"
    }

    [Console]::Out.WriteLine("PASS $CaseName")
    return [pscustomobject]@{
        RequestLine = $requestLine
        ActualText = $actualText
    }
}

function Invoke-McpValidate {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Source,

        [Parameter(Mandatory = $true)]
        [string] $RequestId
    )

    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $McpServerPath
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    $process = [System.Diagnostics.Process]::Start($psi)
    $messages = @(
        [ordered]@{
            jsonrpc = "2.0"
            id = "crlf-init"
            method = "initialize"
            params = [ordered]@{
                protocolVersion = "2025-11-25"
                capabilities = [ordered]@{}
                clientInfo = [ordered]@{ name = "tethers-crlf-regression"; version = "0.1" }
            }
        },
        [ordered]@{
            jsonrpc = "2.0"
            method = "notifications/initialized"
        },
        [ordered]@{
            jsonrpc = "2.0"
            id = $RequestId
            method = "tools/call"
            params = [ordered]@{
                name = "tethers.validate"
                arguments = [ordered]@{ source = $Source }
            }
        }
    )

    foreach ($message in $messages) {
        $process.StandardInput.WriteLine(($message | ConvertTo-Json -Depth 20 -Compress))
    }
    $process.StandardInput.Close()

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) {
        throw "MCP server exited with code $($process.ExitCode) during $RequestId. stderr: $stderr"
    }

    $responses = @()
    foreach ($line in ($stdout -split "`n")) {
        $trimmed = $line.TrimEnd("`r")
        if ([string]::IsNullOrWhiteSpace($trimmed)) {
            continue
        }
        $responses += $trimmed | ConvertFrom-Json -ErrorAction Stop
    }

    if ($responses.Count -ne 2) {
        throw "MCP validation expected two responses for $RequestId, got $($responses.Count)."
    }

    $validationResponse = @($responses | Where-Object { $_.id -eq $RequestId })
    if ($validationResponse.Count -ne 1) {
        throw "MCP validation response for $RequestId was not returned exactly once."
    }

    $result = $validationResponse[0].result
    if ($null -eq $result -or $result.isError -ne $false) {
        throw "MCP validation returned an MCP error for $RequestId."
    }

    return $result.structuredContent
}

$cases = @(
    [pscustomobject]@{
        Name = "top-level"
        RequestPath = Join-Path $ProtocolDir "request.json"
        ExpectedPath = Join-Path $ProtocolDir "expected-response.json"
    }
)

$casesRoot = Join-Path $ProtocolDir "cases"
if (Test-Path -LiteralPath $casesRoot -PathType Container) {
    $caseDirectories = Get-ChildItem -LiteralPath $casesRoot -Directory | Sort-Object Name
    foreach ($caseDirectory in $caseDirectories) {
        $cases += [pscustomobject]@{
            Name = $caseDirectory.Name
            RequestPath = Join-Path $caseDirectory.FullName "request.json"
            ExpectedPath = Join-Path $caseDirectory.FullName "expected-response.json"
        }
    }
}

$happyResult = $null
foreach ($case in $cases) {
    $result = Invoke-EngineCase -CaseName $case.Name -RequestPath $case.RequestPath -ExpectedPath $case.ExpectedPath
    if ($case.Name -eq "happy-path") {
        $happyResult = $result
    }
}

if ($null -eq $happyResult) {
    throw "Missing fixture case: happy-path"
}

$repeatOutput = $happyResult.RequestLine | & $EnginePath
if ($LASTEXITCODE -ne 0) {
    throw "Engine exited with code $LASTEXITCODE during happy-path determinism check."
}

$repeatText = ($repeatOutput -join "`n").Trim()
if ($repeatText -ne $happyResult.ActualText) {
    throw "Happy-path output was not deterministic across two evaluations."
}

Write-Output "PASS happy-path deterministic repeat"

$tetherLines = @(
    'tether "CRLF parser regression"',
    '',
    'anchor',
    '    coding.task_completed',
    '',
    'when',
    '    project.type is "software"',
    '    and task.changed_files greater_than 0',
    '',
    'do',
    '    lantern.task.record',
    '        project: anchor.project',
    '        task: anchor.task'
)
$lfSource = [string]::Join("`n", $tetherLines)
$crlfSource = [string]::Join("`r`n", $tetherLines)
$mixedBuilder = [System.Text.StringBuilder]::new()
$mixedSeparators = @("`r`n", "`n", "`r`n", "`n")
for ($index = 0; $index -lt $tetherLines.Count; $index += 1) {
    [void]$mixedBuilder.Append($tetherLines[$index])
    if ($index -lt ($tetherLines.Count - 1)) {
        [void]$mixedBuilder.Append($mixedSeparators[$index % $mixedSeparators.Count])
    }
}
$mixedSource = $mixedBuilder.ToString()

$lfValidation = Invoke-McpValidate -Source $lfSource -RequestId "validate-lf"
$crlfValidation = Invoke-McpValidate -Source $crlfSource -RequestId "validate-crlf"
$mixedValidation = Invoke-McpValidate -Source $mixedSource -RequestId "validate-mixed"

foreach ($validation in @($lfValidation, $crlfValidation, $mixedValidation)) {
    Assert-True ($validation.valid -eq $true) "Line-ending regression validation was not valid."
    Assert-True ($null -eq $validation.PSObject.Properties["error"]) "Line-ending regression returned a parse error."
}

$lfCanonical = ConvertTo-CanonicalJson $lfValidation
if ($lfCanonical -ne (ConvertTo-CanonicalJson $crlfValidation) -or
    $lfCanonical -ne (ConvertTo-CanonicalJson $mixedValidation)) {
    throw "LF, CRLF, and mixed-line-ending Tethers did not produce equivalent validation results."
}

Write-Output "PASS validate-lf"
Write-Output "PASS validate-crlf"
Write-Output "PASS validate-mixed"
Write-Output "PASS validate-line-ending-equivalence"
Write-Output "Engine responses match all fixture cases"
