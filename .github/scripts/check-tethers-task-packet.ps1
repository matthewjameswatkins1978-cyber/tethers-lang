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

function Get-Field {
    param(
        [string]$Content,
        [string]$Name
    )

    $pattern = '(?m)^{0}:\s*`([^`]+)`\s*$' -f [regex]::Escape($Name)
    $match = [regex]::Match($Content, $pattern)
    if (-not $match.Success) {
        throw "Task packet must contain '${Name}: ``value``'."
    }
    return $match.Groups[1].Value.Trim()
}

function Get-Section {
    param(
        [string]$Content,
        [string]$Name
    )

    $pattern = '(?ms)^## {0}\s*(.*?)(?=^## |\z)' -f [regex]::Escape($Name)
    $match = [regex]::Match($Content, $pattern)
    if (-not $match.Success) {
        throw "Task packet is missing section: $Name"
    }
    return $match.Groups[1].Value.Trim()
}

function Assert-WorkerNote {
    param(
        [string]$RepositoryRoot,
        [string]$RelativePath,
        [string]$ExpectedTaskStatus,
        [string]$ExpectedOwner,
        [string]$ExpectedBaseCommit,
        [string]$ExpectedPacketPath
    )

    if ($RelativePath -notmatch '^docs/worker-notes/[a-z0-9][a-z0-9._-]*\.md$') {
        throw (
            "Worker note must be a safe Markdown path under docs/worker-notes/: " +
            $RelativePath
        )
    }

    $fullPath = Join-Path $RepositoryRoot $RelativePath
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Required worker note does not exist: $RelativePath"
    }

    $note = Get-Content -LiteralPath $fullPath -Raw
    $requiredNoteFields = @(
        "Task",
        "Task packet",
        "Owner",
        "Status",
        "Base commit",
        "Implementation checkpoint"
    )
    foreach ($field in $requiredNoteFields) {
        $fieldValue = Get-Field -Content $note -Name $field
        if ($fieldValue.Contains("<")) {
            throw "Worker note contains an unresolved placeholder in: $field"
        }
    }

    $requiredNoteSections = @(
        "Requested outcome",
        "Changes made",
        "Decisions and assumptions",
        "Evidence",
        "Discoveries",
        "Remaining risks",
        "Smallest next action",
        "References"
    )
    foreach ($section in $requiredNoteSections) {
        $body = Get-Section -Content $note -Name $section
        if ([string]::IsNullOrWhiteSpace($body)) {
            throw "Worker note section is empty: $section"
        }
        if (
            $body -match '(?i)^(state|list|record|give|include|say)\s+(what|exact|only|one|unexpected)'
        ) {
            throw "Worker note still contains template instructions: $section"
        }
    }

    $noteOwner = Get-Field -Content $note -Name "Owner"
    if ($noteOwner -ne $ExpectedOwner) {
        throw "Worker note Owner does not match task packet Owner."
    }
    $noteBaseCommit = Get-Field -Content $note -Name "Base commit"
    if ($noteBaseCommit.ToLowerInvariant() -ne $ExpectedBaseCommit) {
        throw "Worker note Base commit does not match the task packet."
    }
    $notePacketPath = Get-Field -Content $note -Name "Task packet"
    if ($notePacketPath -ne $ExpectedPacketPath) {
        throw "Worker note Task packet does not match the checked packet path."
    }
    $implementationCheckpoint = Get-Field `
        -Content $note `
        -Name "Implementation checkpoint"
    if (
        $implementationCheckpoint -ne "WORKTREE" -and
        $implementationCheckpoint -notmatch '^[0-9a-fA-F]{40}$'
    ) {
        throw (
            "Implementation checkpoint must be WORKTREE or one full " +
            "40-character commit SHA."
        )
    }

    $noteStatus = Get-Field -Content $note -Name "Status"
    if ($ExpectedTaskStatus -eq "BLOCKED" -and $noteStatus -ne "BLOCKED") {
        throw "A BLOCKED task requires a BLOCKED worker note."
    }
    if (
        $ExpectedTaskStatus -in @("COMPLETE", "ACCEPTED", "REJECTED") -and
        $noteStatus -ne "COMPLETE"
    ) {
        throw "$ExpectedTaskStatus requires a COMPLETE worker note."
    }
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

$controlV1 = $packet -match '(?m)^Control contract:\s*`1`\s*$'
$taskStatus = $null
$workerNotePath = $null

if ($controlV1) {
    $taskStatus = Get-Field -Content $packet -Name "Status"
    $taskColour = Get-Field -Content $packet -Name "Task colour"
    $owner = Get-Field -Content $packet -Name "Owner"
    $route = Get-Field -Content $packet -Name "Route"
    $workerNotePath = Get-Field -Content $packet -Name "Worker note"

    $validStatuses = @(
        "PROPOSED",
        "READY",
        "IN_PROGRESS",
        "BLOCKED",
        "COMPLETE",
        "ACCEPTED",
        "REJECTED"
    )
    if ($taskStatus -notin $validStatuses) {
        throw "Invalid task Status: $taskStatus"
    }
    if ($taskColour -notin @("Green", "Amber", "Red")) {
        throw "Invalid Task colour: $taskColour"
    }
    if (
        [string]::IsNullOrWhiteSpace($owner) -or
        $owner.Contains("<") -or
        $owner.Contains(",") -or
        $owner -match '(?i)\s+and\s+'
    ) {
        throw "Owner must name exactly one implementation owner."
    }
    if ([string]::IsNullOrWhiteSpace($route)) {
        throw "Route must name the current worker/tool route."
    }

    $requiredPacketSections = @(
        "Objective",
        "Relevant background and existing behaviour",
        "Required behaviour",
        "Relevant components",
        "Frozen decisions and invariants",
        "Acceptance criteria",
        "Required verification",
        "Forbidden changes",
        "Stop conditions",
        "Expected pre-existing changes"
    )
    foreach ($section in $requiredPacketSections) {
        $body = Get-Section -Content $packet -Name $section
        if ([string]::IsNullOrWhiteSpace($body)) {
            throw "Task packet section is empty: $section"
        }
    }

    $requiredCount = [regex]::Matches(
        (Get-Section -Content $packet -Name "Required behaviour"),
        '(?m)^\d+\.\s+'
    ).Count
    $acceptanceCount = [regex]::Matches(
        (Get-Section -Content $packet -Name "Acceptance criteria"),
        '(?m)^\d+\.\s+'
    ).Count
    if ($requiredCount -eq 0 -or $acceptanceCount -lt $requiredCount) {
        throw (
            "Control-v1 packets require at least one numbered acceptance " +
            "criterion for every numbered required behaviour. Required: " +
            "$requiredCount; acceptance: $acceptanceCount."
        )
    }

    if ($taskStatus -in @("BLOCKED", "COMPLETE", "ACCEPTED", "REJECTED")) {
        Assert-WorkerNote `
            -RepositoryRoot $repositoryRoot `
            -RelativePath $workerNotePath `
            -ExpectedTaskStatus $taskStatus `
            -ExpectedOwner $owner `
            -ExpectedBaseCommit $baseCommit `
            -ExpectedPacketPath $PacketPath
    }
}

$planningPaths = @(
    "docs/CURRENT_CLINE_TASK.md",
    "docs/COPILOT_TRIAL.md",
    "docs/PROJECT_DASHBOARD.md"
)

if (
    $baseCommit -ne $headCommit -and
    (-not $controlV1 -or $taskStatus -in @("PROPOSED", "READY"))
) {
    $descendantPaths = @(
        Invoke-Git diff --name-only "$baseCommit..$headCommit" --
    ) | Where-Object { $_ -ne "" }
    $unexpectedDescendants = @(
        $descendantPaths | Where-Object { $_ -notin $planningPaths }
    )
    if ($unexpectedDescendants.Count -gt 0) {
        throw (
            "Pre-work commits after Base commit change non-planning paths: " +
            ($unexpectedDescendants -join ", ")
        )
    }
}

$shouldCheckPreWorktree = (
    -not $SkipWorktreeCheck -and
    (-not $controlV1 -or $taskStatus -in @("PROPOSED", "READY"))
)

if ($shouldCheckPreWorktree) {
    $expectedSection = Get-Section `
        -Content $packet `
        -Name "Expected pre-existing changes"
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
            "Expected dirty paths do not match live pre-work Git state. Missing: [" +
            ($missing -join ", ") + "]. Unexpected: [" +
            ($unexpected -join ", ") + "]."
        )
    }
}

$contractLabel = if ($controlV1) { "control-v1/$taskStatus" } else { "legacy" }
Write-Host (
    "PASS task packet consistency ({0}): base {1}, HEAD {2}" -f
    $contractLabel,
    $baseCommit.Substring(0, 7),
    $headCommit.Substring(0, 7)
)
