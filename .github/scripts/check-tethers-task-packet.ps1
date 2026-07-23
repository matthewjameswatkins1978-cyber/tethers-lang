param(
    [string]$PacketPath = "docs/CURRENT_CLINE_TASK.md",
    [switch]$SkipWorktreeCheck
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-Git {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)

    $output = & git @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
    return @($output)
}

$repositoryRoot = @(Invoke-Git rev-parse --show-toplevel)[0].Trim()
$packetFullPath = Join-Path $repositoryRoot $PacketPath
if (-not (Test-Path -LiteralPath $packetFullPath -PathType Leaf)) {
    throw "Task packet not found: $PacketPath"
}

$packet = Get-Content -LiteralPath $packetFullPath -Raw
$baseMatch = [regex]::Match(
    $packet,
    '(?m)^Base commit:\s*`([0-9a-fA-F]{40})`\s*$'
)
if (-not $baseMatch.Success) {
    throw "Task packet must contain one full 40-character Base commit SHA."
}

$baseCommit = $baseMatch.Groups[1].Value.ToLowerInvariant()
$headCommit = @(Invoke-Git rev-parse HEAD)[0].Trim().ToLowerInvariant()

& git cat-file -e "$baseCommit`^{commit}"
if ($LASTEXITCODE -ne 0) {
    throw "Base commit does not identify a local commit: $baseCommit"
}

& git merge-base --is-ancestor $baseCommit $headCommit
if ($LASTEXITCODE -ne 0) {
    throw "Base commit $baseCommit is not an ancestor of HEAD $headCommit."
}

$planningPaths = @(
    "docs/CURRENT_CLINE_TASK.md",
    "docs/COPILOT_TRIAL.md"
)

if ($baseCommit -ne $headCommit) {
    $descendantPaths = @(
        Invoke-Git diff --name-only "$baseCommit..$headCommit" --
    ) | Where-Object { $_ -ne "" }
    $unexpectedDescendants = @(
        $descendantPaths | Where-Object { $_ -notin $planningPaths }
    )
    if ($unexpectedDescendants.Count -gt 0) {
        throw (
            "Commits after Base commit change non-planning paths: " +
            ($unexpectedDescendants -join ", ")
        )
    }
}

if (-not $SkipWorktreeCheck) {
    $sectionMatch = [regex]::Match(
        $packet,
        '(?ms)^## Expected pre-existing changes\s*(.*?)(?=^## |\z)'
    )
    if (-not $sectionMatch.Success) {
        throw "Task packet is missing Expected pre-existing changes."
    }

    $expectedSection = $sectionMatch.Groups[1].Value
    if ($expectedSection -match '(?im)^\s*None\b') {
        $expectedPaths = @()
    }
    else {
        $expectedPaths = @(
            [regex]::Matches(
                $expectedSection,
                '(?m)^-\s+`([^`]+)`\s*$'
            ) | ForEach-Object { $_.Groups[1].Value }
        )
    }

    $statusLines = @(Invoke-Git status --porcelain=v1 --untracked-files=all)
    $actualPaths = @(
        foreach ($line in $statusLines) {
            if ($line.Length -lt 4) {
                continue
            }
            $path = $line.Substring(3)
            if ($path.Contains(" -> ")) {
                $path = $path.Split(" -> ")[-1]
            }
            $path = $path.Trim('"').Replace('\', '/')
            if ($path -notin $planningPaths) {
                $path
            }
        }
    )

    $expectedPaths = @($expectedPaths | Sort-Object -Unique)
    $actualPaths = @($actualPaths | Sort-Object -Unique)
    $missing = @($expectedPaths | Where-Object { $_ -notin $actualPaths })
    $unexpected = @($actualPaths | Where-Object { $_ -notin $expectedPaths })
    if ($missing.Count -gt 0 -or $unexpected.Count -gt 0) {
        throw (
            "Expected dirty paths do not match live Git state. Missing: [" +
            ($missing -join ", ") + "]. Unexpected: [" +
            ($unexpected -join ", ") + "]."
        )
    }
}

Write-Host (
    "PASS task packet consistency: base {0}, HEAD {1}" -f
    $baseCommit.Substring(0, 7),
    $headCommit.Substring(0, 7)
)
