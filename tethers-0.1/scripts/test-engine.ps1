Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
$RequestPath = Join-Path $Root "protocol/request.json"
$ExpectedPath = Join-Path $Root "protocol/expected-response.json"

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

if (-not (Test-Path -LiteralPath $RequestPath -PathType Leaf)) {
    throw "Missing request fixture: protocol/request.json"
}

if (-not (Test-Path -LiteralPath $ExpectedPath -PathType Leaf)) {
    throw "Missing expected response fixture: protocol/expected-response.json"
}

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

$request = Get-Content -Raw -LiteralPath $RequestPath | ConvertFrom-Json -ErrorAction Stop
$requestLine = $request | ConvertTo-Json -Depth 100 -Compress
$engineOutput = $requestLine | & $EnginePath
if ($LASTEXITCODE -ne 0) {
    throw "Engine exited with code $LASTEXITCODE."
}

$actualText = ($engineOutput -join "`n").Trim()
if ($actualText -eq "") {
    throw "Engine produced no JSON output."
}

try {
    $actual = $actualText | ConvertFrom-Json -ErrorAction Stop
}
catch {
    throw "Engine produced invalid JSON: $($_.Exception.Message)"
}

$expected = Get-Content -Raw -LiteralPath $ExpectedPath | ConvertFrom-Json -ErrorAction Stop

$actualCanonical = ConvertTo-CanonicalJson $actual
$expectedCanonical = ConvertTo-CanonicalJson $expected

if ($actualCanonical -ne $expectedCanonical) {
    throw "Engine response did not semantically match protocol/expected-response.json.`nExpected: $expectedCanonical`nActual:   $actualCanonical"
}

Write-Output "Engine response matches the frozen fixture"
