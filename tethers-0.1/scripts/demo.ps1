Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$EngineDir = Join-Path $Root "engine-ocaml"
$HostDir = Join-Path $Root "host-rust"
$EnginePath = Join-Path $EngineDir "_build/default/bin/main.exe"
$RequestPath = Join-Path $Root "protocol/request.json"

function Assert-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found on PATH."
    }
}

Assert-Command "opam"
Assert-Command "cargo"

if (-not (Test-Path -LiteralPath $RequestPath -PathType Leaf)) {
    throw "Missing request fixture: protocol/request.json"
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

Push-Location $HostDir
try {
    $output = & cargo run -- $EnginePath $RequestPath
    if ($LASTEXITCODE -ne 0) {
        throw "Rust reference host demo failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}

$text = ($output -join "`n").Trim()
if ($text -eq "") {
    throw "Rust reference host produced no JSON output."
}

try {
    $response = $text | ConvertFrom-Json -ErrorAction Stop
}
catch {
    throw "Rust reference host produced invalid JSON: $($_.Exception.Message)"
}

if ($response.status -ne "matched") {
    throw "Demo completed but response status was '$($response.status)', expected 'matched'."
}

if ($response.execution_status -ne "completed") {
    throw "Demo completed but execution_status was '$($response.execution_status)', expected 'completed'."
}

$text
