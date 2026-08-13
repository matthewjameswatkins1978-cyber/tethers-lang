Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$TranscriptRoot = Join-Path $Root "protocol/mcp-transcripts"
$ServerExe = Join-Path $Root "engine-ocaml\_build\default\bin\tethers_mcp_main.exe"

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

function Assert-SemanticEqual {
    param(
        [Parameter(Mandatory = $true)]
        $Actual,

        [Parameter(Mandatory = $true)]
        $Expected,

        [Parameter(Mandatory = $true)]
        [string] $Context
    )

    $actualCanonical = ConvertTo-CanonicalJson $Actual
    $expectedCanonical = ConvertTo-CanonicalJson $Expected
    if ($actualCanonical -ne $expectedCanonical) {
        throw "Semantic JSON mismatch for ${Context}.`nExpected: $expectedCanonical`nActual:   $actualCanonical"
    }
}

function Assert-SemanticDifferent {
    param(
        [Parameter(Mandatory = $true)]
        $Left,

        [Parameter(Mandatory = $true)]
        $Right,

        [Parameter(Mandatory = $true)]
        [string] $Context
    )

    $leftCanonical = ConvertTo-CanonicalJson $Left
    $rightCanonical = ConvertTo-CanonicalJson $Right
    if ($leftCanonical -eq $rightCanonical) {
        throw "Expected semantic JSON difference for ${Context}, but values matched."
    }
}

function Read-JsonLines {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing JSONL transcript file: $Path"
    }

    $messages = @()
    $lineNumber = 0
    foreach ($line in [System.IO.File]::ReadLines($Path)) {
        $lineNumber += 1
        if ([string]::IsNullOrWhiteSpace($line)) {
            throw "Blank JSONL line at ${Path}:$lineNumber"
        }

        try {
            $messages += $line | ConvertFrom-Json -ErrorAction Stop
        }
        catch {
            throw "Invalid JSON at ${Path}:$lineNumber - $($_.Exception.Message)"
        }
    }

    return @($messages)
}

function Get-PropertyValue {
    param(
        [Parameter(Mandatory = $true)]
        $Object,

        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }

    return $property.Value
}

function Test-JsonRpcMessage {
    param(
        [Parameter(Mandatory = $true)]
        $Message,

        [Parameter(Mandatory = $true)]
        [string] $Context,

        [Parameter(Mandatory = $true)]
        [ValidateSet("stdin", "stdout")]
        [string] $Direction
    )

    Assert-True ((Get-PropertyValue $Message "jsonrpc") -eq "2.0") "$Context must declare jsonrpc 2.0"

    if ($Direction -eq "stdout") {
        $hasResult = $null -ne (Get-PropertyValue $Message "result")
        $hasError = $null -ne (Get-PropertyValue $Message "error")
        Assert-True (($hasResult -xor $hasError) -eq $true) "$Context stdout messages must contain exactly one of result or error"
        Assert-True ($null -ne (Get-PropertyValue $Message "id")) "$Context stdout response must preserve a request id"
    }
    else {
        Assert-True ($null -ne (Get-PropertyValue $Message "method")) "$Context stdin message must contain a method"
        $hasId = $null -ne (Get-PropertyValue $Message "id")
        if ($hasId) {
            Assert-True ([string]::IsNullOrWhiteSpace([string](Get-PropertyValue $Message "id")) -eq $false) "$Context request id must not be blank"
        }
    }
}

function Read-FixtureJson {
    param(
        [Parameter(Mandatory = $true)]
        [string] $RelativePath
    )

    Get-Content -Raw -LiteralPath (Join-Path $Root $RelativePath) | ConvertFrom-Json -ErrorAction Stop
}

function Test-ToolResult {
    param(
        [Parameter(Mandatory = $true)]
        $Message,

        [Parameter(Mandatory = $true)]
        [string] $ExpectedResponsePath,

        [Parameter(Mandatory = $true)]
        [string] $Context
    )

    $result = Get-PropertyValue $Message "result"
    Assert-True ($null -ne $result) "$Context must be a JSON-RPC result response"
    Assert-True ((Get-PropertyValue $result "isError") -eq $false) "$Context must keep Tethers planner errors as data, not MCP tool errors"

    $structured = Get-PropertyValue $result "structuredContent"
    Assert-True ($null -ne $structured) "$Context must include structuredContent"

    $content = @(Get-PropertyValue $result "content")
    Assert-True ($content.Count -eq 1) "$Context must include exactly one text content item"
    Assert-True ((Get-PropertyValue $content[0] "type") -eq "text") "$Context content item must be text"

    $textJson = Get-PropertyValue $content[0] "text"
    $parsedText = $textJson | ConvertFrom-Json -ErrorAction Stop

    Assert-SemanticEqual $parsedText $structured "$Context text content mirrors structuredContent"

    $expected = Read-FixtureJson $ExpectedResponsePath
    Assert-SemanticEqual $structured $expected "$Context structuredContent matches Tethers fixture $ExpectedResponsePath"

    $status = Get-PropertyValue $structured "status"
    if ($status -eq "matched") {
        $actions = @((Get-PropertyValue (Get-PropertyValue $structured "plan") "actions"))
        Assert-True ($actions.Count -gt 0) "$Context matched result must preserve Action array"
    }

    $trail = @(Get-PropertyValue $structured "trail")
    if ($trail.Count -gt 1) {
        $reversedTrail = @($trail)
        [array]::Reverse($reversedTrail)
        $changed = $structured.PSObject.Copy()
        $changed.trail = $reversedTrail
        Assert-SemanticDifferent $changed $structured "$Context Trail array order"
    }
}

function Run-ServerAndCompare {
    param(
        [Parameter(Mandatory = $true)]
        [string] $CaseName
    )

    $caseRoot = Join-Path $TranscriptRoot $CaseName
    $stdinPath = Join-Path $caseRoot "stdin.jsonl"
    $stdoutPath = Join-Path $caseRoot "stdout.jsonl"

    $stdin = Read-JsonLines $stdinPath
    $expectedStdout = Read-JsonLines $stdoutPath

    $stdoutTemp = Join-Path $env:TEMP "mcp_stdout_$CaseName.txt"
    $stderrTemp = Join-Path $env:TEMP "mcp_stderr_$CaseName.txt"

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $ServerExe
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    $process = [System.Diagnostics.Process]::Start($psi)

    $stdinText = [System.IO.File]::ReadAllText($stdinPath)
    $process.StandardInput.Write($stdinText)
    $process.StandardInput.Close()

    $stdoutText = $process.StandardOutput.ReadToEnd()
    $stderrText = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    [System.IO.File]::WriteAllText($stdoutTemp, $stdoutText)
    [System.IO.File]::WriteAllText($stderrTemp, $stderrText)

    $actualStdout = @()
    foreach ($line in ($stdoutText -split "`n")) {
        $trimmed = $line.TrimEnd("`r")
        if ([string]::IsNullOrWhiteSpace($trimmed)) {
            continue
        }
        try {
            $actualStdout += $trimmed | ConvertFrom-Json -ErrorAction Stop
        }
        catch {
            throw "$CaseName server produced invalid JSON on stdout: $trimmed"
        }
    }

    if ($stderrText -and $stderrText.Trim().Length -gt 0) {
        [Console]::Error.WriteLine("[$CaseName stderr] $($stderrText.Trim())")
    }

    Assert-True ($actualStdout.Count -eq $expectedStdout.Count) `
        "$CaseName expected $($expectedStdout.Count) stdout messages, got $($actualStdout.Count)"

    for ($i = 0; $i -lt $actualStdout.Count; $i += 1) {
        Assert-SemanticEqual $actualStdout[$i] $expectedStdout[$i] "$CaseName stdout[$i]"
    }
}

function Test-SemanticComparisonSelfChecks {
    $left = '{"b":2,"a":{"z":3,"y":[1,2]}}' | ConvertFrom-Json
    $right = '{"a":{"y":[1,2],"z":3},"b":2}' | ConvertFrom-Json
    Assert-SemanticEqual $left $right "object key order self-check"

    $arrayLeft = '{"items":[{"id":1},{"id":2}]}' | ConvertFrom-Json
    $arrayRight = '{"items":[{"id":2},{"id":1}]}' | ConvertFrom-Json
    Assert-SemanticDifferent $arrayLeft $arrayRight "array order self-check"

    [Console]::Out.WriteLine("PASS semantic object-key order ignored")
    [Console]::Out.WriteLine("PASS array order remains significant")
}

$requiredCases = @(
    "initialization-success",
    "initialization-success-2025-06-18",
    "incompatible-mcp-protocol-version",
    "tools-list",
    "evaluate-matched",
    "evaluate-not-matched",
    "evaluate-minimal-tethers-error",
    "evaluate-correlated-tethers-error",
    "malformed-tool-arguments",
    "unknown-tool",
    "call-before-initialization",
    "clean-eof-shutdown",
    "validate-valid",
    "validate-invalid",
    "validate-missing-source",
    "validate-together"
)

Test-SemanticComparisonSelfChecks

Assert-True (Test-Path -LiteralPath $ServerExe -PathType Leaf) "MCP server executable not found: $ServerExe"

foreach ($caseName in $requiredCases) {
    $caseRoot = Join-Path $TranscriptRoot $caseName
    Assert-True (Test-Path -LiteralPath $caseRoot -PathType Container) "Missing MCP transcript case directory: $caseName"
    Run-ServerAndCompare $caseName
    [Console]::Out.WriteLine("PASS $caseName")
}

[Console]::Out.WriteLine("MCP transcript server validation complete ($($requiredCases.Count) cases)")