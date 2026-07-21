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

function Test-JsonLinesFile {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path
    )

    $resolved = Join-Path $Root $Path
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "Missing JSONL fixture: $Path"
    }

    $lineNumber = 0
    foreach ($line in [System.IO.File]::ReadLines($resolved)) {
        $lineNumber += 1
        if ([string]::IsNullOrWhiteSpace($line)) {
            throw "Invalid JSONL fixture '$Path': blank line at $lineNumber"
        }

        try {
            $line | ConvertFrom-Json -ErrorAction Stop | Out-Null
        }
        catch {
            throw "Invalid JSONL fixture '$Path' at line ${lineNumber}: $($_.Exception.Message)"
        }
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

$jsonlFixturePaths = @()
$mcpTranscriptRoot = Join-Path $Root "protocol/mcp-transcripts"
if (Test-Path -LiteralPath $mcpTranscriptRoot -PathType Container) {
    $jsonlFixturePaths = Get-ChildItem -LiteralPath $mcpTranscriptRoot -Recurse -File -Filter "*.jsonl" |
        Sort-Object FullName |
        ForEach-Object {
            [System.IO.Path]::GetRelativePath($Root, $_.FullName)
        }
}

foreach ($fixturePath in $jsonlFixturePaths) {
    Test-JsonLinesFile $fixturePath
}

Write-Output "JSON fixtures are valid ($($fixturePaths.Count) JSON files, $($jsonlFixturePaths.Count) JSONL files)"
