# The Evil Bunny public CLI evidence driver (P6).
# Builds the Evil Bunny provider, then for every case runs the real public
# journey: plug pack -> plug inspect -> plug conform (default refusal) ->
# plug conform (approved non-isolated supervised execution).  Writes one
# evidence directory per case under evidence/<case>/ and prints a compact
# summary.  Read-only with respect to the source tree; it only writes under
# evidence/ and temporary directories.

[CmdletBinding()]
param(
    [string]$RepoRoot = (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))),
    [switch]$SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-Capture {
    param(
        [string]$Label,
        [System.IO.FileInfo]$Exe,
        [string[]]$Arguments,
        [string]$StdoutPath,
        [string]$ExitCodePath
    )
    $process = Start-Process -FilePath $Exe.FullName -ArgumentList $Arguments `
        -NoNewWindow -PassThru -RedirectStandardOutput $StdoutPath `
        -RedirectStandardError (Join-Path (Split-Path -Parent $StdoutPath) "$Label.stderr.txt")
    $process.WaitForExit()
    Set-Content -LiteralPath $ExitCodePath -Value $process.ExitCode -Encoding utf8NoBOM
    $process.ExitCode
}

$fixtureRoot = Join-Path $PSScriptRoot ".."
$fixtureRoot = [System.IO.Path]::GetFullPath($fixtureRoot)
$authorRoot = Join-Path $fixtureRoot "author"
$providerCrate = Join-Path $fixtureRoot "provider-rust"
$manifestFile = Join-Path $authorRoot "manifests\evil-probe-v1.json"
$evidenceRoot = Join-Path $fixtureRoot "evidence"
$hostExe = Join-Path $RepoRoot "tethers-0.1\host-rust\target\debug\tethers-reference-host.exe"

if (-not (Test-Path -LiteralPath $hostExe)) {
    throw "host binary not found: $hostExe (build tethers-0.1/host-rust first)"
}

if (-not $SkipBuild) {
    & cargo build --manifest-path (Join-Path $providerCrate "Cargo.toml") --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed for the Evil Bunny provider"
    }
}

$providerExe = Join-Path $providerCrate "target\debug\tethers_evil_bunny_provider.exe"
if (-not (Test-Path -LiteralPath $providerExe)) {
    throw "provider executable not found: $providerExe"
}

$caseDirs = @(Get-ChildItem -LiteralPath (Join-Path $authorRoot "cases") -Directory | Sort-Object Name)
if ($caseDirs.Count -eq 0) {
    throw "no Evil Bunny case directories found under author/cases"
}

if (Test-Path -LiteralPath $evidenceRoot) {
    Remove-Item -LiteralPath $evidenceRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $evidenceRoot | Out-Null

$rows = New-Object System.Collections.Generic.List[object]
$workRoot = Join-Path ([System.IO.Path]::GetTempPath()) "tethers-evil-bunny-proof-work"
if (Test-Path -LiteralPath $workRoot) {
    Remove-Item -LiteralPath $workRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $workRoot | Out-Null

try {
    foreach ($caseDir in $caseDirs) {
        $caseId = $caseDir.Name
        $caseEvidence = Join-Path $evidenceRoot $caseId
        New-Item -ItemType Directory -Path $caseEvidence | Out-Null

        $source = Join-Path $workRoot ($caseId + "-source")
        New-Item -ItemType Directory -Path (Join-Path $source "manifests") | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $source "provider") | Out-Null
        Copy-Item -LiteralPath (Join-Path $caseDir.FullName "plug.json") -Destination (Join-Path $source "plug.json")
        Copy-Item -LiteralPath $manifestFile -Destination (Join-Path $source "manifests\evil-probe-v1.json")
        Copy-Item -LiteralPath $providerExe -Destination (Join-Path $source "provider\tethers_evil_bunny_provider.exe")

        $package = Join-Path $workRoot ($caseId + ".tetherplug")

        $packExit = Invoke-Capture -Label "pack" -Exe $hostExe `
            -Arguments @("plug", "pack", "--source", $source, "--output", $package) `
            -StdoutPath (Join-Path $caseEvidence "pack.json") `
            -ExitCodePath (Join-Path $caseEvidence "pack.exit")

        $inspectExit = $null
        if ($packExit -eq 0) {
            $inspectExit = Invoke-Capture -Label "inspect" -Exe $hostExe `
                -Arguments @("plug", "inspect", "--package", $package) `
                -StdoutPath (Join-Path $caseEvidence "inspect.json") `
                -ExitCodePath (Join-Path $caseEvidence "inspect.exit")
        }

        $deniedExit = $null
        if ($packExit -eq 0) {
            $deniedExit = Invoke-Capture -Label "conform-denied" -Exe $hostExe `
                -Arguments @("plug", "conform", "--package", $package) `
                -StdoutPath (Join-Path $caseEvidence "conform-denied.json") `
                -ExitCodePath (Join-Path $caseEvidence "conform-denied.exit")
        }

        $approvedExit = $null
        if ($packExit -eq 0) {
            $approvedExit = Invoke-Capture -Label "conform-approved" -Exe $hostExe `
                -Arguments @("plug", "conform", "--package", $package, "--allow-non-isolated-supervised-execution") `
                -StdoutPath (Join-Path $caseEvidence "conform-approved.json") `
                -ExitCodePath (Join-Path $caseEvidence "conform-approved.exit")
        }

        $approvedStatus = "n/a"
        $approvedCode = $null
        if ($approvedExit -ne $null) {
            $raw = Get-Content -LiteralPath (Join-Path $caseEvidence "conform-approved.json") -Raw
            $envJson = $null
            if ($raw.Trim().Length -gt 0) {
                try { $envJson = $raw | ConvertFrom-Json } catch { $envJson = $null }
            }
            if ($envJson) {
                $approvedStatus = $envJson.status
                $approvedCode = $envJson.data.conformance.disposition
            }
        }

        $deniedStatus = "n/a"
        if ($deniedExit -ne $null) {
            $raw = Get-Content -LiteralPath (Join-Path $caseEvidence "conform-denied.json") -Raw
            $envJson = $null
            if ($raw.Trim().Length -gt 0) {
                try { $envJson = $raw | ConvertFrom-Json } catch { $envJson = $null }
            }
            if ($envJson) { $deniedStatus = $envJson.status }
        }

        $rows.Add([PSCustomObject]@{
            case = $caseId
            pack = $packExit
            inspect = $inspectExit
            denied = $deniedExit
            denied_status = $deniedStatus
            approved = $approvedExit
            approved_status = $approvedStatus
            disposition = $approvedCode
        })
        Write-Host ("{0}: pack={1} inspect={2} denied={3}({4}) approved={5}({6}) disp={7}" -f `
            $caseId, $packExit, $inspectExit, $deniedExit, $deniedStatus, `
            $approvedExit, $approvedStatus, $approvedCode)
    }

    $rows | Format-Table -AutoSize
    $rows | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidenceRoot "summary.json") -Encoding utf8NoBOM
    Write-Host "Evidence written under: $evidenceRoot"
}
finally {
    if (Test-Path -LiteralPath $workRoot) {
        Remove-Item -LiteralPath $workRoot -Recurse -Force
    }
}
