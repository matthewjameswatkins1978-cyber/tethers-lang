[CmdletBinding()]
param(
    [string]$PacketPath = 'docs/CURRENT_CLINE_TASK.md',
    [switch]$SkipWorktreeCheck
)

$ErrorActionPreference = 'Stop'
$python = Get-Command py -ErrorAction SilentlyContinue
$script = Join-Path $PSScriptRoot 'check-tethers-task-packet.py'
$arguments = @($script, '--packet', $PacketPath)
if ($SkipWorktreeCheck) { $arguments += '--skip-worktree-check' }

if ($null -ne $python) {
    & $python.Source -3 @arguments
}
else {
    & (Get-Command python -ErrorAction Stop).Source @arguments
}

if ($LASTEXITCODE -ne 0) {
    throw "Native task packet checker failed with exit code $LASTEXITCODE."
}
