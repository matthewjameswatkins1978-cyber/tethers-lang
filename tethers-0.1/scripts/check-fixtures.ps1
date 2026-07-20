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

$fixturePaths = @(
    "protocol/request.json"
    "protocol/expected-response.json"
)

$casesRoot = Join-Path $Root "protocol/cases"
if (Test-Path -LiteralPath $casesRoot -PathType Container) {
    $caseFixtures = Get-ChildItem -LiteralPath $casesRoot -Recurse -File -Filter "*.json" |
        Sort-Object FullName |
        ForEach-Object {
            [System.IO.Path]::GetRelativePath($Root, $_.FullName)
        }
    $fixturePaths += $caseFixtures
}

foreach ($fixturePath in $fixturePaths) {
    Test-JsonFile $fixturePath
}

Write-Output "JSON fixtures are valid ($($fixturePaths.Count) files)"
