param(
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$checkerPath = Join-Path $scriptDir "check-tethers-task-packet.ps1"

if ($SkipTests) {
    Write-Host "SKIP: test suite skipped by request"
    exit 0
}

$global:passed = 0
$global:failed = 0

function New-TestRepo {
    $root = New-Item -ItemType Directory -Path ([System.IO.Path]::Combine(
        [System.IO.Path]::GetTempPath(),
        "tethers-checker-test-" + [Guid]::NewGuid().ToString("n").Substring(0, 8)
    )) -Force
    Push-Location $root

    & git init --quiet 2>&1 | Out-Null
    & git config user.email "test@tethers.local"
    & git config user.name "Tethers Test Runner"

    New-Item -ItemType Directory -Path "docs\worker-notes" -Force | Out-Null
    New-Item -ItemType Directory -Path ".github\scripts" -Force | Out-Null

    return $root.FullName
}

function New-InitCommit {
    Set-Content -LiteralPath ".gitignore" -Value "# initial"
    & git add .gitignore
    & git commit --quiet -m "initial commit" 2>&1 | Out-Null
}

function Write-Packet {
    param(
        [string]$Status,
        [string]$BaseCommit,
        [string]$NotePath = "docs/worker-notes/test-note.md"
    )
    $content = @"
# Current Implementation Task

Control contract: ``1``
Task: ``Control Hardening Test``
Owner: ``OpenCode``
Model: ``Test Runner``
Status: ``$Status``
Task colour: ``Green``
Route: ``OpenCode test``
Worker note: ``$NotePath``
Base branch: ``test``
Base commit: ``$BaseCommit``
Implementation branch: ``test-branch``
OCaml switch path: ``N/A``
Rust toolchain: read exact channel from ``rust-toolchain.toml``; use plain Cargo (resolved by root pin); ``--locked`` mandatory
Toolchain preflight: ``pwsh -NoProfile -File scripts/check-dev-tools.ps1``

## Objective

Test the checker.

## Relevant background and existing behaviour

None.

## Required behaviour

1. Test

## Relevant components

None.

## Frozen decisions and invariants

None.

## Forbidden changes

None.

## Stop conditions

None.

## Expected pre-existing changes

None

## Acceptance criteria

1. Test

## Required verification

````powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
````
"@
    Set-Content -LiteralPath "docs/CURRENT_CLINE_TASK.md" -Value $content
}

function Write-Note {
    param(
        [string]$Status,
        [string]$BaseCommit,
        [string]$Checkpoint
    )
    $content = @"
# Worker Note

Task: ``Test Task``

Task packet: ``docs/CURRENT_CLINE_TASK.md``

Owner: ``OpenCode``

Status: ``$Status``

Base commit: ``$BaseCommit``

Implementation checkpoint: ``$Checkpoint``

## Requested outcome

Test.

## Changes made

Test.

## Decisions and assumptions

None.

## Evidence

Test.

## Discoveries

None.

## Remaining risks

None.

## Smallest next action

Stop.

## References

None.
"@
    Set-Content -LiteralPath "docs/worker-notes/test-note.md" -Value $content
}

function Commit-Closeout {
    $paths = @(
        "docs/CURRENT_CLINE_TASK.md",
        "docs/worker-notes/test-note.md"
    )
    & git add -- @paths
    & git commit --quiet -m "closeout docs" 2>&1 | Out-Null
}

function Assert-CheckerPass {
    param([string]$Label)
    try {
        $result = & $checkerPath *>&1
        Write-Host "PASS: $Label — $result"
        $global:passed++
    } catch {
        Write-Host "FAIL: $Label — $_"
        $global:failed++
    }
}

function Assert-CheckerFail {
    param([string]$Label)
    try {
        $result = & $checkerPath *>&1
        Write-Host "FAIL: $Label — checker passed unexpectedly: $result"
        $global:failed++
    } catch {
        Write-Host "PASS: $Label"
        $global:passed++
    }
}

# ═════════════════════════════════════════════════
# TEST A: COMPLETE + WORKTREE -> FAIL
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
Write-Packet -Status "COMPLETE" -BaseCommit $baseSha
Write-Note -Status "COMPLETE" -BaseCommit $baseSha -Checkpoint "WORKTREE"
Commit-Closeout
Assert-CheckerFail "TEST A: COMPLETE + WORKTREE checkpoint is rejected"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
# TEST B: BLOCKED + WORKTREE -> allowed
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
Write-Packet -Status "BLOCKED" -BaseCommit $baseSha
Write-Note -Status "BLOCKED" -BaseCommit $baseSha -Checkpoint "WORKTREE"
Commit-Closeout
Assert-CheckerPass "TEST B: BLOCKED + WORKTREE checkpoint is allowed"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
# TEST C: COMPLETE + nonexistent SHA -> FAIL
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
Write-Packet -Status "COMPLETE" -BaseCommit $baseSha
Write-Note -Status "COMPLETE" -BaseCommit $baseSha -Checkpoint "0000000000000000000000000000000000000000"
Commit-Closeout
Assert-CheckerFail "TEST C: COMPLETE + nonexistent SHA is rejected"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
# TEST D: COMPLETE + valid checkpoint, clean closeout -> PASS
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
# Implementation commit
New-Item -ItemType Directory -Path "src" -Force | Out-Null
Set-Content -LiteralPath "src/lib.rs" -Value "// production"
& git add -- src/lib.rs
& git commit --quiet -m "implementation checkpoint" 2>&1 | Out-Null
$checkpointSha = (& git rev-parse HEAD)
# Closeout docs on top
Write-Packet -Status "COMPLETE" -BaseCommit $baseSha
Write-Note -Status "COMPLETE" -BaseCommit $baseSha -Checkpoint $checkpointSha
Commit-Closeout
Assert-CheckerPass "TEST D: COMPLETE + valid checkpoint with closeout-only diff"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
# TEST E: COMPLETE + production changed after checkpoint -> FAIL
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
New-Item -ItemType Directory -Path "src" -Force | Out-Null
Set-Content -LiteralPath "src/lib.rs" -Value "// production"
& git add -- src/lib.rs
& git commit --quiet -m "implementation checkpoint" 2>&1 | Out-Null
$checkpointSha = (& git rev-parse HEAD)
Write-Packet -Status "COMPLETE" -BaseCommit $baseSha
Write-Note -Status "COMPLETE" -BaseCommit $baseSha -Checkpoint $checkpointSha
Set-Content -LiteralPath "src/lib.rs" -Value "// changed after checkpoint"
& git add -- src/lib.rs docs/CURRENT_CLINE_TASK.md docs/worker-notes/test-note.md
& git commit --quiet -m "tainted: production change after closeout" 2>&1 | Out-Null
Assert-CheckerFail "TEST E: COMPLETE + production changed after checkpoint is rejected"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
# TEST F: COMPLETE + arbitrary doc changed after checkpoint -> FAIL
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
New-Item -ItemType Directory -Path "src" -Force | Out-Null
Set-Content -LiteralPath "src/lib.rs" -Value "// production"
& git add -- src/lib.rs
& git commit --quiet -m "implementation checkpoint" 2>&1 | Out-Null
$checkpointSha = (& git rev-parse HEAD)
Write-Packet -Status "COMPLETE" -BaseCommit $baseSha
Write-Note -Status "COMPLETE" -BaseCommit $baseSha -Checkpoint $checkpointSha
Set-Content -LiteralPath "docs/AGENT_WORKFLOW.md" -Value "# Unrelated edit"
& git add -- docs/AGENT_WORKFLOW.md docs/CURRENT_CLINE_TASK.md docs/worker-notes/test-note.md
& git commit --quiet -m "tainted: unrelated doc change" 2>&1 | Out-Null
Assert-CheckerFail "TEST F: COMPLETE + non-closeout doc changed after checkpoint is rejected"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
# TEST G: COMPLETE + packet/worker-note only closeout -> PASS
# ═════════════════════════════════════════════════
$repo = New-TestRepo
New-InitCommit
$baseSha = (& git rev-parse HEAD)
New-Item -ItemType Directory -Path "src" -Force | Out-Null
Set-Content -LiteralPath "src/lib.rs" -Value "// production"
& git add -- src/lib.rs
& git commit --quiet -m "implementation checkpoint" 2>&1 | Out-Null
$checkpointSha = (& git rev-parse HEAD)
Write-Packet -Status "COMPLETE" -BaseCommit $baseSha
Write-Note -Status "COMPLETE" -BaseCommit $baseSha -Checkpoint $checkpointSha
Commit-Closeout
Assert-CheckerPass "TEST G: COMPLETE + packet and worker-note closeout only passes"
Pop-Location
Remove-Item -LiteralPath $repo -Recurse -Force -ErrorAction SilentlyContinue

# ═════════════════════════════════════════════════
Write-Host ("===== RESULTS =====")
Write-Host ("Passed: $global:passed")
Write-Host ("Failed: $global:failed")

if ($global:failed -gt 0) {
    Write-Host "One or more tests FAILED."
    exit 1
} else {
    Write-Host "All tests passed."
    exit 0
}
