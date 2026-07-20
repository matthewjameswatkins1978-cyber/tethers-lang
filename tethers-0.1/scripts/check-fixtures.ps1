Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")

function Test-JsonFile {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path
    )

    $resolved = Join-Path $Root $Path
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "Missing JSON fixture: $Path"
    }

    try {
        Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json -ErrorAction Stop | Out-Null
    }
    catch {
        throw "Invalid JSON fixture '$Path': $($_.Exception.Message)"
    }
}

Test-JsonFile "protocol/request.json"
Test-JsonFile "protocol/expected-response.json"

Write-Output "JSON fixtures are valid"
