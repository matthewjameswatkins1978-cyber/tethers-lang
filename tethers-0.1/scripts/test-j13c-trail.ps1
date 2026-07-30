Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$HostDir = Join-Path $RepoRoot "host-rust"
$HostPath = Join-Path $HostDir "target\debug\tethers-reference-host.exe"

$script:caseCount = 0
$script:passedCount = 0

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function Assert-Equal {
    param($Actual, $Expected, [string]$Message)
    if ($Actual -ne $Expected) { throw "$Message Expected '$Expected', got '$Actual'." }
}

function Invoke-Case {
    param([string]$Name, [scriptblock]$Body)
    $script:caseCount++
    Write-Output "TEST: $($script:caseCount). $Name"
    & $Body
    $script:passedCount++
    Write-Output "  PASS"
}

function Invoke-Host {
    param([string[]]$ArgumentList)
    $output = @(& $HostPath @ArgumentList 2>&1)
    $exitCode = $LASTEXITCODE
    [pscustomobject]@{
        ExitCode = $exitCode
        Stdout   = ($output -join "`n")
    }
}

function ConvertFrom-SingleEnvelope {
    param([Parameter(Mandatory = $true)]$Result,
          [string]$ExpectedCommand,
          [string]$ExpectedStatus,
          [int]$ExpectedExit)

    $lines = @($Result.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    Assert-Equal $lines.Count 1 "stdout must contain exactly one JSON document"
    $envelope = $lines[0] | ConvertFrom-Json -ErrorAction Stop
    Assert-Equal $envelope.schema "tethers.cli/1" "schema mismatch"
    Assert-Equal $envelope.command $ExpectedCommand "command mismatch"
    Assert-Equal $envelope.status $ExpectedStatus "status mismatch"
    Assert-Equal ([int]$envelope.exit_code) $ExpectedExit "embedded exit code mismatch"
    Assert-Equal $Result.ExitCode $ExpectedExit "process exit code mismatch"
    return $envelope
}

function Get-FileHash-SHA256 {
    param([string]$Path)
    (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLower()
}

$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-j13c-trail-$(New-Guid)"
New-Item -ItemType Directory -Path $TempRoot -Force | Out-Null

try {
    # ------------------------------------------------------------------
    # Fixture helpers
    # ------------------------------------------------------------------

    $ValidExecId = "exec_00000000-0000-4000-8000-000000000000"
    $OtherExecId = "exec_00000000-0000-4000-8000-000000000001"

    function New-TrailFile {
        param([string]$Name, [string]$Content)
        $path = Join-Path $TempRoot $Name
        [System.IO.File]::WriteAllText($path, $Content)
        return $path
    }

    # Build a trail with two matching entries separated by unrelated ones
    $trailWithMatches = New-TrailFile "trail-matches.jsonl" @"
{"execution_id":"$OtherExecId","kind":"other-1"}
{"execution_id":"$ValidExecId","kind":"match-1"}
{"kind":"audit-no-id"}
{"execution_id":"$ValidExecId","kind":"match-2"}
{"execution_id":"$OtherExecId","kind":"other-2"}
"@

    $trailNoMatches = New-TrailFile "trail-none.jsonl" @"
{"execution_id":"$OtherExecId","kind":"other"}
"@

    $trailWithUnicode = New-TrailFile "trail-unicode.jsonl" @"
{"execution_id":"$ValidExecId","kind":"entry","note":"path test"}
"@

    # Path with spaces
    $spaceDir = New-Item -ItemType Directory -Path (Join-Path $TempRoot "path with spaces") -Force
    $spaceTrail = Join-Path $spaceDir "trail.jsonl"
    [System.IO.File]::WriteAllText($spaceTrail, @"
{"execution_id":"$ValidExecId","kind":"unicode-path"}
"@)

    # ------------------------------------------------------------------
    # Case 1: Happy path - two matching entries
    # ------------------------------------------------------------------
    Invoke-Case "two matching entries returned in order" {
        $hashBefore = Get-FileHash-SHA256 $trailWithMatches
        $result = Invoke-Host "trail", "--trail", $trailWithMatches, "--execution-id", $ValidExecId
        $hashAfter = Get-FileHash-SHA256 $trailWithMatches
        $env = ConvertFrom-SingleEnvelope $result "trail" "ok" 0
        Assert-Equal $env.data.entry_count 2 "entry count"
        Assert-Equal $env.data.entries[0].kind "match-1" "first entry"
        Assert-Equal $env.data.entries[1].kind "match-2" "second entry"
        Assert-True ($env.data.trail_path.EndsWith((Get-Item $trailWithMatches).Name) -or $env.data.trail_path.Contains("trail-matches.jsonl")) "trail_path contains filename"
        Assert-True ((-not $env.PSObject.Properties["error"]) -or ($null -eq $env.error)) "no error"
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged"
    }

    # ------------------------------------------------------------------
    # Case 2: Path with spaces
    # ------------------------------------------------------------------
    Invoke-Case "path with spaces" {
        $hashBefore = Get-FileHash-SHA256 $spaceTrail
        $result = Invoke-Host "trail", "--trail", $spaceTrail, "--execution-id", $ValidExecId
        $hashAfter = Get-FileHash-SHA256 $spaceTrail
        $env = ConvertFrom-SingleEnvelope $result "trail" "ok" 0
        Assert-Equal $env.data.entry_count 1 "entry count"
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged"
    }

    # ------------------------------------------------------------------
    # Case 3: Options in reverse order
    # ------------------------------------------------------------------
    Invoke-Case "options in reverse order" {
        $hashBefore = Get-FileHash-SHA256 $trailWithMatches
        $result = Invoke-Host "trail", "--execution-id", $ValidExecId, "--trail", $trailWithMatches
        $hashAfter = Get-FileHash-SHA256 $trailWithMatches
        $env = ConvertFrom-SingleEnvelope $result "trail" "ok" 0
        Assert-Equal $env.data.entry_count 2 "entry count"
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged"
    }

    # ------------------------------------------------------------------
    # Case 4: Exactly one stdout JSON document
    # ------------------------------------------------------------------
    Invoke-Case "exactly one stdout JSON document" {
        $result = Invoke-Host "trail", "--trail", $trailWithMatches, "--execution-id", $ValidExecId
        $lines = @($result.Stdout -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        Assert-Equal $lines.Count 1 "line count must be exactly 1"
    }

    # ------------------------------------------------------------------
    # Case 5: Zero matching entries - not_found
    # ------------------------------------------------------------------
    Invoke-Case "zero matching entries returns not_found" {
        $hashBefore = Get-FileHash-SHA256 $trailNoMatches
        $result = Invoke-Host "trail", "--trail", $trailNoMatches, "--execution-id", $ValidExecId
        $hashAfter = Get-FileHash-SHA256 $trailNoMatches
        $env = ConvertFrom-SingleEnvelope $result "trail" "not_found" 9
        Assert-Equal $env.error.code "EXECUTION_NOT_FOUND" "error code"
        Assert-Equal $env.error.field "--execution-id" "error field"
        Assert-Equal $env.data.entry_count 0 "entry count zero"
        Assert-Equal $env.data.execution_id $ValidExecId "execution_id in data"
        Assert-True ((-not $env.data.PSObject.Properties["entries"])) "no entries in not_found"
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged"
    }

    # ------------------------------------------------------------------
    # Case 6: Malformed execution ID - exit 3
    # ------------------------------------------------------------------
    Invoke-Case "malformed execution ID returns exit 3" {
        $result = Invoke-Host "trail", "--trail", $trailWithMatches, "--execution-id", "not-a-valid-id"
        $env = ConvertFrom-SingleEnvelope $result "trail" "invalid_data" 3
        Assert-Equal $env.error.code "EXECUTION_ID_INVALID" "error code"
        Assert-Equal $env.error.field "--execution-id" "error field"
    }

    # ------------------------------------------------------------------
    # Case 7: Relative path - exit 3
    # ------------------------------------------------------------------
    Invoke-Case "relative trail path returns exit 3" {
        $result = Invoke-Host "trail", "--trail", "relative/path.jsonl", "--execution-id", $ValidExecId
        $env = ConvertFrom-SingleEnvelope $result "trail" "invalid_data" 3
        Assert-Equal $env.error.code "TRAIL_NOT_ABSOLUTE" "error code"
        Assert-Equal $env.error.field "--trail" "error field"
        # Prove no file or directory was created
        Assert-True (-not (Test-Path "relative/path.jsonl")) "no file created"
        Assert-True (-not (Test-Path "relative")) "no directory created"
    }

    # ------------------------------------------------------------------
    # Case 8: Missing file - exit 9
    # ------------------------------------------------------------------
    Invoke-Case "missing trail file returns exit 9" {
        $missing = Join-Path $TempRoot "does-not-exist.jsonl"
        $result = Invoke-Host "trail", "--trail", $missing, "--execution-id", $ValidExecId
        $env = ConvertFrom-SingleEnvelope $result "trail" "not_found" 9
        Assert-Equal $env.error.code "TRAIL_NOT_FOUND" "error code"
        Assert-Equal $env.error.field "--trail" "error field"
        Assert-True (-not (Test-Path $missing)) "file not created"
    }

    # ------------------------------------------------------------------
    # Case 9: Malformed JSON line - exit 8
    # ------------------------------------------------------------------
    Invoke-Case "malformed line returns audit_failed exit 8" {
        $malformedPath = New-TrailFile "trail-malformed.jsonl" @"
{"execution_id":"$ValidExecId","kind":"good"}
{this is not json}
"@
        $hashBefore = Get-FileHash-SHA256 $malformedPath
        $result = Invoke-Host "trail", "--trail", $malformedPath, "--execution-id", $ValidExecId
        $hashAfter = Get-FileHash-SHA256 $malformedPath
        $env = ConvertFrom-SingleEnvelope $result "trail" "audit_failed" 8
        Assert-Equal $env.error.code "TRAIL_INVALID" "error code"
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged"
    }

    # ------------------------------------------------------------------
    # Case 10: Duplicate key - exit 8
    # ------------------------------------------------------------------
    Invoke-Case "duplicate JSON key returns audit_failed exit 8" {
        $dupPath = New-TrailFile "trail-dupe.jsonl" @"
{"execution_id":"$ValidExecId","execution_id":"$ValidExecId"}
"@
        $hashBefore = Get-FileHash-SHA256 $dupPath
        $result = Invoke-Host "trail", "--trail", $dupPath, "--execution-id", $ValidExecId
        $hashAfter = Get-FileHash-SHA256 $dupPath
        $env = ConvertFrom-SingleEnvelope $result "trail" "audit_failed" 8
        Assert-Equal $env.error.code "TRAIL_INVALID" "error code"
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged"
    }

    # ------------------------------------------------------------------
    # Case 11: No timestamp in any envelope
    # ------------------------------------------------------------------
    Invoke-Case "no timestamp in success or error envelopes" {
        $result = Invoke-Host "trail", "--trail", $trailWithMatches, "--execution-id", $ValidExecId
        Assert-True ($result.Stdout -notmatch "timestamp") "success: no timestamp"

        $badResult = Invoke-Host "trail", "--trail", $trailWithMatches, "--execution-id", "bad"
        Assert-True ($badResult.Stdout -notmatch "timestamp") "error: no timestamp"
    }

    # ------------------------------------------------------------------
    # Case 12: Unknown command returns exit 2
    # ------------------------------------------------------------------
    Invoke-Case "unknown command returns exit 2" {
        $result = Invoke-Host "nonexistent"
        Assert-Equal $result.ExitCode 2 "exit code"
        $env = $result.Stdout | ConvertFrom-Json
        Assert-Equal $env.status "invalid_cli_usage" "status"
        Assert-Equal $env.exit_code 2 "embedded exit code"
    }

    # ------------------------------------------------------------------
    # Case 13: Misspelled command returns exit 2
    # ------------------------------------------------------------------
    Invoke-Case "misspelled trail command returns exit 2" {
        $result = Invoke-Host "traill"
        Assert-Equal $result.ExitCode 2 "exit code"
        $env = $result.Stdout | ConvertFrom-Json
        Assert-Equal $env.status "invalid_cli_usage" "status"
    }

    # ------------------------------------------------------------------
    # Case 14: Hidden legacy command remains hidden from help
    # ------------------------------------------------------------------
    Invoke-Case "hidden legacy command not in help" {
        $result = Invoke-Host "--help"
        Assert-True ($result.Stdout -notmatch "__legacy") "legacy not in help"
        Assert-True ($result.Stdout -notmatch "provision-replay") "provision-replay not in help"
    }

    # ------------------------------------------------------------------
    # Case 15: Valid audit entries without execution_id are skipped
    # ------------------------------------------------------------------
    Invoke-Case "audit entries without execution_id do not cause failure" {
        $auditPath = New-TrailFile "trail-audit.jsonl" @"
{"kind":"event_admitted","event_id":"evt-1","correlation_id":"evt-1","source":"external","generation":0,"processing":"continued","timestamp_unix_ms":1}
{"execution_id":"$ValidExecId","kind":"action_intent"}
"@
        $result = Invoke-Host "trail", "--trail", $auditPath, "--execution-id", $ValidExecId
        $env = ConvertFrom-SingleEnvelope $result "trail" "ok" 0
        Assert-Equal $env.data.entry_count 1 "only the matching entry"
    }

    # ------------------------------------------------------------------
    # Case 16: Trail SHA-256 unchanged after failed inspection
    # ------------------------------------------------------------------
    Invoke-Case "Trail SHA-256 unchanged after all failing inspections" {
        $singlePath = New-TrailFile "trail-single.jsonl" @"
{"execution_id":"$ValidExecId","kind":"test"}
"@
        $hashBefore = Get-FileHash-SHA256 $singlePath
        # Run a failing inspection
        $null = Invoke-Host "trail", "--trail", $singlePath, "--execution-id", $OtherExecId
        $hashAfter = Get-FileHash-SHA256 $singlePath
        Assert-Equal $hashBefore $hashAfter "SHA-256 unchanged after not_found"
    }

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    Write-Output ""
    Write-Output "============================================"
    Write-Output "TOTAL: $caseCount cases, $passedCount passed, 0 failed"
    Write-Output "============================================"
}
finally {
    Remove-Item -Recurse -Force $TempRoot -ErrorAction SilentlyContinue
}
