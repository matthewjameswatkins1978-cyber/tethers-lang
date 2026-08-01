[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$requiredTools = @('rg', 'fd', 'jq', 'yq', 'gh', 'just', 'git', 'pwsh')
$missing = @()

foreach ($tool in $requiredTools) {
    $command = Get-Command $tool -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($null -eq $command) {
        Write-Host "$tool : MISSING"
        $missing += $tool
        continue
    }

    $global:LASTEXITCODE = 0
    $version = & $command.Source --version 2>&1 | Select-Object -First 1
    if ($LASTEXITCODE -ne 0) {
        $version = 'version command failed'
    }
    Write-Host "$tool : $($command.Source) : $version"
}

Write-Host 'PATH changes require a new PowerShell, Windows Terminal, VS Code, OpenCode, or Codex process.'
if ($missing.Count -gt 0) {
    Write-Error "Missing required developer tools: $($missing -join ', ')"
    exit 1
}
