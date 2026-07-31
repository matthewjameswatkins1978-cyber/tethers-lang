Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Here = Split-Path -Parent $MyInvocation.MyCommand.Definition
$SourceRunner = Join-Path $Here "verify-0.2.ps1"
if (-not (Test-Path -LiteralPath $SourceRunner -PathType Leaf)) {
    throw "Source runner not found: $SourceRunner"
}

$SuiteMap = [ordered]@{
    "J13A" = "test-j13a-check.ps1"
    "J13B" = "test-j13b-run.ps1"
    "J13C" = "test-j13c-trail.ps1"
    "J14A" = "test-j14a-complete-scenario.ps1"
    "J14B" = "test-j14b-negative-matrix.ps1"
    "J14C" = "test-j14c-real-file-move.ps1"
}
$CanonicalOrder = @("J13A", "J13B", "J13C", "J14A", "J14B", "J14C")

$stamp = [guid]::NewGuid().ToString("N").Substring(0, 8)
$Root = Join-Path $env:TEMP ("Tethers Verify 测试 " + $stamp)

$script:assertions = 0
function Assert {
    param([bool]$Condition, [string]$Message)
    $script:assertions++
    if (-not $Condition) {
        throw "ASSERTION FAILED: $Message"
    }
}

function Clean-Markers {
    Get-ChildItem -LiteralPath $Root -Filter "*.marker" -ErrorAction SilentlyContinue |
        Remove-Item -Force
}

function Write-Stub {
    param(
        [string]$Id,
        [int]$ExitCode,
        [string[]]$Lines
    )
    $fileName = $SuiteMap[$Id]
    $path = Join-Path $Root $fileName
    $markerBase = [System.IO.Path]::GetFileNameWithoutExtension($fileName)
    $body = [System.Collections.Generic.List[string]]::new()
    $body.Add('Set-StrictMode -Version Latest')
    $body.Add('$ErrorActionPreference = "Stop"')
    $body.Add('$marker = Join-Path $PSScriptRoot "' + $markerBase + '.marker"')
    $body.Add('Set-Content -LiteralPath $marker -Value "1" -Encoding utf8NoBOM')
    foreach ($ln in $Lines) {
        $body.Add('Write-Output "' + $ln + '"')
    }
    $body.Add('exit ' + $ExitCode)
    Set-Content -LiteralPath $path -Value ($body -join "`n") -Encoding utf8NoBOM
}

function Invoke-Verifier {
    param(
        [string]$RunnerPath,
        [string[]]$RunnerArgs
    )
    Push-Location $env:TEMP
    try {
        $raw = @(& pwsh.exe -NoProfile -ExecutionPolicy Bypass -File $RunnerPath @RunnerArgs 2>&1)
        $code = $LASTEXITCODE
        return @($raw, $code)
    }
    finally {
        Pop-Location
    }
}

function Count-Markers {
    return @(Get-ChildItem -LiteralPath $Root -Filter "*.marker" -ErrorAction SilentlyContinue).Count
}

$script:executed = @()

function Run-Row {
    param(
        [string]$Id,
        [scriptblock]$Block
    )
    Write-Output "ROW: $Id"
    & $Block
    Write-Output "  PASS"
    $script:executed += $Id
}

try {
    New-Item -ItemType Directory -Path $Root | Out-Null
    $script:RunnerCopy = Join-Path $Root "verify-0.2.ps1"
    Copy-Item -LiteralPath $SourceRunner -Destination $script:RunnerCopy

    foreach ($id in $CanonicalOrder) {
        Write-Stub $id 0 @("baseline line for $id")
    }

    Run-Row 'R01' {
        Clean-Markers
        $out, $code = Invoke-Verifier -RunnerPath $script:RunnerCopy -RunnerArgs @('-List')
        Assert ($code -eq 0) "R01 list exit code should be 0, got $code"
        $expectedList = $CanonicalOrder | ForEach-Object { "$_ $($SuiteMap[$_])" }
        Assert (($out -join "`n") -eq ($expectedList -join "`n")) "R01 list content mismatch"
        Assert (-not ($out | Where-Object { $_ -like "SUITE *" })) "R01 must contain no SUITE START line"
        Assert (0 -eq (Count-Markers)) "R01 must launch no child (no markers)"
    }

    Run-Row 'R02' {
        Clean-Markers
        $out, $code = Invoke-Verifier -RunnerPath $script:RunnerCopy -RunnerArgs @('-Suite', 'ZZTOP')
        Assert ($code -eq 2) "R02 unknown suite exit 2, got $code"
        Assert ([bool]($out -match 'unknown suite id')) "R02 should report unknown suite id"
        Assert (-not ($out | Where-Object { $_ -match '^SUITE .* START' })) "R02 must not start any suite"
        Assert (0 -eq (Count-Markers)) "R02 must launch no child (no markers)"
    }

    Run-Row 'R03' {
        Clean-Markers
        $out, $code = Invoke-Verifier -RunnerPath $script:RunnerCopy -RunnerArgs @('-Suite', 'J13A', 'J13A')
        Assert ($code -eq 2) "R03 duplicate suite exit 2, got $code"
        Assert ([bool]($out -match 'duplicate suite id')) "R03 should report duplicate suite id"
        Assert (-not ($out | Where-Object { $_ -match '^SUITE .* START' })) "R03 must not start any suite"
        Assert (0 -eq (Count-Markers)) "R03 must launch no child (no markers)"
    }

    Run-Row 'R04' {
        Clean-Markers
        Write-Stub 'J13A' 0 @('alpha one', 'alpha two')
        Write-Stub 'J14B' 0 @('beta one')
        Write-Stub 'J14C' 0 @('gamma one', 'gamma two', 'gamma three')
        $out, $code = Invoke-Verifier -RunnerPath $script:RunnerCopy -RunnerArgs @('-Suite', 'J13A', 'J14B', 'J14C')
        Assert ($code -eq 0) "R04 selected pass exit 0, got $code"
        $startIds = @($out | Where-Object { $_ -match '^SUITE \w+ START' } | ForEach-Object { ($_ -split ' ')[1] })
        Assert (($startIds -join ',') -eq 'J13A,J14B,J14C') "R04 START order should preserve supplied order, got $($startIds -join ',')"
        foreach ($id in @('J13A', 'J14B', 'J14C')) {
            $starts = @($out | Where-Object { $_ -eq "SUITE $id START $($SuiteMap[$id])" }).Count
            $passes = @($out | Where-Object { $_ -eq "SUITE $id PASS exit=0" }).Count
            Assert ($starts -eq 1) "R04 exactly one START line for $id (got $starts)"
            Assert ($passes -eq 1) "R04 exactly one PASS line for $id (got $passes)"
        }
        Assert ($out -contains 'J13A | alpha one') "R04 prefix J13A alpha one"
        Assert ($out -contains 'J13A | alpha two') "R04 prefix J13A alpha two"
        Assert ($out -contains 'J14B | beta one') "R04 prefix J14B beta one"
        Assert ($out -contains 'J14C | gamma one') "R04 prefix J14C gamma one"
        Assert ($out -contains 'J14C | gamma two') "R04 prefix J14C gamma two"
        Assert ($out -contains 'J14C | gamma three') "R04 prefix J14C gamma three"
        Assert (-not ($out | Where-Object { $_ -match '^alpha ' -or $_ -match '^beta ' -or $_ -match '^gamma ' })) "R04 child lines must be prefixed, no bare child line"
        Assert ($out -contains 'TOTAL: 3 suites, 3 passed, 0 failed') "R04 total"
        Assert ($out -contains 'RESULT: PASS') "R04 result PASS"
        Assert (3 -eq (Count-Markers)) "R04 three children launched"
    }

    Run-Row 'R05' {
        Clean-Markers
        Write-Stub 'J13A' 0 @('alpha ok')
        Write-Stub 'J13B' 3 @('beta bad')
        Write-Stub 'J14C' 0 @('gamma ok')
        $out, $code = Invoke-Verifier -RunnerPath $script:RunnerCopy -RunnerArgs @('-Suite', 'J13A', 'J13B', 'J14C')
        Assert ($code -eq 1) "R05 mixed exit 1, got $code"
        $startIds = @($out | Where-Object { $_ -match '^SUITE \w+ START' } | ForEach-Object { ($_ -split ' ')[1] })
        Assert (($startIds -join ',') -eq 'J13A,J13B,J14C') "R05 all three suites must start in order, got $($startIds -join ',')"
        Assert ($out -contains 'SUITE J13B FAIL exit=3') "R05 middle failure must report actual exit code 3"
        $idxFail = -1
        $idxLaterStart = -1
        for ($i = 0; $i -lt $out.Count; $i++) {
            if ($out[$i] -eq 'SUITE J13B FAIL exit=3') { $idxFail = $i }
            if ($out[$i] -eq "SUITE J14C START $($SuiteMap['J14C'])") { $idxLaterStart = $i }
        }
        Assert ($idxFail -ge 0 -and $idxLaterStart -ge 0) "R05 must locate middle failure and later start"
        Assert ($idxLaterStart -gt $idxFail) "R05 later suite must launch after middle failure"
        Assert ($out -contains 'TOTAL: 3 suites, 2 passed, 1 failed') "R05 total"
        Assert ($out -contains 'RESULT: FAIL') "R05 result FAIL"
        Assert (3 -eq (Count-Markers)) "R05 all three children launched"
    }

    Run-Row 'R06' {
        Clean-Markers
        Write-Stub 'J14C' 0 @('gamma ok')
        $missingPath = Join-Path $Root $SuiteMap['J14A']
        if (Test-Path -LiteralPath $missingPath) {
            Remove-Item -LiteralPath $missingPath -Force
        }
        Assert (-not (Test-Path -LiteralPath $missingPath)) "R06 J14A child must be omitted"
        $out, $code = Invoke-Verifier -RunnerPath $script:RunnerCopy -RunnerArgs @('-Suite', 'J14A', 'J14C')
        Assert ($code -eq 1) "R06 missing child exit 1, got $code"
        Assert ($out -contains "SUITE J14A FAIL exit=-1") "R06 missing child must produce one FAIL"
        Assert (-not ($out | Where-Object { $_ -match '^SUITE J14A PASS' })) "R06 missing child must not PASS"
        $trace = @($out | Where-Object {
            $_ -match '^At ' -or
            $_ -match 'ScriptStackTrace' -or
            $_ -match 'Exception:' -or
            $_ -match 'TerminatingError' -or
            $_ -match 'at <ScriptBlock>'
        })
        Assert ($trace.Count -eq 0) "R06 must not expose a PowerShell stack trace: $($trace -join ' | ')"
        Assert ($out -contains "SUITE J14C START $($SuiteMap['J14C'])") "R06 later selected stub must still launch"
        Assert ($out -contains 'SUITE J14C PASS exit=0') "R06 later selected stub must PASS"
        Assert ($out -contains 'TOTAL: 2 suites, 1 passed, 1 failed') "R06 honest total"
        Assert ($out -contains 'RESULT: FAIL') "R06 result FAIL"
        Assert (1 -eq (Count-Markers)) "R06 only the later child launched"
    }

    Assert (($script:executed -join ',') -eq 'R01,R02,R03,R04,R05,R06') "row sequence must be exactly R01..R06"

    Write-Output "============================================"
    Write-Output "TOTAL: 6 rows, 6 passed, 0 failed"
    Write-Output "ASSERTIONS: $script:assertions"
    Write-Output "============================================"
}
finally {
    if (Test-Path -LiteralPath $Root) {
        Remove-Item -LiteralPath $Root -Recurse -Force
    }
}

if (Test-Path -LiteralPath $Root) {
    throw "Temporary root was not cleaned up: $Root"
}

exit 0
