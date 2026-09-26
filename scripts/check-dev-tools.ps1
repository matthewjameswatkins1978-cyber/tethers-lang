[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# REQUIRED tools fail the check when missing. OPTIONAL tools are reported but
# never fail it:
#   rg   - fast search convenience (substitute: Select-String); not invoked by any
#          build, test, packaging, or verification script.
#   fd   - file-find convenience; not invoked by any repository script or workflow.
#   jq   - required only on Linux CI (scripts/run-rust-tests.sh,
#          scripts/prepare-current-engine.sh) and recorded informationally by
#          scripts/verify-tethers.py; Ubuntu lanes install it there. Not required
#          on Windows.
#   yq   - not invoked by any repository script or workflow.
#   just - preferred aggregate runner for the justfile routes (`just verify`, ...);
#          every recipe has a directly invocable underlying command (see justfile),
#          so its absence degrades convenience, not the build contract.
$requiredTools = @('gh', 'git', 'pwsh')
$optionalTools = @('rg', 'fd', 'jq', 'yq', 'just')
$missingRequired = @()
$missingOptional = @()

function Test-Tool {
    param([string]$Name)
    $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($null -eq $command) {
        return $null
    }

    $global:LASTEXITCODE = 0
    $version = & $command.Source --version 2>&1 | Select-Object -First 1
    if ($LASTEXITCODE -ne 0) {
        $version = 'version command failed'
    }
    Write-Host "$Name : $($command.Source) : $version"
    return $command
}

foreach ($tool in $requiredTools) {
    if ($null -eq (Test-Tool -Name $tool)) {
        Write-Host "$tool : MISSING (REQUIRED)"
        $missingRequired += $tool
    }
}

foreach ($tool in $optionalTools) {
    if ($null -eq (Test-Tool -Name $tool)) {
        Write-Host "$tool : MISSING (OPTIONAL)"
        $missingOptional += $tool
    }
}

Write-Host 'PATH changes require a new PowerShell, Windows Terminal, VS Code, OpenCode, or Codex process.'
if ($missingOptional.Count -gt 0) {
    Write-Host "Optional tools not found: $($missingOptional -join ', ')"
}
if ($missingRequired.Count -gt 0) {
    Write-Error "Missing required developer tools: $($missingRequired -join ', ')"
    exit 1
}
