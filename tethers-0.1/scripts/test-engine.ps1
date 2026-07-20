Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
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
Write-Output "Engine responses match all fixture cases"
